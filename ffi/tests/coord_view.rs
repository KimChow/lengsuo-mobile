//! 移动端协调面设备视图与权威在线判定的**跨端契约测试**。
//!
//! # 规格来源
//!
//! 逐字对齐服务端已落地的设备视图与在线判定口径（任务书给出的**行为规格**）；
//! 本仓独立实现，⛔ 不引用服务端源码、⛔ 不依赖服务端 crate。
//!
//! # 覆盖点
//!
//! - `DeviceView` / `PeerView` 构造与逐字段保真；
//! - `Availability` 三态与 `is_usable`（Online 可用，Offline / Unknown 不可用）；
//! - `online == false` 是成功响应，不是解析错误；
//! - 设备 ID / 公钥 / 指纹非法 ⇒ 对应错误；
//! - 指纹只做格式校验（16 位小写十六进制），⛔ 不重算、⛔ 不做密码学判定；
//! - `CoordViewError` 全部无字段（不泄漏任何输入）；
//! - 六个错误变体的 `server_code()` 与稳定名字逐条对齐；
//! - FNV-1a 黄金值四条（与设备身份模块共用 `fnv1a64_hex`，两端一致）；
//! - 门面层源码不出现 IO / 线程 / 本地时钟。
//!
//! 向量文件读取复用 `tests/vectors/json.rs` 的极小 JSON 解析器（⛔ 不新增依赖）。
//! 本文件**不改**该目录下任何既有文件。

mod vectors;

use lengsuo_mobile_ffi::core::coord_view::{Availability, CoordViewError, DeviceView, PeerView};
use lengsuo_mobile_ffi::core::device::{fnv1a64_hex, NatCandidate, PublicKey};
use vectors::json::{self, Json};

/// 内嵌的跨端向量原文（与磁盘文件逐字节相同）。
const VECTORS_JSON: &str = include_str!("vectors/coord_view_v1.json");

/// 门面层源码（用于「不做 IO / 不取本地时钟」的静态断言）。
const SOURCE: &str = include_str!("../src/core/coord_view.rs");

/// 合法指纹（16 位小写十六进制；只校验格式，不与公钥交叉验证）。
const VALID_FINGERPRINT: &str = "3e9e7c32b2e9f4c1";

/// 全部错误变体（按声明顺序）。
const ALL_ERRORS: [CoordViewError; 6] = [
    CoordViewError::InvalidDeviceId,
    CoordViewError::InvalidPublicKey,
    CoordViewError::InvalidFingerprint,
    CoordViewError::UnknownDevice,
    CoordViewError::BindingMismatch,
    CoordViewError::DeviceOffline,
];

/// 六个变体的稳定名字（跨端契约，按名字分支）。
const ERROR_NAMES: [&str; 6] = [
    "InvalidDeviceId",
    "InvalidPublicKey",
    "InvalidFingerprint",
    "UnknownDevice",
    "BindingMismatch",
    "DeviceOffline",
];

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

/// 合法设备 ID / 公钥的固定样本（用于错误隔离）。
const DEVICE_ID: &str = "dev0101";
const PUBLIC_KEY: &str = "AAAAAAAAAAAAAAAA";

/// 以固定 `last_seen_at_ms = 0` 构造设备视图（`last_seen` 不参与可用性判定）。
fn probe(
    device_id: &str,
    online: bool,
    public_key: &str,
    fingerprint: &str,
) -> Result<DeviceView, CoordViewError> {
    DeviceView::new(device_id, online, public_key, fingerprint, 0)
}

/// 按**稳定名字**反查错误变体（跨端契约按名分支）。
fn error_by_name(name: &str) -> CoordViewError {
    ALL_ERRORS
        .iter()
        .copied()
        .find(|error| error.stable_name() == name)
        .unwrap_or_else(|| panic!("未知错误名 {name}"))
}

// ---------------------------------------------------------------- 可用性

