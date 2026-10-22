//! 移动端「凭据解锁客户端侧模型」的**跨端契约测试**。
//!
//! 规格逐字对齐服务端契约 `docs/server-contract/04-fingerprint.md` §7.6
//! （`POST /client/secrets/unlock`）：请求体四项且仅四项、成功响应形状恒定、
//! 落码表 60006 / 60007 / 10002 / 50100 / 50116 / 10404。
//! 本仓独立实现；两端一致性靠 `tests/vectors/unlock_v1.json` 这份跨端向量，
//! Swift / Kotlin 侧将来读同一份文件逐条对拍、不靠真机。
//!
//! 覆盖：`credential_id` 长度 31/32/33 与大小写/非 hex/空串；`secret_id` /
//! `store_id` 空串与仅空白；`device_id` 复用既有 `DeviceId` 规则；未知字段
//! `tenant_id` / `user_id` / `aad`；`secret_hex` 大写/奇数长度/空串/非 hex/
//! `0x` 前缀；`UnlockedSecret` 只取一次；码映射双向对拍 + 未知码 `None`；
//! 非 0 码带 secret 必须拒；错误不回显输入。
//!
//! 用例只做纯函数断言：不开网络、不读文件系统（向量由 `include_str!` 内嵌，
//! 与既有 `tests/coord_view.rs` 同款）。

mod vectors;

use lengsuo_mobile_ffi::core::unlock::{
    UnlockModelError, UnlockRequest, UnlockResponse, UnlockServerError, CREDENTIAL_ID_HEX_LEN,
    UNLOCK_PATH,
};
use vectors::json::{self, Json};

/// 内嵌的跨端向量原文（与磁盘文件逐字节相同）。
const VECTORS_JSON: &str = include_str!("vectors/unlock_v1.json");

/// 门面层源码（用于「不做 IO / 不取本地时钟 / 无 unwrap」的静态断言）。
const SOURCE: &str = include_str!("../src/core/unlock.rs");

/// 合法请求的固定样本（用于错误隔离）。
const CRED: &str = "00112233445566778899aabbccddeeff";
const SECRET_ID: &str = "secret-1";
const STORE_ID: &str = "store-1";
const DEVICE_ID: &str = "dev0101";

/// 构造请求（成功 / 失败由调用方断言）。
fn request(
    credential_id: &str,
    secret_id: &str,
    store_id: &str,
    device_id: &str,
) -> Result<UnlockRequest, UnlockModelError> {
    UnlockRequest::new(credential_id, secret_id, store_id, device_id)
}

/// 断言请求被拒绝并返回错误。
fn request_err(
    credential_id: &str,
    secret_id: &str,
    store_id: &str,
    device_id: &str,
) -> UnlockModelError {
    request(credential_id, secret_id, store_id, device_id).expect_err("请求必须被拒绝")
}

/// 断言响应被拒绝并返回错误（`UnlockResponse` 刻意不实现 `Debug`）。
fn response_err(code: i32, msg: &str, secret_hex: &str) -> UnlockModelError {
    match UnlockResponse::from_parts(code, msg, secret_hex) {
        Ok(_) => panic!("响应必须被拒绝"),
        Err(error) => error,
    }
}

/// 断言「secret 列表」形状的响应被拒绝并返回错误。
fn list_err(code: i32, msg: &str, secrets: &[&str]) -> UnlockModelError {
    match UnlockResponse::from_secret_list(code, msg, secrets) {
        Ok(_) => panic!("响应必须被拒绝"),
        Err(error) => error,
    }
}

// ---------------------------------------------------------------- 请求体

