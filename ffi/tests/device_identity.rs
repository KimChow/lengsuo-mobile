//! 移动端设备身份模型（设备 ID / 公钥 / NAT 候选）的**跨端契约测试**。
//!
//! # 规格来源
//!
//! 校验规则逐字对齐服务端已落地的设备身份行为（任务书给出的**行为规格**）；
//! 本仓独立实现，⛔ 不引用服务端源码、⛔ 不依赖服务端 crate。
//! 两端一致性靠 `tests/vectors/device_identity_v1.json` 这份**跨端测试向量**：
//! Swift / Kotlin 侧将来读同一份文件逐条对拍，**不靠真机**。
//!
//! # 覆盖点
//!
//! - 设备 ID：ASCII 字母数字与 `-` `_` `.` `:` 合法；空 / 全空白 / 超 128
//!   **字节** / 含 `/` `?` `#` `%` / 非 ASCII ⇒ 一律 `InvalidDeviceId`
//!   （**无匿名设备兜底**，fail-closed）；
//! - 公钥：`trim` 后 16..=512 **字节**；字母表覆盖 base64 / base64url / hex；
//! - 指纹：FNV-1a 64 位（**非密码学摘要**）⇒ 16 位小写十六进制，三个黄金值钉死；
//! - NAT 候选：`kind` 三值**精确**匹配（拼错 / 大小写不同都拒绝）；地址非空、
//!   ≤128 字节、无空白或控制字符；
//! - 错误码：本地 4 个变体逐一映射 50100 / 50102 / 50104 / 50105；
//! - ⛔ 错误值不携带任何输入片段（无密钥字节、无长度 / 计数器等元数据）。
//!
//! 向量文件读取复用 `tests/vectors/json.rs` 的极小 JSON 解析器（⛔ 不新增依赖）；
//! 本文件**不改**该目录下任何既有文件。

mod vectors;

use lengsuo_mobile_ffi::core::device::{
    fnv1a64_hex, DeviceError, DeviceId, NatCandidate, PublicKey, SERVER_ERROR_CODES,
};
use vectors::json::{self, Json};

/// 内嵌的跨端向量原文（与磁盘文件逐字节相同）。
const VECTORS_JSON: &str = include_str!("vectors/device_identity_v1.json");

/// 生成指定长度的同字符文本。
fn repeat_of(ch: char, len: usize) -> String {
    std::iter::repeat_n(ch, len).collect()
}

/// 解析指定长度的公钥并返回其长度。
fn key_len(len: usize) -> usize {
    PublicKey::parse(&repeat_of('A', len))
        .unwrap()
        .as_str()
        .len()
}

/// 断言设备 ID 被拒绝，并返回错误值。
fn id_err(input: &str) -> DeviceError {
    DeviceId::new(input).expect_err("设备 ID 必须被拒绝")
}

/// 断言公钥被拒绝，并返回错误值。
fn key_err(input: &str) -> DeviceError {
    PublicKey::parse(input).expect_err("公钥必须被拒绝")
}

/// 断言 NAT 候选被拒绝，并返回错误值。
fn cand_err(kind: &str, address: &str) -> DeviceError {
    NatCandidate::new(kind, address, 0).expect_err("NAT 候选必须被拒绝")
}

/// 断言一组设备 ID 全部被拒（错误名固定为 `InvalidDeviceId`）。
fn assert_ids_rejected(inputs: &[&str]) {
    for input in inputs {
        assert_eq!(id_err(input).stable_name(), "InvalidDeviceId", "{input:?}");
    }
}

/// 断言一组公钥全部被拒（错误名固定为 `InvalidPublicKey`）。
fn assert_keys_rejected(inputs: &[&str]) {
    for input in inputs {
        assert_eq!(
            key_err(input).stable_name(),
            "InvalidPublicKey",
            "{input:?}"
        );
    }
}

/// 断言一个 NAT 候选被拒（错误名固定为 `InvalidCandidate`）。
fn assert_cand_rejected(kind: &str, address: &str) {
    let error = cand_err(kind, address);
    assert_eq!(
        error.stable_name(),
        "InvalidCandidate",
        "{kind:?} {address:?}"
    );
}