#[test]
fn availability_verbatim_from_server_bool() {
    let online = probe(DEVICE_ID, true, PUBLIC_KEY, VALID_FINGERPRINT).expect("合法视图必须通过");
    assert_eq!(online.availability(), Availability::Online);
    assert!(online.is_usable());

    // online=false 是服务端成功响应（设备存在但心跳超时），不是解析错误。
    let offline =
        probe(DEVICE_ID, false, PUBLIC_KEY, VALID_FINGERPRINT).expect("online=false 合法");
    assert_eq!(offline.availability(), Availability::Offline);
    assert!(!offline.is_usable());
}

#[test]
fn availability_usability_and_distinction() {
    assert!(Availability::Online.is_usable());
    assert!(!Availability::Offline.is_usable());
    assert!(!Availability::Unknown.is_usable());
    // Offline 与 Unknown 对「能不能用」等价，但枚举本身可区分、不得混为一谈。
    assert_ne!(Availability::Offline, Availability::Unknown);
    assert_eq!(Availability::Offline.stable_name(), "Offline");
    assert_eq!(Availability::Unknown.stable_name(), "Unknown");
    // 读不到权威答案 ⇒ Unknown（fail-closed），绝不降级成 Online。
    assert_eq!(
        Availability::from_server_answer(None),
        Availability::Unknown
    );
    assert!(!Availability::from_server_answer(None).is_usable());
    assert_eq!(
        Availability::from_server_answer(Some(true)),
        Availability::Online
    );
    assert_eq!(
        Availability::from_server_answer(Some(false)),
        Availability::Offline
    );
}

// ---------------------------------------------------------------- 构造与校验

#[test]
fn device_view_constructs_and_preserves_fields() {
    let view = DeviceView::new(
        DEVICE_ID,
        true,
        PUBLIC_KEY,
        VALID_FINGERPRINT,
        1_700_000_000_000,
    )
    .expect("合法 DeviceView 必须通过");
    assert_eq!(view.device_id, DEVICE_ID);
    assert!(view.online);
    assert_eq!(view.public_key, PUBLIC_KEY);
    assert_eq!(view.public_key_fingerprint, VALID_FINGERPRINT);
    assert_eq!(view.last_seen_at_ms, 1_700_000_000_000);
}

#[test]
fn peer_view_constructs_and_preserves_nat_candidates() {
    let cand = NatCandidate::new("host", "192.0.2.1:3478", 100).unwrap();
    let peer = PeerView::new(
        DEVICE_ID,
        false,
        PUBLIC_KEY,
        VALID_FINGERPRINT,
        0,
        vec![cand.clone()],
    )
    .expect("合法 PeerView 必须通过");
    assert!(!peer.online);
    assert_eq!(peer.nat_candidates, vec![cand]);
    assert_eq!(peer.availability(), Availability::Offline);
    assert!(!peer.is_usable());
}

#[test]
fn peer_view_nat_candidates_must_pass_validation() {
    // 候选类型复用 NatCandidate ⇒ 列表里每一项都必然过了 new 的校验。
    assert!(NatCandidate::new("bogus", "192.0.2.1:3478", 0).is_err());
}

#[test]
fn invalid_device_id_is_rejected() {
    for input in ["", "   ", "dev/1", &"a".repeat(129)] {
        let error = probe(input, true, PUBLIC_KEY, VALID_FINGERPRINT).unwrap_err();
        assert_eq!(error, CoordViewError::InvalidDeviceId, "{input:?}");
    }
}

#[test]
fn invalid_public_key_is_rejected() {
    for input in [&"A".repeat(15), &"A".repeat(513), "AAAAAAAAAAAAAAAA!"] {
        let error = probe(DEVICE_ID, true, input, VALID_FINGERPRINT).unwrap_err();
        assert_eq!(error, CoordViewError::InvalidPublicKey, "{input:?}");
    }
}

#[test]
fn invalid_fingerprint_length_is_rejected() {
    for fingerprint in ["abcdef012345678", "abcdef0123456789a"] {
        let error = probe(DEVICE_ID, true, PUBLIC_KEY, fingerprint).unwrap_err();
        assert_eq!(error, CoordViewError::InvalidFingerprint, "{fingerprint:?}");
    }
}

