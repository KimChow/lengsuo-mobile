//! **跨端帧向量对拍**：把 `lengsuo-link-core` 的 L3 帧向量搬进移动端仓，
//! 让门面层的无状态帧编解码逐字节对拍权威字节。
//!
//! 与 `core_vectors.rs`（自证型）的区别：那份测试自己造明文、自己封自己解；
//! 本文件读的是**源仓另一侧产出的期望整帧字节**（`frame_hex`），
//! 证明「门面层解析 / 重新编码出来的字节 == 跨端约定的字节」。
//!
//! # 对拍边界（为什么只对「帧格式」，不对「AEAD 封印」）
//!
//! `frame_hex` = 20 字节帧头 + AEAD 密文包。门面层**刻意不依赖
//! `lengsuo-wireguard`**：封 / 解密文需要的 `Sender` / `Receiver` 由
//! `lengsuo-link-core` 的会话内部持有，门面没有 API 能拿「测试密钥 + 接收者
//! 标识 + 起始计数器」直接构造传输上下文（会话握手密钥是派生值、且握手含墙钟）。
//! ⇒ 本卡对拍的是门面层确实暴露的那段：`decode_exact` 解析 authoritative 字节、
//! `encode` 再逐字节还原 authoritative 字节（帧头字段 + CRC 全部覆盖）。
//! 完整的明文↔密文往返属于 `lengsuo-link-core` 自己的三重校验，见 `SOURCE.md`。
//!
//! # 固定测试密钥
//!
//! 向量里的密钥是**测试专用**（全 `0x5A`），仅用于对拍，**不得**用于真实会话。

mod vectors;

use lengsuo_mobile_ffi::core::{
    decode_exact, encode, MAX_FRAME_LEN, MAX_FRAME_PAYLOAD_LEN, MAX_PLAIN_LEN,
};

/// 向量文件必须存在、非空，且与编译期内嵌文本逐字节相同。
#[test]
fn vector_file_is_present_and_not_empty() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/vectors/frame_vectors.json"
    );
    let on_disk = std::fs::read_to_string(path).expect("向量文件必须存在且可读");
    assert!(!on_disk.is_empty(), "向量文件不能为空");
    assert_eq!(
        on_disk,
        vectors::VECTORS_JSON,
        "内嵌文本必须与磁盘文件逐字节相同"
    );
}

/// 向量集非空，且「解析后重新渲染」与原文逐字节相同（规范化校验）。
#[test]
fn vectors_json_is_canonical() {
    let value = vectors::json::parse(vectors::VECTORS_JSON);
    assert_eq!(
        vectors::json::render(&value),
        vectors::VECTORS_JSON,
        "解析后重新渲染必须与向量原文逐字节相同"
    );
}

/// 向量文件非空，且 case 数量 > 0（文件级元数据由 `parse_cases` 校验）。
#[test]
fn vector_set_is_non_empty_and_has_cases() {
    assert!(!vectors::VECTORS_JSON.trim().is_empty(), "向量文件不能为空");
    let cases = vectors::parse_cases(vectors::VECTORS_JSON);
    assert!(!cases.is_empty(), "向量 case 数量必须 > 0");
}

/// 逐字节对拍（本卡核心判据）：门面 `decode_exact` 解析 authoritative 字节，
/// `encode` 再**逐字节**还原期望整帧。
#[test]
fn each_vector_decodes_and_reencodes_byte_for_byte() {
    for case in vectors::parse_cases(vectors::VECTORS_JSON) {
        let decoded = decode_exact(&case.frame).unwrap_or_else(|error| {
            panic!("向量 {} 必须可解码：{}", case.name, error.stable_name())
        });
        assert_eq!(
            decoded.frame_type, case.frame_type,
            "向量 {} 帧类型",
            case.name
        );
        assert_eq!(decoded.seq, case.seq, "向量 {} 序号", case.name);
        assert_eq!(
            decoded.payload.as_slice(),
            &case.frame[vectors::header_len(&case.frame)..],
            "向量 {} 载荷切分",
            case.name
        );
        let reencoded =
            encode(case.frame_type, case.seq, &decoded.payload).unwrap_or_else(|error| {
                panic!("向量 {} 重新编码失败：{}", case.name, error.stable_name())
            });
        assert_eq!(
            reencoded, case.frame,
            "向量 {} 门面编码必须逐字节还原期望整帧",
            case.name
        );
    }
}

/// 帧头布局对拍：authoritative 字节里的每个头字段都与 JSON 元数据一致。
#[test]
fn each_vector_header_layout_matches_the_protocol() {
    let cases = vectors::parse_cases(vectors::VECTORS_JSON);
    let (header_len, _) = vectors::frame_metrics(&cases);
    for case in &cases {
        assert!(
            case.frame.len() >= header_len,
            "向量 {} 帧长不足帧头",
            case.name
        );
        assert_eq!(&case.frame[0..2], &[0x4C, 0x53], "向量 {} 魔数", case.name);
        assert_eq!(case.frame[2], 1, "向量 {} 版本", case.name);
        assert_eq!(
            case.frame[3],
            case.frame_type.to_u8(),
            "向量 {} 帧类型字节",
            case.name
        );
        assert_eq!(
            u64::from_be_bytes(case.frame[4..12].try_into().expect("8 字节序号")),
            case.seq,
            "向量 {} 序号",
            case.name
        );
        assert_eq!(
            u16::from_be_bytes([case.frame[14], case.frame[15]]),
            0,
            "向量 {} 保留位必须为 0",
            case.name
        );
    }
}

/// 门面上限一致性：`MAX_FRAME_PAYLOAD_LEN` / `MAX_PLAIN_LEN` 由 `MAX_FRAME_LEN`
/// 减「帧头长 / 传输层开销」推出，帧头长与开销从 authoritative 字节推导，
/// **不写死** 20 / 32 / 1400 / 1368 这些数字。
#[test]
fn facade_length_limits_are_consistent_with_the_vectors() {
    let cases = vectors::parse_cases(vectors::VECTORS_JSON);
    let (header_len, overhead) = vectors::frame_metrics(&cases);
    assert_eq!(
        MAX_FRAME_PAYLOAD_LEN,
        MAX_FRAME_LEN - header_len,
        "MAX_FRAME_PAYLOAD_LEN 必须 = MAX_FRAME_LEN − 帧头长"
    );
    assert_eq!(
        MAX_PLAIN_LEN,
        MAX_FRAME_PAYLOAD_LEN - overhead,
        "MAX_PLAIN_LEN 必须 = MAX_FRAME_PAYLOAD_LEN − 传输层开销"
    );
    for case in &cases {
        assert!(
            case.frame.len() <= MAX_FRAME_LEN,
            "向量 {} 整帧不得超过上限",
            case.name
        );
        assert!(
            case.plaintext.len() <= MAX_PLAIN_LEN,
            "向量 {} 明文不得超过上限",
            case.name
        );
    }
}