#[test]
fn credential_id_requires_32_hex_and_canonicalizes_to_lowercase() {
    let lower = "00112233445566778899aabbccddeeff";
    assert_eq!(lower.len(), CREDENTIAL_ID_HEX_LEN);

    let parsed = request(lower, SECRET_ID, STORE_ID, DEVICE_ID).expect("合法请求必须通过");
    assert_eq!(parsed.credential_id(), lower);

    // 入参大小写皆可（契约 §7.6），**规范形态是小写**。
    let upper = "00112233445566778899AABBCCDDEEFF";
    let parsed = request(upper, SECRET_ID, STORE_ID, DEVICE_ID).expect("大写 hex 合法");
    assert_eq!(parsed.credential_id(), lower, "规范形态必须是小写");

    let mixed = "00112233445566778899aAbBcCdDeEfF";
    let parsed = request(mixed, SECRET_ID, STORE_ID, DEVICE_ID).expect("混合大小写合法");
    assert_eq!(parsed.credential_id(), lower, "规范形态必须是小写");
}

#[test]
fn credential_id_rejects_wrong_length_and_non_hex() {
    let too_short = "00112233445566778899aabbccddeef";
    let too_long = "00112233445566778899aabbccddeeff0";
    assert_eq!(too_short.len(), 31);
    assert_eq!(too_long.len(), 33);

    let mut rejected: Vec<&str> = vec![
        too_short,
        too_long,
        "",
        " 0112233445566778899aabbccddeeff",
        "00112233445566778899aabbccddeefg",
    ];
    let non_ascii = "设备设备设备设备设备设备设备设备";
    rejected.push(non_ascii);

    for input in rejected {
        let error = request_err(input, SECRET_ID, STORE_ID, DEVICE_ID);
        assert_eq!(error, UnlockModelError::InvalidCredentialId, "{input:?}");
        assert_eq!(error.stable_name(), "InvalidCredentialId");
        assert_eq!(error.contract_code(), 10002, "{input:?}");
        assert_eq!(error.to_server_error(), UnlockServerError::InvalidRequest);
    }
}

#[test]
fn secret_and_store_ids_must_be_non_empty_and_non_whitespace() {
    for input in ["", " ", "   ", "\t", "\n", " \t\r\n "] {
        let error = request_err(CRED, input, STORE_ID, DEVICE_ID);
        assert_eq!(error, UnlockModelError::InvalidSecretId, "{input:?}");
        assert_eq!(error.contract_code(), 10002);
        assert_eq!(error.to_server_error(), UnlockServerError::InvalidRequest);

        let error = request_err(CRED, SECRET_ID, input, DEVICE_ID);
        assert_eq!(error, UnlockModelError::InvalidStoreId, "{input:?}");
        assert_eq!(error.contract_code(), 10002);
        assert_eq!(error.to_server_error(), UnlockServerError::InvalidRequest);
    }

    // 规则是「非空」而不是「trim 后非空」：非空 id 原样保留、不 trim。
    let padded = " secret-1 ";
    let parsed = request(CRED, padded, STORE_ID, DEVICE_ID).expect("非空 id 合法");
    assert_eq!(parsed.secret_id(), padded);
}

#[test]
fn device_id_reuses_device_model_validation() {
    let too_long = "a".repeat(129);
    let too_long_max = "a".repeat(128);
    for input in [
        "",
        "   ",
        "dev/1",
        "dev?1",
        "dev#1",
        "dev%2F",
        "设备-01",
        too_long.as_str(),
    ] {
        let error = request_err(CRED, SECRET_ID, STORE_ID, input);
        assert_eq!(error, UnlockModelError::InvalidDeviceId, "{input:?}");
        assert_eq!(error.stable_name(), "InvalidDeviceId");
        assert_eq!(error.contract_code(), 50100, "{input:?}");
        assert_eq!(error.to_server_error(), UnlockServerError::InvalidDeviceId);
    }

    // 设备 ID 的字符集与 128 字节上限来自既有 `DeviceId`，⛔ 不另写一套。
    assert!(request(CRED, SECRET_ID, STORE_ID, too_long_max.as_str()).is_ok());
    assert!(request(CRED, SECRET_ID, STORE_ID, "a-b_c.d:e").is_ok());
}

