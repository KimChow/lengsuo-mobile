//! 服务端契约里**逐字冻结**的 5 位错误码集中登记表。
//!
//! # 为什么需要这张表
//!
//! 在引入本模块之前，码值散落在各业务模块里（`device.rs` 的
//! [`SERVER_ERROR_CODES`]、`coord_view.rs` 的落码方法、`unlock.rs` 的
//! `UnlockServerError`），各自只覆盖自己用到的那几个。客户端因此拿不到
//! 「这个码是什么意思」的权威映射，也没有「未知码怎么处理」的统一约定。
//!
//! 本模块是**唯一一张**全量登记表：码 ↔ 变体、稳定 slug、契约给出的 HTTP 状态，
//! 以及未知码的 fail-closed 判定。⛔ 其它模块已有的码表**不改**，但必须与本表一致
//! （`ffi/tests/server_code.rs` 的 `existing_modules_covered` 锁死这一点）。
//!
//! # 码表（20 行，逐字照抄契约）
//!
//! | 码 | 变体 | slug | HTTP | 来源 |
//! |---|---|---|---|---|
//! | 10002 | `InvalidRequest` | `invalid_request` | 400 | C2 |
//! | 10009 | `IdempotencyKeyConflict` | `idempotency_key_conflict` | 无（`None`） | C2 |
//! | 10404 | `NotFound` | `not_found` | 404 | C2 |
//! | 50004 | `DeviceOffline` | `device_offline` | 无（`None`） | C1 |
//! | 50100 | `InvalidDeviceId` | `invalid_device_id` | 400 | C1 |
//! | 50101 | `UnknownDevice` | `unknown_device` | 404 | C1 |
//! | 50102 | `InvalidPublicKey` | `invalid_public_key` | 400 | C1 |
//! | 50103 | `InvalidOwner` | `invalid_owner` | 400 | C1 |
//! | 50104 | `InvalidCandidate` | `invalid_candidate` | 400 | C1 |
//! | 50105 | `DeviceOwnershipConflict` | `device_ownership_conflict` | 409 | C1 |
//! | 50110 | `MissingApiKey` | `missing_api_key` | 401 | C1 |
//! | 50111 | `InvalidApiKey` | `invalid_api_key` | 401 | C1 |
//! | 50112 | `MissingBearerToken` | `missing_bearer_token` | 401 | C1 |
//! | 50113 | `InvalidBearerToken` | `invalid_bearer_token` | 401 | C1 |
//! | 50114 | `BearerExpired` | `bearer_expired` | 401 | C1 |
//! | 50115 | `TenantMismatch` | `tenant_mismatch` | 403 | C1 |
//! | 50116 | `PermissionDenied` | `permission_denied` | 403 | C1 |
//! | 50117 | `BindingMismatch` | `binding_mismatch` | 403 | C1 |
//! | 50118 | `StoreUnavailable` | `store_unavailable` | 503 | C1 |
//! | 50119 | `InvalidCredential` | `invalid_credential` | 401 | C1 |
//!
//! 来源缩写（服务端私有仓 `docs/server-contract/`）：
//!
//! - `C1` = `docs/server-contract/07-crossendpoint.md` §码表（第 132–148 行）；
//! - `C2` = `docs/server-contract/README.md` §二（第 30–35 行）；
//! - `C3` = `docs/server-contract/04-fingerprint.md`（第 154 行，交叉引用 10xxx 段）。
//!
//! 说明：契约把跨段通用码写在 `README.md` §二、设备/鉴权码写在
//! `07-crossendpoint.md` §码表，因此「来源」列按 10xxx / 5xxxx 分组标注；
//! 同一码在多个文档出现时以 `C1` 的表为准。
//!
//! # 不在本表（⛔ 不要顺手指望本表覆盖）
//!
//! - `60001`–`60005`（指纹域）与 `60006` / `60007`（凭据域，见 `unlock.rs`）；
//! - `10001` / `10008` / `50001` / `50002` / `50003` / `50005`：契约只给了中文短语、
//!   **没给 slug**，因此不进本表。
//!
//! # fail-closed
//!
//! 未知码**不猜语义**（不进 [`ServerCode`]），但处置与已知失败码一致：
//! [`CodeClass::must_fail`] 恒为 `true`。契约 `README.md` §二：
//! 「客户端收到未知错误码按失败处理」⇒ 任何非 0 码都不得被当成成功。
//!
//! # 纪律
//!
//! 只用 `core`：⛔ 无 IO、⛔ 无 `unsafe`、⛔ 无 serde/JSON、⛔ 无新增依赖。
//! `#[non_exhaustive]` ⛔ 不加 —— 要与服务端码表逐字对拍。

