//! 移动端「**凭据解锁客户端侧模型**」。
//!
//! # 这一层为什么存在
//!
//! 服务端协调面已完整落地 `POST /client/secrets/unlock`（明文**即用即弃**），
//! 而移动端门面层此前**完全没有**该端点的客户端模型：没有请求体形状校验、
//! 没有成功响应形状校验、没有「落码 → 语义」的反向映射。
//! 本模块把服务端契约 `docs/server-contract/04-fingerprint.md` §7.6 的
//! **请求体 / 成功响应 / 落码表**逐字复刻为纯 Rust 模型。
//!
//! ⛔ 不做 uniffi 导出、⛔ 不做 HTTP 客户端、⛔ 不做 JSON 序列化、⛔ 无 IO
//! （纪律同 [`super`]）：本模块只做纯函数校验，网络与编码由后续卡负责。
//!
//! # 契约（逐字，⛔ 不得改写）
//!
//! ## 请求体：四项且仅四项
//!
//! | 字段 | 规则 |
//! |---|---|
//! | `credential_id` | **32 位 hex**（16 字节）；入参大小写皆可，**规范形态是小写** |
//! | `secret_id` | 目标 secret 条目 id（**非空**） |
//! | `store_id` | 目标环境 id（**非空**） |
//! | `device_id` | 发起设备 id（复用 [`DeviceId`] 的字符集 / 长度规则） |
//!
//! ⛔ 请求体**不含** `tenant_id` / `user_id`（必须来自登录态）、**不含** `aad`
//! （AAD 由服务端注入）⇒ 出现这三个字段一律按「未知字段」拒（10002）。
//!
//! ## 成功响应
//!
//! HTTP 200、`code == 0`、`msg == "ok"`、`data` **恒为**
//! `{"secret": "<小写 hex>"}` ⇒ **只解一条、只回一条**：
//! ⛔ 不得出现数组、⛔ 不得出现第二个 secret、⛔ 不得出现明文的其它字段。
//!
//! ## 落码表（[`UnlockServerError`]）
//!
//! | 情形 | 码 | HTTP | `msg` |
//! |---|---|---|---|
//! | 凭据引用不可用（含解封失败 / 存储故障） | 60006 | 403 | `credential_reference_invalid` |
//! | 解锁被审计拦截 | 60007 | 503 | `unlock_audit_blocked` |
//! | 请求体形状非法 | 10002 | 400 | `invalid_request` |
//! | `device_id` 非法 | 50100 | 400 | `invalid_device_id` |
//! | 账号归属不符 | 50116 | 403 | `permission_denied` |
//! | 未接凭据域（端点等同于不存在） | 10404 | 404 | `not_found` |
//!
//! # fail-closed（没有兜底）
//!
//! 协调面不可达 / 响应不可解析 / 码不在已知集合 ⇒ 调用方**必须按失败处理**。
//! ⛔ 不得降级为「未解锁但继续」；⛔ 本模块不提供重试、缓存或降级开关；
//! ⛔ 非 0 码即使 `data` 里有 secret 也一律拒（[`UnlockResponse::from_parts`]）。
//!
//! # 即用即弃
//!
//! [`UnlockedSecret`] 字段私有，**只**能经 `into_bytes(self)` 取值（消费 `self`），
//! 且**不实现** `Debug` / `Display` / `Clone` / `Serialize` ⇒
//! 「只解一条、用完即弃」由**类型层面**钉死。
//!
//! # 错误不泄输入
//!
//! [`UnlockModelError`] / [`UnlockServerError`] 的 `Display` 只输出固定文本，
//! `Debug` 只输出变体名：错误里不可能携带 secret 明文、凭据 id 或任何输入片段。
//! 跨端按 [`UnlockModelError::stable_name`] 分支，⛔ 不解析 `Display` 文本。

use thiserror::Error;

use super::device::DeviceId;

/// `credential_id` 的**规范字符长度**：32 位 hex = 16 字节（服务端 `CREDENTIAL_ID_LEN`）。
pub const CREDENTIAL_ID_HEX_LEN: usize = 32;

/// 端点路径（服务端 `http::UNLOCK_PATH` 逐字同款）。
pub const UNLOCK_PATH: &str = "/client/secrets/unlock";

/// 成功响应的契约码。
pub const SUCCESS_CODE: i32 = 0;

/// 成功响应的契约 `msg`。
pub const SUCCESS_MSG: &str = "ok";