#[test]
fn unknown_fields_are_rejected() {
    for field_name in ["tenant_id", "user_id", "aad", "passkey_id", "role", ""] {
        let fields = [
            ("credential_id", CRED),
            ("secret_id", SECRET_ID),
            ("store_id", STORE_ID),
            ("device_id", DEVICE_ID),
            (field_name, "x"),
        ];
        let error = UnlockRequest::from_fields(&fields).expect_err("未知字段必须拒");
        assert_eq!(error, UnlockModelError::UnknownField, "{field_name:?}");
        assert_eq!(error.stable_name(), "UnknownField");
        assert_eq!(error.contract_code(), 10002);
        assert_eq!(error.to_server_error(), UnlockServerError::InvalidRequest);
    }
}

#[test]
fn request_fields_can_arrive_in_any_order() {
    assert_eq!(
        UnlockRequest::FIELD_NAMES,
        ["credential_id", "secret_id", "store_id", "device_id"]
    );
    let fields = [
        ("device_id", DEVICE_ID),
        ("store_id", STORE_ID),
        ("secret_id", SECRET_ID),
        ("credential_id", CRED),
    ];
    let parsed = UnlockRequest::from_fields(&fields).expect("四字段齐全必须通过");
    assert_eq!(parsed.credential_id(), CRED);
    assert_eq!(parsed.secret_id(), SECRET_ID);
    assert_eq!(parsed.store_id(), STORE_ID);
    assert_eq!(parsed.device_id(), DEVICE_ID);
}

#[test]
fn duplicate_or_missing_fields_are_rejected() {
    // 重复字段不是契约里的形状 ⇒ fail-closed（按 UnknownField 归口，10002）。
    let duplicated = [
        ("credential_id", CRED),
        ("credential_id", CRED),
        ("secret_id", SECRET_ID),
        ("store_id", STORE_ID),
        ("device_id", DEVICE_ID),
    ];
    assert_eq!(
        UnlockRequest::from_fields(&duplicated).expect_err("重复字段必须拒"),
        UnlockModelError::UnknownField
    );
    // 缺字段按空值落到该字段自己的校验上。
    assert_eq!(
        UnlockRequest::from_fields(&[]).expect_err("缺字段必须拒"),
        UnlockModelError::InvalidCredentialId
    );
}

// ---------------------------------------------------------------- 成功响应

#[test]
fn success_response_yields_exactly_one_secret() {
    let response = UnlockResponse::from_parts(0, "ok", "deadbeef").expect("合法成功响应");
    assert_eq!(response.len(), 4);
    assert!(!response.is_empty());

    let secret = response.into_secret();
    assert_eq!(secret.len(), 4);
    assert!(!secret.is_empty());
    assert_eq!(secret.into_bytes(), vec![0xde, 0xad, 0xbe, 0xef]);
}

#[test]
fn unlocked_secret_can_only_be_taken_once() {
    // 编译期保证：字段私有 + `into_bytes(self)`（消费 self）是**唯一**取值路径；
    // `UnlockedSecret` 刻意不实现 `Debug` / `Display` / `Clone` / `Serialize`。
    // 运行期只断言长度与字节内容。
    let secret = UnlockResponse::from_secret_list(0, "ok", &["00ff10a5"])
        .expect("合法成功响应")
        .into_secret();
    assert_eq!(secret.len(), 4);
    assert!(!secret.is_empty());
    assert_eq!(secret.into_bytes(), vec![0x00, 0xff, 0x10, 0xa5]);

    // 空明文不是合法成功响应（`EmptySecret`）；不存在「空 secret 兜底」。
    assert_eq!(
        response_err(0, "ok", ""),
        UnlockModelError::EmptySecret,
        "空 secret 必须拒"
    );
}

#[test]
fn secret_hex_rejects_bad_shapes() {
    for (input, expected) in [
        ("", UnlockModelError::EmptySecret),
        ("DEADBEEF", UnlockModelError::InvalidSecretHex),
        ("DeadBeef", UnlockModelError::InvalidSecretHex),
        ("abc", UnlockModelError::InvalidSecretHex),
        ("0xdeadbeef", UnlockModelError::InvalidSecretHex),
        ("deadbeez", UnlockModelError::InvalidSecretHex),
        ("dead beef", UnlockModelError::InvalidSecretHex),
        ("dead-beef", UnlockModelError::InvalidSecretHex),
    ] {
        let error = response_err(0, "ok", input);
        assert_eq!(error, expected, "{input:?}");
        assert_eq!(error.stable_name(), expected.stable_name(), "{input:?}");
        // 响应形状不可用 ⇒ 按「凭据引用不可用」606 归口（本卡报告已注明）。
        assert_eq!(error.contract_code(), 60006, "{input:?}");
        assert_eq!(
            error.to_server_error(),
            UnlockServerError::CredentialReferenceInvalid
        );
    }
}

