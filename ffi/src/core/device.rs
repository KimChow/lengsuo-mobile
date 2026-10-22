//! 移动端「**设备身份**」模型（设备 ID / 公钥 / NAT 候选）。
//!
//! # 这一层为什么存在
//!
//! 移动端此前**完全没有**设备身份概念：门面层里只有帧、会话、协商、错误、
//! 远程授权门五块，**没有**「这台设备是谁」这一层。而设备身份是后续
//! uniffi 生产绑定与配对流程的**共同底座**：设备 ID 决定注册与寻址，
//! 公钥决定归属凭证，NAT 候选决定可达性。
//!
//! ⇒ 本模块只做**纯 Rust 门面层模型 + 校验规则**，⛔ 不做 uniffi 导出、
//! ⛔ 不做 HTTP 客户端、⛔ 不做 iOS/Android 壳。
//!
//! # 规则 = **跨端契约**，逐字钉死
//!
//! 下面的校验规则与错误码来自**服务端已落地的设备身份行为**（任务书给出的
//! 行为规格）。移动端按**同一规格**独立实现，两端一致性靠
//! `tests/vectors/device_identity_v1.json` 这份**跨端测试向量**保证
//! （Swift / Kotlin 侧将来读同一份文件逐条对拍，**不靠真机**）。
//!
//! | 项 | 规则 |
//! |---|---|
//! | 设备 ID | 非空、≤128 **字节**、字符集限 ASCII 字母数字与 `-` `_` `.` `:` |
//! | 公钥 | `trim` 后 16..=512 **字节**、字符集限 ASCII 字母数字与 `+` `/` `=` `-` `_` |
//! | 指纹 | FNV-1a 64 位 ⇒ 16 位小写十六进制（**非**密码学摘要） |
//! | NAT 候选 | `kind` ∈ {`host`,`srflx`,`relay`}、地址非空、≤128 字节、无空白/控制字符 |
//!
//! # fail-closed（没有兜底）
//!
//! - ⛔ **不存在「匿名设备」兜底**：设备 ID 非法 ⇒ 直接拒绝，⛔ 不生成
//!   随机 ID、⛔ 不降级为「未绑定」状态；
//! - ⛔ **未知 `kind` 直接拒绝**：拼写错误（`srflx2`）**不得**被静默当作
//!   `host`；
//! - ⛔ **设备归属冲突直接拒绝**：不同属主注册同一 `device_id` ⇒ 拒绝接管，
//!   **不是**覆盖。
//!
//! # 错误不泄输入
//!
//! [`DeviceError`] 的变体**全部无字段**：错误里不可能携带密钥字节、载荷
//! 明文或密文内容，长度 / 计数器这类元数据**也不带**（避免成为侧信道）。
//! 跨端按 [`DeviceError::stable_name`] 分支，⛔ **不解析 `Display` 文本**。

use std::fmt::Write as _;

use thiserror::Error;

/// 设备 ID 的**最大字节长度**。
///
/// 设备 ID 会**原样**出现在协调面 URL 路径里（`.../devices/{id}/...`）⇒
/// 长度必须封顶，且字符集必须排除路径分隔符与 URL 保留字符。
pub const MAX_DEVICE_ID_LEN: usize = 128;

/// 公钥的**最小字节长度**。
///
/// 下界让「明显不是密钥的短串」在本地就被拒掉，省一次往返。
pub const MIN_PUBLIC_KEY_LEN: usize = 16;

/// 公钥的**最大字节长度**。
pub const MAX_PUBLIC_KEY_LEN: usize = 512;

/// NAT 候选地址的**最大字节长度**。
pub const MAX_NAT_ADDRESS_LEN: usize = 128;

/// 设备身份相关的**本地**错误。
///
/// ⛔ 变体**全部无字段**（与门面层其余错误同风格）：错误值里不携带任何
/// 输入片段，也不带长度 / 计数器等元数据。
///
/// - 稳定名字（[`DeviceError::stable_name`]）是**跨端契约**；
/// - 数字码（[`DeviceError::server_code`]）与服务端返回的码**逐字一致**，
///   移动端可据此查语义（映射表见 [`SERVER_ERROR_CODES`]）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Error)]
#[non_exhaustive]
pub enum DeviceError {
    /// 设备 ID 非法：空 / 全空白 / 超长 / 含不允许的字符。
    #[error("设备 ID 非法")]
    InvalidDeviceId,