/// 服务端返回的**非 0 契约码** → 客户端语义。
///
/// 码值为服务端 `CoordError::code()` 与凭据域 60xxx 段的**逐条实测值**，
/// ⛔ 不自行编号、⛔ 不改号。未知码由
/// [`UnlockServerError::from_contract_code`] 返回 `None`（fail-closed）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Error)]
#[non_exhaustive]
pub enum UnlockServerError {
    /// **60006** 凭据引用不可用（凭据不存在 / 未授权到环境 / 设备未绑定 /
    /// secret 不存在 / 密钥材料缺失 / 解封失败 / 存储故障）。
    #[error("凭据引用不可用")]
    CredentialReferenceInvalid,

    /// **60007** 解锁被审计拦截（审计写不进去 ⇒ 本次解锁作废）。
    #[error("解锁被审计拦截")]
    UnlockAuditBlocked,

    /// **10002** 请求体形状非法（未知字段 / 长度不符）。
    #[error("请求体形状非法")]
    InvalidRequest,

    /// **50116** 账号归属不符（成员未被授权到该环境）。
    #[error("账号归属不符")]
    PermissionDenied,

    /// **10404** 未接凭据域 ⇒ 端点等同于不存在。
    #[error("端点不存在")]
    EndpointAbsent,

    /// **50100** `device_id` 非法。
    #[error("设备 ID 非法")]
    InvalidDeviceId,
}

impl UnlockServerError {
    /// 全部已知变体（顺序即跨端向量 `server_error_cases` 的对照顺序）。
    pub const ALL: [Self; 6] = [
        Self::CredentialReferenceInvalid,
        Self::UnlockAuditBlocked,
        Self::InvalidRequest,
        Self::PermissionDenied,
        Self::EndpointAbsent,
        Self::InvalidDeviceId,
    ];

    /// **反向映射**：契约码 → 变体。
    ///
    /// 未知码一律返回 `None` ⇒ 调用方按 **fail-closed** 处理：
    /// ⛔ 绝不把未知码当成功、⛔ 不建 Catch-all 成功分支。
    #[must_use]
    pub const fn from_contract_code(code: i32) -> Option<Self> {
        match code {
            60006 => Some(Self::CredentialReferenceInvalid),
            60007 => Some(Self::UnlockAuditBlocked),
            10002 => Some(Self::InvalidRequest),
            50116 => Some(Self::PermissionDenied),
            10404 => Some(Self::EndpointAbsent),
            50100 => Some(Self::InvalidDeviceId),
            _ => None,
        }
    }

    /// 对应的契约码（与 [`UnlockServerError::from_contract_code`] 双向一致）。
    #[must_use]
    pub const fn contract_code(self) -> i32 {
        match self {
            Self::CredentialReferenceInvalid => 60006,
            Self::UnlockAuditBlocked => 60007,
            Self::InvalidRequest => 10002,
            Self::PermissionDenied => 50116,
            Self::EndpointAbsent => 10404,
            Self::InvalidDeviceId => 50100,
        }
    }

    /// 稳定名字（**跨端契约**：一旦发布不再改动，跨端按它分支）。
    #[must_use]
    pub const fn stable_name(self) -> &'static str {
        match self {
            Self::CredentialReferenceInvalid => "CredentialReferenceInvalid",
            Self::UnlockAuditBlocked => "UnlockAuditBlocked",
            Self::InvalidRequest => "InvalidRequest",
            Self::PermissionDenied => "PermissionDenied",
            Self::EndpointAbsent => "EndpointAbsent",
            Self::InvalidDeviceId => "InvalidDeviceId",
        }
    }

    /// 信封 `msg`（契约 §7.6 逐字，ASCII、无插值）。
    #[must_use]
    pub const fn msg(self) -> &'static str {
        match self {
            Self::CredentialReferenceInvalid => "credential_reference_invalid",
            Self::UnlockAuditBlocked => "unlock_audit_blocked",
            Self::InvalidRequest => "invalid_request",
            Self::PermissionDenied => "permission_denied",
            Self::EndpointAbsent => "not_found",
            Self::InvalidDeviceId => "invalid_device_id",
        }
    }

    /// 契约 HTTP 状态码（契约 §7.6 逐字）。
    #[must_use]
    pub const fn http_status(self) -> u16 {
        match self {
            Self::CredentialReferenceInvalid => 403,
            Self::UnlockAuditBlocked => 503,
            Self::InvalidRequest => 400,
            Self::PermissionDenied => 403,
            Self::EndpointAbsent => 404,
            Self::InvalidDeviceId => 400,
        }
    }
}