/// 服务端契约里逐字冻结的错误码。未知码不进本枚举（见 [`CodeClass`]）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ServerCode {
    /// `10002` 请求形状非法（未知字段 / 长度不符等）。
    InvalidRequest,
    /// `10009` 幂等键冲突：同一幂等键携带了不同请求。
    IdempotencyKeyConflict,
    /// `10404` 资源不存在（端点等同于不存在）。
    NotFound,
    /// `50004` 设备心跳超时 ⇒ 设备离线。
    DeviceOffline,
    /// `50100` 设备 ID 非法。
    InvalidDeviceId,
    /// `50101` 设备不存在。
    UnknownDevice,
    /// `50102` 公钥非法。
    InvalidPublicKey,
    /// `50103` 属主非法。
    InvalidOwner,
    /// `50104` NAT 候选非法。
    InvalidCandidate,
    /// `50105` 设备归属冲突（同一设备已属主他方）。
    DeviceOwnershipConflict,
    /// `50110` 缺少 API key。
    MissingApiKey,
    /// `50111` API key 非法。
    InvalidApiKey,
    /// `50112` 缺少 Bearer token。
    MissingBearerToken,
    /// `50113` Bearer token 非法。
    InvalidBearerToken,
    /// `50114` Bearer token 已过期。
    BearerExpired,
    /// `50115` 租户不匹配。
    TenantMismatch,
    /// `50116` 权限不足。
    PermissionDenied,
    /// `50117` 绑定不匹配。
    BindingMismatch,
    /// `50118` 存储不可用。
    StoreUnavailable,
    /// `50119` 凭据非法。
    InvalidCredential,
}

impl ServerCode {
    /// 全表（顺序与模块文档里的码表一致），供绑定层与测试遍历。
    pub const ALL: [ServerCode; 20] = [
        Self::InvalidRequest,
        Self::IdempotencyKeyConflict,
        Self::NotFound,
        Self::DeviceOffline,
        Self::InvalidDeviceId,
        Self::UnknownDevice,
        Self::InvalidPublicKey,
        Self::InvalidOwner,
        Self::InvalidCandidate,
        Self::DeviceOwnershipConflict,
        Self::MissingApiKey,
        Self::InvalidApiKey,
        Self::MissingBearerToken,
        Self::InvalidBearerToken,
        Self::BearerExpired,
        Self::TenantMismatch,
        Self::PermissionDenied,
        Self::BindingMismatch,
        Self::StoreUnavailable,
        Self::InvalidCredential,
    ];

    /// 契约码（整数）。反向见 [`ServerCode::from_code`]。
    #[must_use]
    pub const fn code(self) -> i32 {
        match self {
            Self::InvalidRequest => 10002,
            Self::IdempotencyKeyConflict => 10009,
            Self::NotFound => 10404,
            Self::DeviceOffline => 50004,
            Self::InvalidDeviceId => 50100,
            Self::UnknownDevice => 50101,
            Self::InvalidPublicKey => 50102,
            Self::InvalidOwner => 50103,
            Self::InvalidCandidate => 50104,
            Self::DeviceOwnershipConflict => 50105,
            Self::MissingApiKey => 50110,
            Self::InvalidApiKey => 50111,
            Self::MissingBearerToken => 50112,
            Self::InvalidBearerToken => 50113,
            Self::BearerExpired => 50114,
            Self::TenantMismatch => 50115,
            Self::PermissionDenied => 50116,
            Self::BindingMismatch => 50117,
            Self::StoreUnavailable => 50118,
            Self::InvalidCredential => 50119,
        }
    }