/// 取 JSON 布尔值（`json.rs` 没有 `as_bool`，且该文件不在本卡可改范围）。
fn as_bool(value: &Json) -> bool {
    match value {
        Json::Bool(flag) => *flag,
        other => panic!("期望布尔值，实际 {other:?}"),
    }
}

/// 取向量用例的某个字符串字段（**缺字段即 panic** ⇒ 字段齐全性由此保证）。
fn field<'a>(case: &'a Json, key: &str) -> &'a str {
    case.field(key).as_str()
}

// ---------------------------------------------------------------- 设备 ID

#[test]
fn device_id_accepts_ascii_alnum_and_four_separators() {
    for input in [
        "A", "z", "0", "9", "dev-1", "dev_1", "dev.1", "dev:1", "Az09-_.:",
    ] {
        let parsed = DeviceId::new(input).expect("合法设备 ID 必须通过");
        assert_eq!(parsed.as_str(), input, "设备 ID 必须原样保留（不 trim）");
    }
}

#[test]
fn device_id_rejects_malformed_inputs() {
    // 空 / 全空白：不存在「匿名设备」兜底。
    assert_ids_rejected(&["", " ", "   ", "\t", "\n", " \t\r\n "]);
    // 长度按**字节**：128 合法，129 拒绝。
    assert_eq!(
        DeviceId::new(&repeat_of('a', 128)).unwrap().as_str().len(),
        128
    );
    assert_ids_rejected(&[&repeat_of('a', 129)]);
    // 设备 ID 会原样出现在 URL 路径里 ⇒ 这四个字符必须被拒。
    let urls: Vec<String> = ['/', '?', '#', '%']
        .iter()
        .map(|c| format!("dev{c}1"))
        .collect();
    let refs: Vec<&str> = urls.iter().map(String::as_str).collect();
    assert_ids_rejected(&refs);
    // 非 ASCII 一律拒绝。
    assert_ids_rejected(&["设备-1", "café", "dev-🔒", "dev\u{00a0}1"]);
}

// ---------------------------------------------------------------- 公钥

#[test]
fn public_key_enforces_byte_length_bounds() {
    assert_keys_rejected(&[&repeat_of('A', 15)]);
    assert_eq!(key_len(16), 16);
    assert_eq!(key_len(512), 512);
    assert_keys_rejected(&[&repeat_of('A', 513)]);
}

#[test]
fn public_key_accepts_valid_alphabets_and_trims_whitespace() {
    // base64 / base64url（带或不带 padding）/ hex 都在字母表内。
    for input in [
        "AAAA+BBB/CCC==DDDD",
        "AAAA-BBB_CCCDDDD==",
        "0123456789abcdef-_+/ABCDEF",
    ] {
        assert_eq!(PublicKey::parse(input).unwrap().as_str(), input);
    }
    let raw = "  AAAAAAAAAAAAAAAA  ";
    let parsed = PublicKey::parse(raw).expect("trim 后长度合法必须通过");
    assert_eq!(parsed.as_str(), "AAAAAAAAAAAAAAAA");
    assert_eq!(parsed.as_str(), raw.trim());
}

#[test]
fn public_key_rejects_illegal_chars() {
    assert_keys_rejected(&[
        "AAAAAAAAAAAAAAAA!",
        "AAAA AAAAAAAAAAAAAA",
        "AAAA\tAAAAAAAAAAAA",
        "AAAA\nAAAAAAAAAAAA",
        "AAAA:AAAAAAAAAAAAA",
    ]);
}

// ---------------------------------------------------------------- 指纹

#[test]
fn fnv1a64_golden_values_and_fingerprint_shape() {
    // 服务端自带的三个黄金值：算法一旦漂移，这三条先红。
    assert_eq!(fnv1a64_hex(b""), "cbf29ce484222325");
    assert_eq!(fnv1a64_hex(b"a"), "af63dc4c8601ec8c");
    assert_eq!(fnv1a64_hex(b"foobar"), "85944171f73967e8");

    let key = PublicKey::parse("AAAAAAAAAAAAAAAA").unwrap();
    let fingerprint = key.fingerprint();
    assert_eq!(fingerprint.len(), 16, "指纹恒为 16 位十六进制");
    for ch in fingerprint.chars() {
        let lower_hex = ch.is_ascii_digit() || ('a'..='f').contains(&ch);
        assert!(lower_hex, "指纹必须全小写十六进制：{fingerprint}");
    }
    assert_eq!(fingerprint, fnv1a64_hex(key.as_str().as_bytes()));
    // 指纹是展示用的稳定值，不是密码学摘要：同一输入恒得同一值。
    assert_eq!(fingerprint, key.fingerprint());
}