    /// 公钥非法：长度越界或含不允许的字符。
    #[error("公钥非法")]
    InvalidPublicKey,

    /// NAT 候选非法：未知 `kind` 或地址不合法。
    #[error("NAT 候选非法")]
    InvalidCandidate,

    /// 设备归属冲突：同一 `device_id` 已属主他方 ⇒ 拒绝接管，⛔ 不覆盖。
    #[error("设备归属冲突")]
    DeviceOwnershipConflict,
}

impl DeviceError {
    /// 稳定名字（**跨端契约**：一旦发布不再改动，跨端按它分支）。
    #[must_use]
    pub const fn stable_name(self) -> &'static str {
        match self {
            Self::InvalidDeviceId => "InvalidDeviceId",
            Self::InvalidPublicKey => "InvalidPublicKey",
            Self::InvalidCandidate => "InvalidCandidate",
            Self::DeviceOwnershipConflict => "DeviceOwnershipConflict",
        }
    }

    /// 对应的**服务端错误码**（跨端按数字码查语义）。
    ///
    /// 四个码与服务端已落地的映射逐字一致：设备 ID 非法 `50100`、
    /// 公钥非法 `50102`、候选非法 `50104`、归属冲突 `50105`。
    #[must_use]
    pub const fn server_code(self) -> u32 {
        match self {
            Self::InvalidDeviceId => 50100,
            Self::InvalidPublicKey => 50102,
            Self::InvalidCandidate => 50104,
            Self::DeviceOwnershipConflict => 50105,
        }
    }
}

/// **稳定名 → 服务端错误码**的常量表。
///
/// 供 Swift / Kotlin 侧按服务端返回的数字码查语义：表里的每一项都是
/// 「跨端契约」，⛔ 不得增删改。
pub const SERVER_ERROR_CODES: [(&str, u32); 4] = [
    ("InvalidDeviceId", 50100),
    ("InvalidPublicKey", 50102),
    ("InvalidCandidate", 50104),
    ("DeviceOwnershipConflict", 50105),
];

/// 设备 ID：一条**已通过校验**的设备标识。
///
/// ⛔ 不存在「匿名设备」：构造失败就是失败，没有默认构造路径。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceId(String);

impl DeviceId {
    /// 校验并构造设备 ID。
    ///
    /// 规则（逐字）：空 / 全空白 ⇒ 拒绝；长度 > [`MAX_DEVICE_ID_LEN`]
    /// **字节** ⇒ 拒绝；每个字符必须满足
    /// `is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':')`
    /// ⇒ 否则拒绝。
    ///
    /// ⛔ 字符集**不得放宽**：设备 ID 会原样出现在 URL 路径里，
    /// `/` `?` `#` `%` 必须被拒。
    ///
    /// # Errors
    /// 违反上述任一规则 ⇒ [`DeviceError::InvalidDeviceId`]。
    pub fn new(id: &str) -> Result<Self, DeviceError> {
        if id.trim().is_empty() || id.len() > MAX_DEVICE_ID_LEN {
            return Err(DeviceError::InvalidDeviceId);
        }
        if !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':'))
        {
            return Err(DeviceError::InvalidDeviceId);
        }
        Ok(Self(id.to_string()))
    }

    /// 设备 ID 原文（**原样保留**，不做任何改写）。
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 公钥：一条**已通过校验**的设备公钥文本。
///
/// 校验通过 ⇒ 文本必然只由 ASCII 字母数字与 `+` `/` `=` `-` `_` 组成 ⇒
/// base64 / base64url（带或不带 padding）与 hex 都能原样承载。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PublicKey(String);