    /// 稳定 slug（跨端契约名，逐字小写下划线）。⛔ 不做本地化、不加前缀。
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::IdempotencyKeyConflict => "idempotency_key_conflict",
            Self::NotFound => "not_found",
            Self::DeviceOffline => "device_offline",
            Self::InvalidDeviceId => "invalid_device_id",
            Self::UnknownDevice => "unknown_device",
            Self::InvalidPublicKey => "invalid_public_key",
            Self::InvalidOwner => "invalid_owner",
            Self::InvalidCandidate => "invalid_candidate",
            Self::DeviceOwnershipConflict => "device_ownership_conflict",
            Self::MissingApiKey => "missing_api_key",
            Self::InvalidApiKey => "invalid_api_key",
            Self::MissingBearerToken => "missing_bearer_token",
            Self::InvalidBearerToken => "invalid_bearer_token",
            Self::BearerExpired => "bearer_expired",
            Self::TenantMismatch => "tenant_mismatch",
            Self::PermissionDenied => "permission_denied",
            Self::BindingMismatch => "binding_mismatch",
            Self::StoreUnavailable => "store_unavailable",
            Self::InvalidCredential => "invalid_credential",
        }
    }

    /// 契约明文给出的 HTTP 状态；**契约没给的返回 `None`**（不许猜）。
    #[must_use]
    pub const fn http_status(self) -> Option<u16> {
        match self {
            Self::InvalidRequest => Some(400),
            Self::IdempotencyKeyConflict => None,
            Self::NotFound => Some(404),
            Self::DeviceOffline => None,
            Self::InvalidDeviceId => Some(400),
            Self::UnknownDevice => Some(404),
            Self::InvalidPublicKey => Some(400),
            Self::InvalidOwner => Some(400),
            Self::InvalidCandidate => Some(400),
            Self::DeviceOwnershipConflict => Some(409),
            Self::MissingApiKey => Some(401),
            Self::InvalidApiKey => Some(401),
            Self::MissingBearerToken => Some(401),
            Self::InvalidBearerToken => Some(401),
            Self::BearerExpired => Some(401),
            Self::TenantMismatch => Some(403),
            Self::PermissionDenied => Some(403),
            Self::BindingMismatch => Some(403),
            Self::StoreUnavailable => Some(503),
            Self::InvalidCredential => Some(401),
        }
    }

    /// 码 → 变体。不在码表 ⇒ `None`（⛔ 不存在 `Other` / `Unknown` 变体）。
    #[must_use]
    pub const fn from_code(code: i32) -> Option<Self> {
        match code {
            10002 => Some(Self::InvalidRequest),
            10009 => Some(Self::IdempotencyKeyConflict),
            10404 => Some(Self::NotFound),
            50004 => Some(Self::DeviceOffline),
            50100 => Some(Self::InvalidDeviceId),
            50101 => Some(Self::UnknownDevice),
            50102 => Some(Self::InvalidPublicKey),
            50103 => Some(Self::InvalidOwner),
            50104 => Some(Self::InvalidCandidate),
            50105 => Some(Self::DeviceOwnershipConflict),
            50110 => Some(Self::MissingApiKey),
            50111 => Some(Self::InvalidApiKey),
            50112 => Some(Self::MissingBearerToken),
            50113 => Some(Self::InvalidBearerToken),
            50114 => Some(Self::BearerExpired),
            50115 => Some(Self::TenantMismatch),
            50116 => Some(Self::PermissionDenied),
            50117 => Some(Self::BindingMismatch),
            50118 => Some(Self::StoreUnavailable),
            50119 => Some(Self::InvalidCredential),
            _ => None,
        }
    }
}

/// 分类：已知码 / 未知码。未知码**不猜语义**，但处置与已知码一致 —— 一律按失败处理。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeClass {
    /// 落在 [`ServerCode`] 表内的码。
    Known(ServerCode),
    /// 不在表内的码。只保留原始数值，⛔ 不推断含义。
    Unknown(i32),
}

impl CodeClass {
    /// 按码分类，未知码原样保留。
    #[must_use]
    pub const fn of(code: i32) -> Self {
        match ServerCode::from_code(code) {
            Some(known) => Self::Known(known),
            None => Self::Unknown(code),
        }
    }

    /// 恒为 `true`。契约 `README.md` §二：「客户端收到未知错误码按失败处理」⇒
    /// 已知失败码与未知码**都不许被当成成功**。本函数存在的意义是把这条写进类型里，
    /// 供上层 fail-closed 断言使用。
    #[must_use]
    pub const fn must_fail(self) -> bool {
        true
    }
}
