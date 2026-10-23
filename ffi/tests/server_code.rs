//! 服务端 5 位错误码集中登记表的机器可判定用例。
//!
//! 这一组用例守护三件事：
//! 1. 表**完整且不重复**（20 条，码与 slug 两两不等）；
//! 2. 表**与契约逐字一致**（slug / HTTP，尤其「契约没给 HTTP 就不许发明」）；
//! 3. **未知码 fail-closed**，且既有模块在用的码全部落在本表（防止两套表分叉）。

use lengsuo_mobile_ffi::core::{CodeClass, ServerCode};

/// 组 1：表长度恒为 20。
#[test]
fn all_len_is_20() {
    assert_eq!(ServerCode::ALL.len(), 20);
}

/// 组 1b：`ALL` 的顺序与契约码表逐行一致（供绑定层按稳定顺序遍历）。
#[test]
fn all_order_matches_contract() {
    let codes: Vec<i32> = ServerCode::ALL.iter().map(|c| c.code()).collect();
    assert_eq!(
        codes,
        vec![
            10002, 10009, 10404, 50004, 50100, 50101, 50102, 50103, 50104, 50105, 50110, 50111,
            50112, 50113, 50114, 50115, 50116, 50117, 50118, 50119,
        ]
    );
}

/// 组 2：码 ↔ 变体双向一致（每个变体 `from_code(code)` 都回到自身）。
#[test]
fn round_trip() {
    for code in ServerCode::ALL {
        assert_eq!(
            ServerCode::from_code(code.code()),
            Some(code),
            "码 {} 无法回到变体 {:?}",
            code.code(),
            code
        );
    }
}

/// 组 3a：20 个码两两不等。
#[test]
fn codes_unique() {
    let mut codes: Vec<i32> = ServerCode::ALL.iter().map(|c| c.code()).collect();
    codes.sort_unstable();
    codes.dedup();
    assert_eq!(codes.len(), 20, "存在重复码：{codes:?}");
}

/// 组 3b：20 个 slug 两两不等。
#[test]
fn slugs_unique() {
    let mut slugs: Vec<&'static str> = ServerCode::ALL.iter().map(|c| c.slug()).collect();
    slugs.sort_unstable();
    slugs.dedup();
    assert_eq!(slugs.len(), 20, "存在重复 slug：{slugs:?}");
}

/// 组 4：`code → slug` 与契约逐字一致。
///
/// 20 条独立断言（不循环），失败时能直接指出是哪一条码错了。
#[test]
fn slug_matches_contract() {
    assert_eq!(ServerCode::InvalidRequest.slug(), "invalid_request");
    assert_eq!(
        ServerCode::IdempotencyKeyConflict.slug(),
        "idempotency_key_conflict"
    );
    assert_eq!(ServerCode::NotFound.slug(), "not_found");
    assert_eq!(ServerCode::DeviceOffline.slug(), "device_offline");
    assert_eq!(ServerCode::InvalidDeviceId.slug(), "invalid_device_id");
    assert_eq!(ServerCode::UnknownDevice.slug(), "unknown_device");
    assert_eq!(ServerCode::InvalidPublicKey.slug(), "invalid_public_key");
    assert_eq!(ServerCode::InvalidOwner.slug(), "invalid_owner");
    assert_eq!(ServerCode::InvalidCandidate.slug(), "invalid_candidate");
    assert_eq!(
        ServerCode::DeviceOwnershipConflict.slug(),
        "device_ownership_conflict"
    );
    assert_eq!(ServerCode::MissingApiKey.slug(), "missing_api_key");
    assert_eq!(ServerCode::InvalidApiKey.slug(), "invalid_api_key");
    assert_eq!(
        ServerCode::MissingBearerToken.slug(),
        "missing_bearer_token"
    );
    assert_eq!(
        ServerCode::InvalidBearerToken.slug(),
        "invalid_bearer_token"
    );
    assert_eq!(ServerCode::BearerExpired.slug(), "bearer_expired");
    assert_eq!(ServerCode::TenantMismatch.slug(), "tenant_mismatch");
    assert_eq!(ServerCode::PermissionDenied.slug(), "permission_denied");
    assert_eq!(ServerCode::BindingMismatch.slug(), "binding_mismatch");
    assert_eq!(ServerCode::StoreUnavailable.slug(), "store_unavailable");
    assert_eq!(ServerCode::InvalidCredential.slug(), "invalid_credential");
}