#[test]
fn invalid_fingerprint_uppercase_is_rejected() {
    // ⛔ 必须是小写十六进制。
    let error = probe(DEVICE_ID, true, PUBLIC_KEY, "ABCDEF0123456789").unwrap_err();
    assert_eq!(error, CoordViewError::InvalidFingerprint);
}

#[test]
fn invalid_fingerprint_non_hex_is_rejected() {
    for fingerprint in ["abcdef012345678g", "abcdef01-3456789", "abcdef01 4567890"] {
        let error = probe(DEVICE_ID, true, PUBLIC_KEY, fingerprint).unwrap_err();
        assert_eq!(error, CoordViewError::InvalidFingerprint, "{fingerprint:?}");
    }
}

#[test]
fn valid_fingerprint_is_format_checked_only() {
    // 指纹只校验格式；⛔ 不与公钥交叉验证、⛔ 不重算（权威值是完整公钥）。
    let view = probe(DEVICE_ID, true, PUBLIC_KEY, VALID_FINGERPRINT).expect("合法指纹必须通过");
    assert_eq!(view.public_key_fingerprint, VALID_FINGERPRINT);
    assert_eq!(view.public_key, PUBLIC_KEY);
}

// ---------------------------------------------------------------- 错误契约

#[test]
fn error_server_codes_match_server_contract() {
    let codes = [50100, 50102, 50102, 50101, 50117, 50004];
    for (variant, code) in ALL_ERRORS.iter().zip(codes) {
        assert_eq!(variant.server_code(), code, "{}", variant.stable_name());
    }
}

#[test]
fn error_stable_names_match_server_contract() {
    for (variant, name) in ALL_ERRORS.iter().zip(ERROR_NAMES) {
        assert_eq!(variant.stable_name(), name);
    }
}

#[test]
fn errors_never_carry_input_bytes() {
    let secret_id = "device/secret?token#fragment%2F";
    let secret_key = "SECRETKEYBYTES!!!!!!!!!!!!!!!!";
    let errors = [
        (CoordViewError::InvalidDeviceId, secret_id),
        (CoordViewError::InvalidPublicKey, secret_key),
    ];
    for (error, fragment) in errors {
        let debug = format!("{error:?}");
        let display = error.to_string();
        for text in [debug.as_str(), display.as_str()] {
            assert!(!text.contains(fragment), "错误值不得携带输入：{text}");
            assert!(!text.contains("SECRET"), "错误值不得携带输入：{text}");
        }
    }
    // 变体无字段 ⇒ 错误本身不可能装下长度/计数器等元数据。
    assert_eq!(std::mem::size_of::<CoordViewError>(), 1);
}

// ---------------------------------------------------------------- FNV 黄金值

#[test]
fn fnv1a64_golden_values_match_server_contract() {
    // 与服务端 hash 实现里的三个黄金值逐字相同。
    assert_eq!(fnv1a64_hex(b""), "cbf29ce484222325");
    assert_eq!(fnv1a64_hex(b"a"), "af63dc4c8601ec8c");
    assert_eq!(fnv1a64_hex(b"foobar"), "85944171f73967e8");
    // 额外黄金值：公钥 `0123456789abcdef0123456789abcdef` 的指纹。
    assert_eq!(
        fnv1a64_hex(b"0123456789abcdef0123456789abcdef"),
        "01527c9731f0ff55"
    );
    // 移动端复用 device.rs 的 fnv1a64_hex，与服务端同一算法同一常量。
    let key = PublicKey::parse("AAAAAAAAAAAAAAAA").unwrap();
    assert_eq!(key.fingerprint(), fnv1a64_hex(key.as_str().as_bytes()));
}

#[test]
fn source_has_no_io_threads_or_local_clock() {
    for forbidden in [
        "std::net",
        "std::fs",
        "std::thread",
        "Instant::now",
        "SystemTime",
    ] {
        assert!(!SOURCE.contains(forbidden), "门面层不得出现 {forbidden}");
    }
}

// ---------------------------------------------------------------- 跨端向量