// ---------------------------------------------------------------- NAT 候选

#[test]
fn nat_candidate_accepts_all_kinds_and_preserves_fields() {
    assert_eq!(NatCandidate::KINDS, ["host", "srflx", "relay"]);
    for kind in NatCandidate::KINDS {
        let candidate = NatCandidate::new(kind, "192.0.2.1:3478", 100).unwrap();
        assert_eq!(candidate.kind(), kind);
        assert_eq!(candidate.address(), "192.0.2.1:3478");
        assert_eq!(candidate.priority(), 100);
    }
    let ipv6 = NatCandidate::new("relay", "[2001:db8::1]:3478", 7).unwrap();
    assert_eq!(ipv6.kind(), "relay");
    assert_eq!(ipv6.address(), "[2001:db8::1]:3478");
    assert_eq!(ipv6.priority(), 7);
}

#[test]
fn nat_candidate_rejects_unknown_kind_or_malformed_address() {
    // 未知 kind 必须 fail-closed，不得静默当作 host（含大小写不同的拼写）。
    for kind in ["", "srflx2", "HOST", "Host", "Relay", "prflx", " host"] {
        assert_cand_rejected(kind, "192.0.2.1:3478");
    }
    // 地址：空 / 全空白 / 超 128 字节 / 含空白或控制字符，一律拒绝。
    for address in ["", " ", "\t\t", "  \r\n "] {
        assert_cand_rejected("host", address);
    }
    assert!(NatCandidate::new("host", &repeat_of('a', 128), 0).is_ok());
    assert_cand_rejected("host", &repeat_of('a', 129));
    for address in [
        "192.0.2.1: 3478",
        " 192.0.2.1:3478",
        "192.0.2.1:\t3478",
        "192.0.2.1:3478\n",
        "192.0.2.1:34\u{0007}78",
    ] {
        assert_cand_rejected("srflx", address);
    }
}

// ---------------------------------------------------------------- 错误契约

#[test]
fn error_codes_and_stable_names_match_server_contract() {
    assert_eq!(id_err("").server_code(), 50100);
    assert_eq!(key_err("short").server_code(), 50102);
    assert_eq!(cand_err("bogus", "192.0.2.1:1").server_code(), 50104);
    assert_eq!(DeviceError::DeviceOwnershipConflict.server_code(), 50105);

    let variants = [
        DeviceError::InvalidDeviceId,
        DeviceError::InvalidPublicKey,
        DeviceError::InvalidCandidate,
        DeviceError::DeviceOwnershipConflict,
    ];
    let names = [
        "InvalidDeviceId",
        "InvalidPublicKey",
        "InvalidCandidate",
        "DeviceOwnershipConflict",
    ];
    for (variant, name) in variants.iter().zip(names) {
        assert_eq!(variant.stable_name(), name);
        let entry = (variant.stable_name(), variant.server_code());
        assert!(SERVER_ERROR_CODES.contains(&entry), "常量表缺 {entry:?}");
    }
    let mut seen: Vec<&str> = SERVER_ERROR_CODES.iter().map(|(name, _)| *name).collect();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), SERVER_ERROR_CODES.len(), "常量表不得重名");
}

