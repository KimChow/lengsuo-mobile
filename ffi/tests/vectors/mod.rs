//! 跨端帧向量的读取端：内嵌 JSON 原文、十六进制工具、向量模型。
//!
//! 这个目录下的文件不会被 Cargo 当作独立测试目标；`cross_end_vectors.rs`
//! 用 `mod vectors;` 引入。数据文件 `frame_vectors.json` 由 Cargo 在编译期
//! 经 `include_str!` 内嵌，运行时不再依赖工作目录。
#![allow(dead_code)] // 每个测试二进制只用到其中一部分工具。

pub mod json;

use lengsuo_mobile_ffi::core::FrameType;

/// 内嵌的权威向量原文（与 `ffi/tests/vectors/frame_vectors.json` 逐字节相同）。
///
/// 来源与不变式见同目录 `SOURCE.md`。
pub const VECTORS_JSON: &str = include_str!("frame_vectors.json");

/// 测试专用密钥（全 `0x5A`）。**仅用于测试**，不得用于任何真实会话。
pub const TEST_KEY: [u8; 32] = [0x5A; 32];

/// 期望的接收者标识（与 JSON 的 `receiver` 一致）。
pub const VECTOR_RECEIVER: u32 = 0x1122_3344;

/// 一条测试向量。
#[derive(Debug, Clone)]
pub struct Case {
    /// 向量名（用于断言信息）。
    pub name: String,
    /// 帧类型。
    pub frame_type: FrameType,
    /// 帧序号。
    pub seq: u64,
    /// 传输层计数器（决定 nonce；密文逐字节可复现的关键输入）。
    pub counter: u64,
    /// 明文。
    pub plaintext: Vec<u8>,
    /// 期望的整帧字节。
    pub frame: Vec<u8>,
}

/// 由名字还原帧类型。
pub fn frame_type_from_name(name: &str) -> FrameType {
    match name {
        "data" => FrameType::Data,
        "control" => FrameType::Control,
        "keepalive" => FrameType::Keepalive,
        other => panic!("未知帧类型 {other}"),
    }
}

/// 十六进制解码（拒绝奇数长度与非法字符）。
pub fn from_hex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0, "十六进制长度必须是偶数：{text}");
    let mut out = Vec::with_capacity(text.len() / 2);
    let (pairs, _remainder) = text.as_bytes().as_chunks::<2>();
    for pair in pairs {
        let high = (pair[0] as char).to_digit(16).expect("非法十六进制字符");
        let low = (pair[1] as char).to_digit(16).expect("非法十六进制字符");
        out.push(((high << 4) | low) as u8);
    }
    out
}

/// 十六进制小写编码。
pub fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut out, "{byte:02x}").expect("写字符串失败");
    }
    out
}

/// 解析整个向量文件，并对**文件级**元数据做一致性校验。
pub fn parse_cases(text: &str) -> Vec<Case> {
    let root = json::parse(text);
    assert_eq!(
        root.field("format").as_str(),
        "lengsuo-link-frame-vectors",
        "向量 format 字段不匹配"
    );
    assert_eq!(root.field("version").as_u64(), 1, "向量 version 字段不匹配");
    assert_eq!(
        root.field("spec").as_str(),
        "docs/link-frame-spec.md",
        "向量 spec 字段不匹配"
    );
    assert_eq!(
        from_hex(root.field("key_hex").as_str()),
        TEST_KEY.to_vec(),
        "向量 key_hex 必须是测试专用密钥（全 0x5A）"
    );
    assert_eq!(
        root.field("receiver").as_u64(),
        u64::from(VECTOR_RECEIVER),
        "向量 receiver 字段不匹配"
    );

    root.field("vectors")
        .as_arr()
        .iter()
        .map(|item| Case {
            name: item.field("name").as_str().to_string(),
            frame_type: frame_type_from_name(item.field("frame_type").as_str()),
            seq: item.field("seq").as_u64(),
            counter: item.field("counter").as_u64(),
            plaintext: from_hex(item.field("plaintext_hex").as_str()),
            frame: from_hex(item.field("frame_hex").as_str()),
        })
        .collect()
}

/// 帧头声明的载荷长度（帧头第 12..14 字节，大端）。
pub fn payload_len_field(frame: &[u8]) -> usize {
    u16::from_be_bytes([frame[12], frame[13]]) as usize
}

/// 帧头长度（= 帧总长 − 载荷声明长），由权威向量本身推导，不写死 20。
pub fn header_len(frame: &[u8]) -> usize {
    frame.len() - payload_len_field(frame)
}

/// 从全部向量推导「帧头长度」与「传输层开销」，并在向量间校验一致。
///
/// - 帧头长度 = 帧总长 − 载荷声明长；
/// - 传输层开销 = 载荷声明长 − 明文长度；
///
/// 两者都从权威字节推出，不在测试里写死协议常量。
pub fn frame_metrics(cases: &[Case]) -> (usize, usize) {
    let mut header_len = None;
    let mut overhead = None;
    for case in cases {
        let declared = payload_len_field(&case.frame);
        let this_header = case.frame.len() - declared;
        let this_overhead = declared - case.plaintext.len();
        match header_len {
            None => header_len = Some(this_header),
            Some(value) => assert_eq!(value, this_header, "向量 {} 帧头长度不一致", case.name),
        }
        match overhead {
            None => overhead = Some(this_overhead),
            Some(value) => assert_eq!(value, this_overhead, "向量 {} 传输层开销不一致", case.name),
        }
    }
    (
        header_len.expect("至少一条向量"),
        overhead.expect("至少一条向量"),
    )
}