/// 解锁客户端模型的**本地**错误。
///
/// ⛔ 变体语义固定、`Display` 只输出固定文本：错误里不携带任何输入片段。
/// 跨端按 [`UnlockModelError::stable_name`] 分支，数字码见
/// [`UnlockModelError::contract_code`]。
///
/// ⚠️ 响应形状类错误（`InvalidSecretHex` / `EmptySecret` / `SecretNotSingle`）
/// 在契约 §7.6 里没有独立码行；本模块按「这次解锁没有产出可用的凭据引用」
/// 归入 **60006**（与 `UnsealFailed` / `Store(_)` 同一档），已在卡报告中注明。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Error)]
#[non_exhaustive]
pub enum UnlockModelError {
    /// `credential_id` 不是 32 位 hex。
    #[error("凭据 ID 非法")]
    InvalidCredentialId,

    /// `secret_id` 为空或仅空白。
    #[error("secret id 非法")]
    InvalidSecretId,

    /// `store_id` 为空或仅空白。
    #[error("store id 非法")]
    InvalidStoreId,

    /// `device_id` 非法（规则复用 [`DeviceId`]）。
    #[error("设备 ID 非法")]
    InvalidDeviceId,

    /// 请求体出现四字段之外的字段（含 `tenant_id` / `user_id` / `aad`）。
    #[error("请求体含未知字段")]
    UnknownField,

    /// 明文不是**非空偶数长度小写 hex**。
    #[error("明文编码非法")]
    InvalidSecretHex,

    /// 明文为空（成功响应必须恰好一条非空 secret）。
    #[error("明文为空")]
    EmptySecret,

    /// 响应里出现不止一条 secret（不得是数组 / 不得有第二个 secret）。
    #[error("响应包含不止一条 secret")]
    SecretNotSingle,

    /// 服务端返回了**已知**的非 0 契约码。
    #[error("服务端拒绝了本次解锁")]
    ServerRejected(UnlockServerError),

    /// 响应不是成功形状（非 0 未知码 / `code == 0` 但 `msg != "ok"`）。
    #[error("响应状态不是成功")]
    UnexpectedStatus,
}

impl UnlockModelError {
    /// 稳定名字（**跨端契约**：一旦发布不再改动，跨端按它分支）。
    #[must_use]
    pub const fn stable_name(self) -> &'static str {
        match self {
            Self::InvalidCredentialId => "InvalidCredentialId",
            Self::InvalidSecretId => "InvalidSecretId",
            Self::InvalidStoreId => "InvalidStoreId",
            Self::InvalidDeviceId => "InvalidDeviceId",
            Self::UnknownField => "UnknownField",
            Self::InvalidSecretHex => "InvalidSecretHex",
            Self::EmptySecret => "EmptySecret",
            Self::SecretNotSingle => "SecretNotSingle",
            Self::ServerRejected(_) => "ServerRejected",
            Self::UnexpectedStatus => "UnexpectedStatus",
        }
    }

    /// 对应的契约码（按服务端 §7.6 落码表；本地响应形状错误归 60006）。
    #[must_use]
    pub const fn contract_code(self) -> i32 {
        match self {
            Self::InvalidCredentialId
            | Self::InvalidSecretId
            | Self::InvalidStoreId
            | Self::UnknownField
            | Self::UnexpectedStatus => UnlockServerError::InvalidRequest.contract_code(),
            Self::InvalidDeviceId => UnlockServerError::InvalidDeviceId.contract_code(),
            Self::InvalidSecretHex | Self::EmptySecret | Self::SecretNotSingle => {
                UnlockServerError::CredentialReferenceInvalid.contract_code()
            }
            Self::ServerRejected(error) => error.contract_code(),
        }
    }

    /// 映射到服务端错误语义（`ServerRejected` 原样透传被拒的码）。
    #[must_use]
    pub const fn to_server_error(self) -> UnlockServerError {
        match self {
            Self::InvalidCredentialId
            | Self::InvalidSecretId
            | Self::InvalidStoreId
            | Self::UnknownField
            | Self::UnexpectedStatus => UnlockServerError::InvalidRequest,
            Self::InvalidDeviceId => UnlockServerError::InvalidDeviceId,
            Self::InvalidSecretHex | Self::EmptySecret | Self::SecretNotSingle => {
                UnlockServerError::CredentialReferenceInvalid
            }
            Self::ServerRejected(error) => error,
        }
    }
}

/// 解锁请求体：四项且仅四项，**全部字段已通过校验**。
///
/// 只能由 [`UnlockRequest::new`]（或按字段名构造的
/// [`UnlockRequest::from_fields`]）产出 ⇒ ⛔ 不存在「半合法对象」。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnlockRequest {
    credential_id: String,
    secret_id: String,
    store_id: String,
    device_id: String,
}