#[test]
fn success_shape_requires_code_zero_and_msg_ok() {
    for msg in ["", "OK", "ok ", " ok", "success", "Ok"] {
        let error = response_err(0, msg, "deadbeef");
        assert_eq!(error, UnlockModelError::UnexpectedStatus, "{msg:?}");
        assert_eq!(error.stable_name(), "UnexpectedStatus");
    }
}

#[test]
fn nonzero_code_with_secret_in_data_is_rejected() {
    // ⚠️ `data` 里有 secret **不得**当成成功（fail-closed）。
    for (code, expected) in [
        (60006, UnlockServerError::CredentialReferenceInvalid),
        (60007, UnlockServerError::UnlockAuditBlocked),
        (10002, UnlockServerError::InvalidRequest),
        (50116, UnlockServerError::PermissionDenied),
        (10404, UnlockServerError::EndpointAbsent),
        (50100, UnlockServerError::InvalidDeviceId),
    ] {
        let error = response_err(code, "ok", "deadbeef");
        assert_eq!(error.stable_name(), "ServerRejected", "码 {code}");
        assert_eq!(error.contract_code(), code, "码 {code}");
        assert_eq!(error.to_server_error(), expected, "码 {code}");
    }

    // 未知码即使 `data` 里有 secret 也必须拒绝（⛔ 不建 Catch-all 成功分支）。
    let error = response_err(99999, "ok", "deadbeef");
    assert_eq!(error, UnlockModelError::UnexpectedStatus);
    assert_eq!(error.contract_code(), 10002);
    assert_eq!(error.to_server_error(), UnlockServerError::InvalidRequest);
}

#[test]
fn secret_count_must_be_exactly_one() {
    assert_eq!(
        list_err(0, "ok", &[]),
        UnlockModelError::EmptySecret,
        "零条 secret 必须拒"
    );
    let error = list_err(0, "ok", &["deadbeef", "cafe"]);
    assert_eq!(
        error,
        UnlockModelError::SecretNotSingle,
        "第二条 secret 必须拒"
    );
    assert_eq!(error.stable_name(), "SecretNotSingle");
    assert_eq!(error.contract_code(), 60006);

    let response = UnlockResponse::from_secret_list(0, "ok", &["deadbeef"]).expect("单条合法");
    assert_eq!(response.len(), 4);

    // 非 0 码优先于条数判定（响应不是成功 ⇒ 按服务端码归口）。
    let error = list_err(60006, "credential_reference_invalid", &["deadbeef", "cafe"]);
    assert_eq!(
        error.to_server_error(),
        UnlockServerError::CredentialReferenceInvalid
    );
}

// ---------------------------------------------------------------- 码映射

#[test]
fn server_error_reverse_mapping_is_bidirectional() {
    // 与服务端契约 §7.6 落码表逐条一致（顺序与 `UnlockServerError::ALL` 对齐）。
    let codes = [60006, 60007, 10002, 50116, 10404, 50100];
    let names = [
        "CredentialReferenceInvalid",
        "UnlockAuditBlocked",
        "InvalidRequest",
        "PermissionDenied",
        "EndpointAbsent",
        "InvalidDeviceId",
    ];
    let msgs = [
        "credential_reference_invalid",
        "unlock_audit_blocked",
        "invalid_request",
        "permission_denied",
        "not_found",
        "invalid_device_id",
    ];
    let statuses = [403, 503, 400, 403, 404, 400];

    assert_eq!(UnlockServerError::ALL.len(), codes.len());
    for (index, variant) in UnlockServerError::ALL.into_iter().enumerate() {
        assert_eq!(
            UnlockServerError::from_contract_code(codes[index]),
            Some(variant)
        );
        assert_eq!(variant.contract_code(), codes[index]);
        assert_eq!(variant.stable_name(), names[index]);
        assert_eq!(variant.msg(), msgs[index]);
        assert_eq!(variant.http_status(), statuses[index]);
    }
}