impl PublicKey {
    /// 校验并构造公钥。
    ///
    /// 规则（逐字）：**先** `trim`；`trim` 后的长度不在
    /// `MIN_PUBLIC_KEY_LEN..=MAX_PUBLIC_KEY_LEN` **字节** ⇒ 拒绝；
    /// 每个字符必须满足
    /// `is_ascii_alphanumeric() || matches!(c, '+' | '/' | '=' | '-' | '_')`
    /// ⇒ 否则拒绝。
    ///
    /// # Errors
    /// 违反上述任一规则 ⇒ [`DeviceError::InvalidPublicKey`]。
    pub fn parse(raw: &str) -> Result<Self, DeviceError> {
        let trimmed = raw.trim();
        if trimmed.len() < MIN_PUBLIC_KEY_LEN || trimmed.len() > MAX_PUBLIC_KEY_LEN {
            return Err(DeviceError::InvalidPublicKey);
        }
        if !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '=' | '-' | '_'))
        {
            return Err(DeviceError::InvalidPublicKey);
        }
        Ok(Self(trimmed.to_string()))
    }

    /// 公钥原文（**已 trim**，其余原样保留）。
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 公钥的**展示指纹**（FNV-1a 64 位 ⇒ 16 位小写十六进制）。
    ///
    /// ⚠️ **不是**密码学摘要：它只用于展示与日志关联，
    /// ⛔ **不得**用它判定两把公钥「是同一把」——权威值是完整的
    /// `public_key` 本身。
    #[must_use]
    pub fn fingerprint(&self) -> String {
        fnv1a64_hex(self.0.as_bytes())
    }
}

/// NAT 候选：一条**已通过校验**的可达性候选。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NatCandidate {
    kind: String,
    address: String,
    priority: u32,
}

impl NatCandidate {
    /// 全部**已知**的候选类型（三项）。
    ///
    /// ⛔ 未知类型**一律拒绝**（fail-closed）：拼写错误不得被静默当作 `host`。
    pub const KINDS: [&'static str; 3] = ["host", "srflx", "relay"];

    /// 校验并构造 NAT 候选。
    ///
    /// 规则（逐字）：`kind` 必须在 [`NatCandidate::KINDS`] 内，否则拒绝；
    /// 地址 `trim` 后为空 ⇒ 拒绝；地址长度 > [`MAX_NAT_ADDRESS_LEN`]
    /// **字节** ⇒ 拒绝；地址的**任何**字符是控制字符或空白 ⇒ 拒绝。
    ///
    /// 地址语义是 `ip:port`（IPv4）或 `[ipv6]:port`；本层**只做形状校验**，
    /// ⛔ 不解析 IP（解析属网络栈，不在本层）。
    ///
    /// # Errors
    /// 违反上述任一规则 ⇒ [`DeviceError::InvalidCandidate`]。
    pub fn new(kind: &str, address: &str, priority: u32) -> Result<Self, DeviceError> {
        if !Self::KINDS.contains(&kind) {
            return Err(DeviceError::InvalidCandidate);
        }
        if address.trim().is_empty() || address.len() > MAX_NAT_ADDRESS_LEN {
            return Err(DeviceError::InvalidCandidate);
        }
        if address.chars().any(|c| c.is_control() || c.is_whitespace()) {
            return Err(DeviceError::InvalidCandidate);
        }
        Ok(Self {
            kind: kind.to_string(),
            address: address.to_string(),
            priority,
        })
    }

    /// 候选类型（**必然**是 [`NatCandidate::KINDS`] 之一）。
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// 候选地址（原样保留）。
    #[must_use]
    pub fn address(&self) -> &str {
        &self.address
    }

    /// 候选优先级（数值越大越优先，语义由调用方定）。
    #[must_use]
    pub fn priority(&self) -> u32 {
        self.priority
    }
}

/// FNV-1a 64 位哈希的**小写十六进制**渲染（16 位）。
///
/// 算法（逐字）：初值 `0xcbf2_9ce4_8422_2325`；每字节
/// `hash ^= u64::from(byte)`、`hash = hash.wrapping_mul(0x0000_0100_0000_01b3)`；
/// 渲染 `format!("{:016x}", hash)`。
///
/// ⛔ **不是**密码学摘要（见 [`PublicKey::fingerprint`] 的告警）。
#[must_use]
pub fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    let mut out = String::with_capacity(16);
    // `write!` 对 `String` 不会失败。
    let _ = write!(out, "{hash:016x}");
    out
}