impl UnlockRequest {
    /// 请求体**唯一允许**的四个字段名（顺序即契约表格顺序）。
    pub const FIELD_NAMES: [&'static str; 4] =
        ["credential_id", "secret_id", "store_id", "device_id"];

    /// 全量校验并构造请求。
    ///
    /// 校验顺序与服务端接线一致：`credential_id` → `device_id` →
    /// `secret_id` → `store_id`。
    ///
    /// - `credential_id`：恰好 [`CREDENTIAL_ID_HEX_LEN`] 个 ASCII hex 字符
    ///   （大小写皆可），规范化保存为**小写**；
    /// - `device_id`：**复用** [`DeviceId::new`]，⛔ 不重写第二套规则；
    /// - `secret_id` / `store_id`：`trim` 后非空（仅空白一律拒），原样保留。
    ///
    /// # Errors
    /// 违反上述任一规则 ⇒ 对应的 [`UnlockModelError`] 变体。
    pub fn new(
        credential_id: &str,
        secret_id: &str,
        store_id: &str,
        device_id: &str,
    ) -> Result<Self, UnlockModelError> {
        let credential_id = parse_credential_id(credential_id)?;
        let device = DeviceId::new(device_id).map_err(|_| UnlockModelError::InvalidDeviceId)?;
        let secret_id = parse_non_empty_id(secret_id, UnlockModelError::InvalidSecretId)?;
        let store_id = parse_non_empty_id(store_id, UnlockModelError::InvalidStoreId)?;
        Ok(Self {
            credential_id,
            secret_id,
            store_id,
            device_id: device.as_str().to_string(),
        })
    }

    /// 按「字段名 → 值」构造，模拟服务端 `#[serde(deny_unknown_fields)]`。
    ///
    /// - 字段名不在 [`UnlockRequest::FIELD_NAMES`] ⇒ [`UnlockModelError::UnknownField`]
    ///   （`tenant_id` / `user_id` / `aad` 由此被拒）；
    /// - 同一字段出现两次 ⇒ 同样是形状非法，按 `UnknownField` 归口（10002）；
    /// - 缺字段按空值落到该字段自己的校验上（不发明「半合法」默认值）。
    ///
    /// # Errors
    /// 未知 / 重复字段，或 [`UnlockRequest::new`] 的任一校验失败。
    pub fn from_fields(fields: &[(&str, &str)]) -> Result<Self, UnlockModelError> {
        let mut values = ["", "", "", ""];
        let mut seen = [false; 4];
        for &(name, value) in fields {
            let Some(index) = Self::FIELD_NAMES.iter().position(|known| *known == name) else {
                return Err(UnlockModelError::UnknownField);
            };
            if std::mem::replace(&mut seen[index], true) {
                return Err(UnlockModelError::UnknownField);
            }
            values[index] = value;
        }
        Self::new(values[0], values[1], values[2], values[3])
    }

    /// 目标凭据 id（**已规范化为小写 hex**）。
    #[must_use]
    pub fn credential_id(&self) -> &str {
        &self.credential_id
    }

    /// 目标 secret 条目 id（非空，原样保留）。
    #[must_use]
    pub fn secret_id(&self) -> &str {
        &self.secret_id
    }

    /// 目标环境 id（非空，原样保留）。
    #[must_use]
    pub fn store_id(&self) -> &str {
        &self.store_id
    }

    /// 发起设备 id（已通过 [`DeviceId`] 校验）。
    #[must_use]
    pub fn device_id(&self) -> &str {
        &self.device_id
    }
}

/// 解封产物：**一次性**明文载体。
///
/// 字段私有；唯一取值路径是 [`UnlockedSecret::into_bytes`]（消费 `self`）。
/// ⛔ **不实现** `Debug` / `Display` / `Clone` / `Serialize` ⇒ 明文不会因
/// 日志、格式化或克隆而多留一份。
#[must_use = "解锁明文只取一次，用完即弃"]
pub struct UnlockedSecret {
    bytes: Vec<u8>,
}

impl UnlockedSecret {
    /// 消费 `self`，交出明文（调用一次即不可再用）。
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// 明文字节数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// 明文是否为空（成功响应必然为 `false`）。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

/// 解锁成功响应模型。
///
/// 只能由 [`UnlockResponse::from_parts`] 构造（成功路径）：
/// 三条同时成立才产出 [`UnlockedSecret`] ——
/// `code == 0`、`msg == "ok"`、`secret_hex` 为**非空偶数长度小写 hex**。
///
/// [`UnlockResponse::from_secret_list`] 是 `data` 为**数组**形状时的守卫：
/// 恰好一条才委托给 `from_parts`，零条 / 多条一律拒
/// （契约：「只解一条、只回一条」）。
#[must_use = "响应必须被消费并取走明文"]
pub struct UnlockResponse {
    secret: UnlockedSecret,
}

impl UnlockResponse {
    /// 由响应的 `code` / `msg` / `data.secret` 三部分构造成功模型。
    ///
    /// ⚠️ **非 0 码即使 `secret_hex` 非空也必须拒**：不因 `data` 有值就当成功。
    /// 已知非 0 码 → [`UnlockModelError::ServerRejected`]；
    /// 未知非 0 码 / `code == 0` 但 `msg != "ok"` → [`UnlockModelError::UnexpectedStatus`]。
    ///
    /// # Errors
    /// 状态不是成功，或明文不是非空偶数长度小写 hex。
    pub fn from_parts(code: i32, msg: &str, secret_hex: &str) -> Result<Self, UnlockModelError> {
        if code != SUCCESS_CODE {
            return Err(match UnlockServerError::from_contract_code(code) {
                Some(error) => UnlockModelError::ServerRejected(error),
                // 未知码：fail-closed，绝不猜测语义。
                None => UnlockModelError::UnexpectedStatus,
            });
        }
        if msg != SUCCESS_MSG {
            return Err(UnlockModelError::UnexpectedStatus);
        }
        Ok(Self {
            secret: decode_secret_hex(secret_hex)?,
        })
    }