#[test]
fn unknown_contract_codes_are_fail_closed() {
    // 0 是成功码、不是服务端错误；其余是未登记码（含同域其它坐标码）。
    for code in [0, -1, 1, 404, 60005, 60008, 10009, 50101, 50117, 99999] {
        assert_eq!(
            UnlockServerError::from_contract_code(code),
            None,
            "码 {code} 必须 fail-closed"
        );
    }
}

#[test]
fn unlock_model_errors_map_to_server_errors() {
    let errors = [
        UnlockModelError::InvalidCredentialId,
        UnlockModelError::InvalidSecretId,
        UnlockModelError::InvalidStoreId,
        UnlockModelError::InvalidDeviceId,
        UnlockModelError::UnknownField,
        UnlockModelError::InvalidSecretHex,
        UnlockModelError::EmptySecret,
        UnlockModelError::SecretNotSingle,
        UnlockModelError::UnexpectedStatus,
        UnlockModelError::ServerRejected(UnlockServerError::UnlockAuditBlocked),
    ];
    let names = [
        "InvalidCredentialId",
        "InvalidSecretId",
        "InvalidStoreId",
        "InvalidDeviceId",
        "UnknownField",
        "InvalidSecretHex",
        "EmptySecret",
        "SecretNotSingle",
        "UnexpectedStatus",
        "ServerRejected",
    ];
    let codes = [
        10002, 10002, 10002, 50100, 10002, 60006, 60006, 60006, 10002, 60007,
    ];
    let server_errors = [
        UnlockServerError::InvalidRequest,
        UnlockServerError::InvalidRequest,
        UnlockServerError::InvalidRequest,
        UnlockServerError::InvalidDeviceId,
        UnlockServerError::InvalidRequest,
        UnlockServerError::CredentialReferenceInvalid,
        UnlockServerError::CredentialReferenceInvalid,
        UnlockServerError::CredentialReferenceInvalid,
        UnlockServerError::InvalidRequest,
        UnlockServerError::UnlockAuditBlocked,
    ];
    for (index, error) in errors.into_iter().enumerate() {
        assert_eq!(error.stable_name(), names[index]);
        assert_eq!(error.contract_code(), codes[index], "{}", names[index]);
        assert_eq!(
            error.to_server_error(),
            server_errors[index],
            "{}",
            names[index]
        );
    }
}

// ---------------------------------------------------------------- 卫生

#[test]
fn errors_never_echo_input_bytes() {
    let errors = [
        (
            request_err("SECRETCREDENTIAL", SECRET_ID, STORE_ID, DEVICE_ID),
            "SECRETCREDENTIAL",
        ),
        (
            request_err(CRED, SECRET_ID, STORE_ID, "SECRET/device"),
            "SECRET/device",
        ),
        (response_err(0, "ok", "DEADBEEF"), "DEADBEEF"),
        (response_err(0, "ok", "SECRETSECRET"), "SECRETSECRET"),
    ];
    for (error, fragment) in errors {
        let debug = format!("{error:?}");
        let display = error.to_string();
        for text in [debug.as_str(), display.as_str()] {
            assert!(!text.contains(fragment), "错误值不得携带输入：{text}");
            assert!(!text.contains("SECRET"), "错误值不得携带输入：{text}");
        }
    }
    // 无字段枚举 ⇒ 错误本身装不下长度 / 计数器等元数据。
    assert_eq!(std::mem::size_of::<UnlockServerError>(), 1);
}