#[test]
fn errors_never_carry_input_bytes() {
    let secret_id = "device/secret?token#fragment%2F";
    let secret_key = "SECRETKEYBYTES!!!!!!!!!!!!!!!!";
    let secret_addr = "10.0.0.1: \tSECRETADDR";
    let errors = [
        (id_err(secret_id), secret_id),
        (key_err(secret_key), secret_key),
        (cand_err("host", secret_addr), "SECRETADDR"),
    ];
    for (error, fragment) in errors {
        let debug = format!("{error:?}");
        let display = error.to_string();
        for text in [debug.as_str(), display.as_str()] {
            assert!(!text.contains(fragment), "错误值不得携带输入：{text}");
            assert!(!text.contains("SECRET"), "错误值不得携带输入：{text}");
        }
        // 变体无字段 ⇒ 错误本身不可能装下长度/计数器等元数据。
        assert_eq!(std::mem::size_of::<DeviceError>(), 1);
    }
}

// ---------------------------------------------------------------- 跨端向量

#[test]
fn vectors_json_is_canonical() {
    let rendered = json::render(&json::parse(VECTORS_JSON));
    assert_eq!(rendered, VECTORS_JSON, "向量原文必须是规范化 JSON");
}

#[test]
fn vectors_are_complete_and_match_implementation() {
    let root = json::parse(VECTORS_JSON);
    assert_eq!(
        root.field("format").as_str(),
        "lengsuo-device-identity-vectors"
    );
    assert_eq!(root.field("version").as_u64(), 1);

    for golden in root.field("fnv1a64_golden").as_arr() {
        let bytes = vectors::from_hex(golden.field("input_hex").as_str());
        let expect = golden.field("expect_hex").as_str();
        assert_eq!(fnv1a64_hex(&bytes), expect, "FNV 黄金值不符");
    }

    let cases = root.field("cases").as_arr();
    assert!(cases.len() >= 24, "向量用例不得少于 24 条");
    let mut names: Vec<&str> = cases.iter().map(|c| field(c, "name")).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), cases.len(), "用例名不得重复");

    for case in cases {
        // `field()` 缺字段即 panic ⇒ 每条用例的字段必须齐全。
        let name = field(case, "name");
        let kind = field(case, "kind");
        let input = field(case, "input");
        let nat_kind = field(case, "nat_kind");
        let priority = case.field("priority").as_u64() as u32;
        let valid = as_bool(case.field("expect_valid"));
        let code = case.field("expect_code").as_u64() as u32;
        let stable_name = field(case, "expect_stable_name");
        let canonical = field(case, "expect_canonical");
        let fingerprint = field(case, "expect_fingerprint");

        assert_eq!(valid, code == 0, "用例 {name}：code 矛盾");
        assert_eq!(valid, stable_name.is_empty(), "用例 {name}：名矛盾");
        let is_cand = kind == "nat_candidate";
        assert_eq!(is_cand, !nat_kind.is_empty(), "用例 {name}：kind 矛盾");
        assert!(is_cand || priority == 0, "用例 {name}：不该有 priority");
        let has_fp = kind == "public_key" && valid;
        assert_eq!(has_fp, !fingerprint.is_empty(), "用例 {name}：指纹矛盾");
        assert_eq!(!valid, canonical.is_empty(), "用例 {name}：结果矛盾");

        // 与实现逐条对拍。
        let outcome: Result<(String, String), DeviceError> = match kind {
            "device_id" => DeviceId::new(input).map(|v| (v.as_str().to_string(), String::new())),
            "public_key" => {
                PublicKey::parse(input).map(|v| (v.as_str().to_string(), v.fingerprint()))
            }
            "nat_candidate" => NatCandidate::new(nat_kind, input, priority)
                .map(|v| (v.address().to_string(), String::new())),
            other => panic!("未知向量 kind {other}（用例 {name}）"),
        };
        match outcome {
            Ok((got, got_fp)) => {
                assert!(valid, "用例 {name}：期望拒绝却通过");
                assert_eq!(code, 0, "用例 {name}：合法输入码必须为 0");
                assert_eq!(stable_name, "", "用例 {name}：合法输入不写名");
                assert_eq!(got, canonical, "用例 {name}：结果不符");
                assert_eq!(got_fp, fingerprint, "用例 {name}：指纹不符");
            }
            Err(error) => {
                assert!(!valid, "用例 {name}：期望通过却拒绝");
                assert_eq!(error.server_code(), code, "用例 {name}：码不符");
                assert_eq!(error.stable_name(), stable_name, "用例 {name}：名不符");
            }
        }
    }
}