/// 按向量用例构造设备视图。
fn case_view(case: &Json, online: bool) -> Result<DeviceView, CoordViewError> {
    DeviceView::new(
        field(case, "input"),
        online,
        field(case, "public_key"),
        field(case, "fingerprint"),
        case.field("last_seen_at_ms").as_u64() as i64,
    )
}

/// 按向量用例的 `expect_kind` 触发对应校验失败。
fn case_error(case: &Json) -> Result<DeviceView, CoordViewError> {
    let input = field(case, "input");
    match field(case, "expect_kind") {
        "device_id" => probe(input, true, PUBLIC_KEY, VALID_FINGERPRINT),
        "public_key" => probe(DEVICE_ID, true, input, VALID_FINGERPRINT),
        "fingerprint" => probe(DEVICE_ID, true, PUBLIC_KEY, input),
        other => panic!("未知 error kind {other}"),
    }
}

#[test]
fn vectors_json_is_canonical() {
    let rendered = json::render(&json::parse(VECTORS_JSON));
    assert_eq!(rendered, VECTORS_JSON, "向量原文必须是规范化 JSON");
}

#[test]
fn vectors_are_complete_and_match_implementation() {
    let root = json::parse(VECTORS_JSON);
    assert_eq!(root.field("format").as_str(), "lengsuo-coord-view-vectors");
    assert_eq!(root.field("version").as_u64(), 1);

    // FNV 黄金值（向量里 4 条，与实现逐条对拍）。
    let goldens = root.field("fnv1a64_golden").as_arr();
    assert_eq!(goldens.len(), 4, "FNV 黄金值必须为 4 条");
    for golden in goldens {
        let bytes = vectors::from_hex(golden.field("input_hex").as_str());
        assert_eq!(fnv1a64_hex(&bytes), golden.field("expect_hex").as_str());
    }

    let cases = root.field("cases").as_arr();
    assert!(cases.len() >= 16, "向量用例不得少于 16 条");
    let mut names: Vec<&str> = cases.iter().map(|c| field(c, "name")).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), cases.len(), "用例名不得重复");

    for case in cases {
        let name = field(case, "name");
        let kind = field(case, "kind");
        let valid = as_bool(case.field("expect_valid"));
        let code = case.field("expect_code").as_u64() as u32;
        let stable_name = field(case, "expect_stable_name");
        // 字段齐全性：缺字段即 panic。
        field(case, "input");
        case.field("expect_online");
        case.field("expect_usable");
        assert_eq!(valid, code == 0, "用例 {name}：code 矛盾");
        assert_eq!(valid, stable_name.is_empty(), "用例 {name}：名矛盾");

        let result = match kind {
            "device_view" => case_view(case, as_bool(case.field("expect_online"))),
            "error" => case_error(case),
            "server_error" => {
                let error = error_by_name(field(case, "input"));
                assert!(!valid, "用例 {name}：服务端错误必须按失败处理");
                assert_eq!(error.server_code(), code, "用例 {name}：码不符");
                assert_eq!(error.stable_name(), stable_name, "用例 {name}：名不符");
                continue;
            }
            other => panic!("未知向量 kind {other}（用例 {name}）"),
        };
        match result {
            Ok(view) => {
                assert!(valid, "用例 {name}：期望拒绝却通过");
                assert_eq!(stable_name, "", "用例 {name}：合法输入不写名");
                let online = as_bool(case.field("expect_online"));
                let availability = if online {
                    Availability::Online
                } else {
                    Availability::Offline
                };
                assert_eq!(
                    view.availability(),
                    availability,
                    "用例 {name}：在线状态不符"
                );
                assert_eq!(
                    view.is_usable(),
                    as_bool(case.field("expect_usable")),
                    "用例 {name}：可用性不符"
                );
            }
            Err(error) => {
                assert!(!valid, "用例 {name}：期望通过却拒绝");
                assert_eq!(error.server_code(), code, "用例 {name}：码不符");
                assert_eq!(error.stable_name(), stable_name, "用例 {name}：名不符");
            }
        }
    }
}