#[test]
fn source_has_no_io_threads_clock_or_unwrap() {
    for forbidden in [
        "std::net",
        "std::fs",
        "std::thread",
        "std::time",
        "Instant::now",
        "SystemTime",
    ] {
        assert!(!SOURCE.contains(forbidden), "门面层不得出现 {forbidden}");
    }
    assert!(!SOURCE.contains(".unwrap()"), "生产路径不得出现 unwrap()");
    assert_eq!(UNLOCK_PATH, "/client/secrets/unlock");
}

// ---------------------------------------------------------------- 跨端向量

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

/// 取可选字符串字段（缺省返回 `default`）。
fn opt_str<'a>(case: &'a Json, key: &str, default: &'a str) -> &'a str {
    case.get(key).map_or(default, Json::as_str)
}

/// 取可选数字字段。
fn opt_u64(case: &Json, key: &str) -> Option<u64> {
    case.get(key).map(Json::as_u64)
}

/// 断言用例名不重复。
fn assert_unique_names(cases: &[Json], label: &str) {
    let mut names: Vec<&str> = cases.iter().map(|case| field(case, "name")).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), cases.len(), "{label} 用例名不得重复");
}

#[test]
fn vectors_json_is_canonical() {
    let rendered = json::render(&json::parse(VECTORS_JSON));
    assert_eq!(rendered, VECTORS_JSON, "向量原文必须是规范化 JSON");
}

#[test]
fn vectors_declare_endpoint_fields_and_codes() {
    let root = json::parse(VECTORS_JSON);
    assert_eq!(
        root.field("format").as_str(),
        "lengsuo-unlock-client-vectors"
    );
    assert_eq!(root.field("version").as_u64(), 1);
    assert_eq!(root.field("endpoint").field("path").as_str(), UNLOCK_PATH);
    assert_eq!(root.field("endpoint").field("method").as_str(), "POST");

    let codes = root.field("error_codes");
    for variant in UnlockServerError::ALL {
        let code = codes.field(variant.stable_name()).as_u64() as i32;
        assert_eq!(code, variant.contract_code(), "{}", variant.stable_name());
    }

    let fields = root.field("request_fields").as_arr();
    assert_eq!(fields.len(), UnlockRequest::FIELD_NAMES.len());
    for (item, expected) in fields.iter().zip(UnlockRequest::FIELD_NAMES) {
        assert_eq!(item.as_str(), expected);
    }

    let forbidden = root.field("forbidden_fields").as_arr();
    for expected in ["tenant_id", "user_id", "aad"] {
        assert!(
            forbidden.iter().any(|item| item.as_str() == expected),
            "禁止字段表缺 {expected}"
        );
    }
}

#[test]
fn vectors_request_cases_match_implementation() {
    let root = json::parse(VECTORS_JSON);
    let cases = root.field("request_cases").as_arr();
    assert!(cases.len() >= 16, "请求向量不得少于 16 条");
    assert_unique_names(cases, "request");

    for case in cases {
        let name = field(case, "name");
        let credential_id = opt_str(case, "credential_id", CRED);
        let secret_id = opt_str(case, "secret_id", SECRET_ID);
        let store_id = opt_str(case, "store_id", STORE_ID);
        let device_id = opt_str(case, "device_id", DEVICE_ID);
        let valid = as_bool(case.field("expect_valid"));
        let code = case.field("expect_code").as_u64() as i32;
        let stable_name = field(case, "expect_stable_name");
        assert_eq!(valid, code == 0, "用例 {name}：code 矛盾");
        assert_eq!(valid, stable_name.is_empty(), "用例 {name}：名矛盾");

        match request(credential_id, secret_id, store_id, device_id) {
            Ok(parsed) => {
                assert!(valid, "用例 {name}：期望拒绝却通过");
                let canonical = opt_str(case, "expect_canonical_credential_id", CRED);
                assert_eq!(parsed.credential_id(), canonical, "用例 {name}：规范化不符");
            }
            Err(error) => {
                assert!(!valid, "用例 {name}：期望通过却拒绝");
                assert_eq!(error.contract_code(), code, "用例 {name}：码不符");
                assert_eq!(error.stable_name(), stable_name, "用例 {name}：名不符");
            }
        }
    }
}