/// 组 5：HTTP 状态**只在契约明文给出时**才给出。
///
/// 18 条断言命中上表所列值；`10009` / `50004` 契约未给 ⇒ 必须是 `None`
/// （⛔ 这条就是「禁止发明 HTTP 状态」的机器闸门）。
#[test]
fn http_status_only_where_frozen() {
    assert_eq!(ServerCode::InvalidRequest.http_status(), Some(400));
    assert_eq!(ServerCode::NotFound.http_status(), Some(404));
    assert_eq!(ServerCode::InvalidDeviceId.http_status(), Some(400));
    assert_eq!(ServerCode::UnknownDevice.http_status(), Some(404));
    assert_eq!(ServerCode::InvalidPublicKey.http_status(), Some(400));
    assert_eq!(ServerCode::InvalidOwner.http_status(), Some(400));
    assert_eq!(ServerCode::InvalidCandidate.http_status(), Some(400));
    assert_eq!(ServerCode::DeviceOwnershipConflict.http_status(), Some(409));
    assert_eq!(ServerCode::MissingApiKey.http_status(), Some(401));
    assert_eq!(ServerCode::InvalidApiKey.http_status(), Some(401));
    assert_eq!(ServerCode::MissingBearerToken.http_status(), Some(401));
    assert_eq!(ServerCode::InvalidBearerToken.http_status(), Some(401));
    assert_eq!(ServerCode::BearerExpired.http_status(), Some(401));
    assert_eq!(ServerCode::TenantMismatch.http_status(), Some(403));
    assert_eq!(ServerCode::PermissionDenied.http_status(), Some(403));
    assert_eq!(ServerCode::BindingMismatch.http_status(), Some(403));
    assert_eq!(ServerCode::StoreUnavailable.http_status(), Some(503));
    assert_eq!(ServerCode::InvalidCredential.http_status(), Some(401));

    // 契约未给 HTTP ⇒ 不许猜，必须是 None。
    assert_eq!(ServerCode::IdempotencyKeyConflict.http_status(), None);
    assert_eq!(ServerCode::DeviceOffline.http_status(), None);
}

/// 组 6：未知码 fail-closed。
///
/// `from_code` 必须 `None`，且 `CodeClass::of(..).must_fail()` 必须 `true`；
/// 同时不得出现在 `ALL` 里（⛔ 没有 `Other` / `Unknown` 变体）。
#[test]
fn unknown_codes_fail_closed() {
    let unknown = [
        0,
        -1,
        10003,
        10008,
        50001,
        50106,
        50109,
        60001,
        99999,
        i32::MAX,
    ];
    for code in unknown {
        assert_eq!(ServerCode::from_code(code), None, "码 {code} 不应进表");
        assert_eq!(
            CodeClass::of(code),
            CodeClass::Unknown(code),
            "码 {code} 应分类为 Unknown"
        );
        assert!(CodeClass::of(code).must_fail(), "码 {code} 必须按失败处理");
    }
}

/// 组 7：既有模块已在用的码全部落在本表（防止两套表分叉）。
///
/// 覆盖 `device.rs` 的 `SERVER_ERROR_CODES`、`coord_view.rs` 的落码、
/// `unlock.rs` 的 `UnlockServerError`（三处既有码表的并集）。
#[test]
fn existing_modules_covered() {
    assert!(ServerCode::from_code(50100).is_some());
    assert!(ServerCode::from_code(50102).is_some());
    assert!(ServerCode::from_code(50104).is_some());
    assert!(ServerCode::from_code(50105).is_some());
    assert!(ServerCode::from_code(50101).is_some());
    assert!(ServerCode::from_code(50117).is_some());
    assert!(ServerCode::from_code(50004).is_some());
    assert!(ServerCode::from_code(50116).is_some());
    assert!(ServerCode::from_code(10002).is_some());
    assert!(ServerCode::from_code(10404).is_some());
}

/// 组 8：已知码同样 `must_fail() == true` —— 成功不走错误码。
#[test]
fn known_codes_also_fail() {
    for code in ServerCode::ALL {
        assert!(
            CodeClass::Known(code).must_fail(),
            "已知错误码 {:?} 也必须按失败处理",
            code
        );
    }
}