    /// `data` 以**数组**给出时的形状守卫（模拟 `{"secret": [...]}` / 多条）。
    ///
    /// - 状态不是成功 ⇒ 与服务端码归口（同 [`UnlockResponse::from_parts`]）；
    /// - 恰好一条 ⇒ 委托 `from_parts`（成功路径仍只有这一条）；
    /// - 零条 ⇒ [`UnlockModelError::EmptySecret`]；
    /// - 多条 ⇒ [`UnlockModelError::SecretNotSingle`]。
    ///
    /// # Errors
    /// 状态不是成功，或条数不是恰好一条，或单条明文形状非法。
    pub fn from_secret_list(
        code: i32,
        msg: &str,
        secrets: &[&str],
    ) -> Result<Self, UnlockModelError> {
        if code != SUCCESS_CODE || msg != SUCCESS_MSG {
            return Self::from_parts(code, msg, "");
        }
        match secrets {
            [] => Err(UnlockModelError::EmptySecret),
            [only] => Self::from_parts(code, msg, only),
            _ => Err(UnlockModelError::SecretNotSingle),
        }
    }

    /// 消费响应，取走唯一一条明文。
    #[must_use]
    pub fn into_secret(self) -> UnlockedSecret {
        self.secret
    }

    /// 明文字节数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.secret.len()
    }

    /// 明文是否为空（成功响应必然为 `false`）。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.secret.is_empty()
    }
}

/// 校验并**规范化**凭据 id：32 位 ASCII hex → 小写。
fn parse_credential_id(raw: &str) -> Result<String, UnlockModelError> {
    if raw.len() != CREDENTIAL_ID_HEX_LEN || !raw.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(UnlockModelError::InvalidCredentialId);
    }
    Ok(raw.to_ascii_lowercase())
}

/// 校验「非空」id：仅空白同样视为空（fail-closed），否则原样保留（不 trim）。
fn parse_non_empty_id(raw: &str, error: UnlockModelError) -> Result<String, UnlockModelError> {
    if raw.trim().is_empty() {
        return Err(error);
    }
    Ok(raw.to_string())
}

/// 解码成功响应里的明文：**非空偶数长度小写 hex**。
///
/// ⛔ 大写、奇数长度、`0x` 前缀、空白或任何非 `[0-9a-f]` 字符一律拒。
fn decode_secret_hex(secret_hex: &str) -> Result<UnlockedSecret, UnlockModelError> {
    if secret_hex.is_empty() {
        return Err(UnlockModelError::EmptySecret);
    }
    let text = secret_hex.as_bytes();
    if text.len() % 2 != 0 {
        return Err(UnlockModelError::InvalidSecretHex);
    }
    if !text
        .iter()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
    {
        return Err(UnlockModelError::InvalidSecretHex);
    }
    let mut bytes = Vec::with_capacity(text.len() / 2);
    for pair in text.chunks_exact(2) {
        // 上一步已把字符集收紧到 `[0-9a-f]` ⇒ 这里不可能越界。
        bytes.push((nibble(pair[0]) << 4) | nibble(pair[1]));
    }
    Ok(UnlockedSecret { bytes })
}

/// 单个**已校验**小写 hex 字符 → 半字节。
const fn nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        _ => byte - b'a' + 10,
    }
}