#[test]
fn vectors_unknown_field_cases_match_implementation() {
    let root = json::parse(VECTORS_JSON);
    let cases = root.field("unknown_field_cases").as_arr();
    assert!(cases.len() >= 3, "未知字段向量不得少于 3 条");
    assert_unique_names(cases, "unknown_field");

    for case in cases {
        let name = field(case, "name");
        let field_name = field(case, "field");
        let code = case.field("expect_code").as_u64() as i32;
        let stable_name = field(case, "expect_stable_name");
        let fields = [
            ("credential_id", CRED),
            ("secret_id", SECRET_ID),
            ("store_id", STORE_ID),
            ("device_id", DEVICE_ID),
            (field_name, "x"),
        ];
        let error = UnlockRequest::from_fields(&fields).expect_err("未知字段必须拒");
        assert_eq!(error.contract_code(), code, "用例 {name}：码不符");
        assert_eq!(error.stable_name(), stable_name, "用例 {name}：名不符");
    }
}

#[test]
fn vectors_response_cases_match_implementation() {
    let root = json::parse(VECTORS_JSON);
    let cases = root.field("response_cases").as_arr();
    assert!(cases.len() >= 12, "响应向量不得少于 12 条");
    assert_unique_names(cases, "response");

    for case in cases {
        let name = field(case, "name");
        let code = opt_u64(case, "code").map_or(0, |value| value as i32);
        let msg = opt_str(case, "msg", "ok");
        let expect_ok = as_bool(case.field("expect_ok"));
        let expect_code = opt_u64(case, "expect_code").map_or(0, |value| value as i32);
        let expect_name = opt_str(case, "expect_stable_name", "");

        let result = if let Some(secrets) = case.get("secrets") {
            let secrets: Vec<&str> = secrets.as_arr().iter().map(Json::as_str).collect();
            UnlockResponse::from_secret_list(code, msg, &secrets).map(|response| response.len())
        } else {
            let secret_hex = opt_str(case, "secret_hex", "");
            UnlockResponse::from_parts(code, msg, secret_hex).map(|response| response.len())
        };

        match result {
            Ok(len) => {
                assert!(expect_ok, "用例 {name}：期望拒绝却通过");
                assert_eq!(expect_name, "", "用例 {name}：合法响应不写名");
                assert_eq!(expect_code, 0, "用例 {name}：合法响应码必须为 0");
                let expected_len = case.field("expect_secret_len").as_u64() as usize;
                assert_eq!(len, expected_len, "用例 {name}：明文长度不符");
            }
            Err(error) => {
                assert!(!expect_ok, "用例 {name}：期望通过却拒绝");
                assert_eq!(error.contract_code(), expect_code, "用例 {name}：码不符");
                assert_eq!(error.stable_name(), expect_name, "用例 {name}：名不符");
            }
        }
    }
}

#[test]
fn vectors_server_error_cases_match_implementation() {
    let root = json::parse(VECTORS_JSON);
    let cases = root.field("server_error_cases").as_arr();
    assert!(cases.len() >= 7, "码映射向量不得少于 7 条");
    assert_unique_names(cases, "server_error");

    for case in cases {
        let name = field(case, "name");
        let code = case.field("code").as_u64() as i32;
        let known = as_bool(case.field("expect_known"));
        let mapped = UnlockServerError::from_contract_code(code);

        if known {
            let variant = mapped.unwrap_or_else(|| panic!("用例 {name}：码 {code} 必须已知"));
            assert_eq!(variant.stable_name(), field(case, "expect_variant"));
            assert_eq!(variant.contract_code(), code, "用例 {name}：码不符");
            assert_eq!(
                variant.msg(),
                field(case, "expect_msg"),
                "用例 {name}：msg 不符"
            );
            assert_eq!(
                variant.http_status(),
                case.field("expect_http").as_u64() as u16,
                "用例 {name}：HTTP 状态不符"
            );
        } else {
            assert!(mapped.is_none(), "用例 {name}：未知码必须返回 None");
            assert_eq!(
                field(case, "expect_variant"),
                "",
                "用例 {name}：未知码不写变体"
            );
        }
    }
}
