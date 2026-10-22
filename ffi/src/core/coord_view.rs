//! 移动端「**协调面设备视图 + 权威在线判定**」模型。
//!
//! # 这一层为什么存在
//!
//! 服务端协调面已落地 `DeviceView` / `PeerView` 与心跳超时口径，而移动端
//! 门面层此前**完全没有**「设备是否在线 / 可用」的模型；N11 远程授权门的
//! 执行侧却必须回答「目标设备在不在」。本模块把服务端的设备视图与在线
//! 判定口径**逐字复刻**为纯 Rust 门面层模型。
//!
//! ⛔ 不做 uniffi 导出、⛔ 不做 HTTP 客户端、⛔ 不做 iOS/Android 壳；
//! 本模块**不取本地时钟、不做 IO**（纪律同 [`super`]）。
//!
//! # 语义红线（逐条钉死）
//!
//! 1. **服务端单源权威**：移动端**只**消费服务端给的 `online` 布尔值。
//!    ⛔ **不得**用本地时钟 + `last_seen_at_ms` 自行推算在线——90s 是服务端
//!    的心跳超时窗口，**不是**客户端的计时职责。⇒ 本模块**不实现**超时窗口。
//! 2. **fail-closed**：协调面不可达 / 请求失败 / 响应不可解析 ⇒
//!    [`Availability::Unknown`]（不可用）。⛔ 绝不降级成 `Online`。
//! 3. **`online=false` 是成功响应**（设备存在但心跳超时），**不是**解析错误：
//!    [`DeviceView::new`] 接受它，[`DeviceView::availability`] 返回
//!    [`Availability::Offline`]。
//! 4. **指纹不是权威值**：权威值是完整 `public_key`。`public_key_fingerprint`
//!    只做格式校验（16 位小写十六进制），⛔ 不在移动端重算指纹做「验证」，
//!    ⛔ 不用指纹做密码学判定或判等。
//! 5. `last_seen_at_ms` 在移动端**只做展示 / 诊断**，⛔ 不参与可用性判定。
//!
//! # 错误码
//!
//! [`CoordViewError`] 只做**服务端码 → 语义**的对照（[`CoordViewError::server_code`]），
//! ⛔ 不把 `50101` / `50117` / `50004` 当成成功。跨端按
//! [`CoordViewError::stable_name`] 分支，⛔ 不解析 `Display` 文本。
//! 服务端对「指纹格式非法」没有独立变体 ⇒ 本模块兜底用 `50102`（公钥域错），
//! 该兜底已写入卡报告等主理人复核。

use std::fmt;

use thiserror::Error;

use super::device::{DeviceId, NatCandidate, PublicKey};

/// 公钥指纹的**固定字符长度**（16 位小写十六进制）。
///
/// ⚠️ 指纹是 FNV-1a 64 位的展示值，**不是**密码学摘要；权威值是完整公钥。
pub const FINGERPRINT_LEN: usize = 16;

/// 服务端下发的**设备视图**（移动端镜像）。
///
/// 字段与顺序逐字对齐服务端协调面的设备视图。构造请走
/// [`DeviceView::new`]（字段为 `pub` 是为了跨端门面直读，校验入口仍是 `new`）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceView {
    /// 设备标识（已通过 [`DeviceId`] 校验）。
    pub device_id: String,
    /// **服务端**给出的在线判定（唯一权威值）。
    pub online: bool,
    /// 完整公钥（已通过 [`PublicKey`] 校验；**权威值**）。
    pub public_key: String,
    /// 公钥展示指纹（16 位小写十六进制；**仅展示**，非密码学摘要）。
    pub public_key_fingerprint: String,
    /// 服务端记录的最后心跳时刻（**仅展示 / 诊断**，不参与可用性判定）。
    pub last_seen_at_ms: i64,
}

impl DeviceView {
    /// 校验并构造设备视图。
    ///
    /// - `device_id` 复用 [`DeviceId::new`] 的规则；
    /// - `public_key` 复用 [`PublicKey::parse`] 的规则；
    /// - `public_key_fingerprint` **只校验格式**：恰好 [`FINGERPRINT_LEN`] 个
    ///   小写十六进制字符（`0-9a-f`）。⛔ 不重算指纹、⛔ 不与公钥交叉验证；
    /// - `online == false` **是合法值**（设备存在但心跳超时），不是错误。
    ///
    /// `last_seen_at_ms` 原样保存，只做展示 / 诊断。
    ///
    /// # Errors
    /// 设备 ID / 公钥 / 指纹格式非法 ⇒ 对应的 [`CoordViewError`] 变体。
    pub fn new(
        device_id: &str,
        online: bool,
        public_key: &str,
        public_key_fingerprint: &str,
        last_seen_at_ms: i64,
    ) -> Result<Self, CoordViewError> {
        Ok(Self {
            device_id: parse_device_id(device_id)?,
            online,
            public_key: parse_public_key(public_key)?,
            public_key_fingerprint: parse_fingerprint(public_key_fingerprint)?,
            last_seen_at_ms,
        })
    }

    /// 本设备的可用性判定：**逐字**返回服务端给的 `online`。
    ///
    /// `online == true` ⇒ [`Availability::Online`]；
    /// `online == false` ⇒ [`Availability::Offline`]。
    ///
    /// ⛔ 不用 `last_seen_at_ms` + 90s 窗口重算——服务端单源权威。
    #[must_use]
    pub const fn availability(&self) -> Availability {
        Availability::from_server_answer(Some(self.online))
    }

    /// 本设备此刻是否可用于远程授权等后续动作。
    ///
    /// 只有 [`Availability::Online`] ⇒ `true`；`Offline` 与 `Unknown` 均 `false`。
    #[must_use]
    pub fn is_usable(&self) -> bool {
        self.availability().is_usable()
    }
}

/// 服务端下发的**对端视图**（移动端镜像）。
///
/// 比 [`DeviceView`] 多一份 NAT 候选列表；候选类型**复用**
/// [`NatCandidate`]（⛔ 不另写一套），因此列表里的每一项必然已通过
/// [`NatCandidate::new`] 校验。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PeerView {
    /// 设备标识（已通过 [`DeviceId`] 校验）。
    pub device_id: String,
    /// **服务端**给出的在线判定（唯一权威值）。
    pub online: bool,
    /// 完整公钥（已通过 [`PublicKey`] 校验；**权威值**）。
    pub public_key: String,
    /// 公钥展示指纹（16 位小写十六进制；**仅展示**，非密码学摘要）。
    pub public_key_fingerprint: String,
    /// 服务端记录的最后心跳时刻（**仅展示 / 诊断**）。
    pub last_seen_at_ms: i64,
    /// 对端上报的可达性候选（**已校验**，原样保留）。
    pub nat_candidates: Vec<NatCandidate>,
}

impl PeerView {
    /// 校验并构造对端视图。
    ///
    /// 前五个字段的校验与 [`DeviceView::new`] **完全一致**（直接复用其实现）；
    /// `nat_candidates` 的每一项都是 [`NatCandidate`] ⇒ 已在构造时通过校验，
    /// ⛔ 不存在「绕过校验直接塞字符串」的路径。
    ///
    /// # Errors
    /// 设备 ID / 公钥 / 指纹格式非法 ⇒ 对应的 [`CoordViewError`] 变体。
    pub fn new(
        device_id: &str,
        online: bool,
        public_key: &str,
        public_key_fingerprint: &str,
        last_seen_at_ms: i64,
        nat_candidates: Vec<NatCandidate>,
    ) -> Result<Self, CoordViewError> {
        let base = DeviceView::new(
            device_id,
            online,
            public_key,
            public_key_fingerprint,
            last_seen_at_ms,
        )?;
        Ok(Self {
            device_id: base.device_id,
            online: base.online,
            public_key: base.public_key,
            public_key_fingerprint: base.public_key_fingerprint,
            last_seen_at_ms: base.last_seen_at_ms,
            nat_candidates,
        })
    }

    /// 本对端的可用性判定：**逐字**返回服务端给的 `online`（同 [`DeviceView`]）。
    #[must_use]
    pub const fn availability(&self) -> Availability {
        Availability::from_server_answer(Some(self.online))
    }

    /// 本对端此刻是否可用（只有 [`Availability::Online`] ⇒ `true`）。
    #[must_use]
    pub fn is_usable(&self) -> bool {
        self.availability().is_usable()
    }
}

/// 设备可用性（**三态，只有服务端说了算**）。
///
/// 消费方视角：只有 [`Availability::Online`] 可用；[`Availability::Offline`]
/// 与 [`Availability::Unknown`] 对调用方**等价**（都按不可用处理），
/// 但两者**不得**在日志或错误信息里混为一谈——前者是服务端明确说「离线」，
/// 后者是「读不到权威答案」。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Availability {
    /// 服务端明确回答 `online == true`。
    Online,
    /// 服务端明确回答 `online == false`（设备存在，但心跳已超时）。
    Offline,
    /// **读不到权威答案**：协调面不可达 / 请求失败 / 响应无法解析。
    Unknown,
}

impl Availability {
    /// 由服务端的回答构造三态。
    ///
    /// `Some(true)` ⇒ [`Availability::Online`]；`Some(false)` ⇒
    /// [`Availability::Offline`]；`None`（不可达 / 报错 / 不可解析）⇒
    /// [`Availability::Unknown`]。
    ///
    /// ⛔ `None` **绝不**降级成 `Online`（fail-closed）。
    #[must_use]
    pub const fn from_server_answer(answer: Option<bool>) -> Self {
        match answer {
            Some(true) => Self::Online,
            Some(false) => Self::Offline,
            None => Self::Unknown,
        }
    }

    /// 是否可用：**只有** [`Availability::Online`] 为 `true`。
    #[must_use]
    pub const fn is_usable(&self) -> bool {
        matches!(*self, Self::Online)
    }

    /// 稳定名字（**跨端契约**：跨端按它分支，⛔ 不解析 `Display` 文本）。
    #[must_use]
    pub const fn stable_name(self) -> &'static str {
        match self {
            Self::Online => "Online",
            Self::Offline => "Offline",
            Self::Unknown => "Unknown",
        }
    }
}

impl fmt::Display for Availability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.stable_name())
    }
}

/// 协调面视图相关的**本地**错误。
///
/// ⛔ 变体**全部无字段**：错误里不携带设备 ID / 公钥原文或任何输入片段。
/// 跨端按 [`CoordViewError::stable_name`] 分支，
/// 数字码见 [`CoordViewError::server_code`]。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Error)]
#[non_exhaustive]
pub enum CoordViewError {
    /// 设备 ID 非法。
    #[error("设备 ID 非法")]
    InvalidDeviceId,

    /// 公钥非法。
    #[error("公钥非法")]
    InvalidPublicKey,

    /// 指纹格式非法（长度不是 16 或含非小写十六进制字符）。
    #[error("指纹格式非法")]
    InvalidFingerprint,

    /// 服务端回「设备不存在」（`50101`）。
    #[error("设备不存在")]
    UnknownDevice,

    /// 服务端回「绑定不匹配」（`50117`）。
    #[error("绑定不匹配")]
    BindingMismatch,

    /// 服务端回「设备心跳超时」（`50004`）。
    #[error("设备心跳超时")]
    DeviceOffline,
}

impl CoordViewError {
    /// 稳定名字（**跨端契约**：一旦发布不再改动，跨端按它分支）。
    ///
    /// ⛔ 服务端码 `50102` 同时对应 [`CoordViewError::InvalidPublicKey`] 与
    /// [`CoordViewError::InvalidFingerprint`] ⇒ 跨端**必须按名字**区分，
    /// 不能只靠数字码。
    #[must_use]
    pub const fn stable_name(self) -> &'static str {
        match self {
            Self::InvalidDeviceId => "InvalidDeviceId",
            Self::InvalidPublicKey => "InvalidPublicKey",
            Self::InvalidFingerprint => "InvalidFingerprint",
            Self::UnknownDevice => "UnknownDevice",
            Self::BindingMismatch => "BindingMismatch",
            Self::DeviceOffline => "DeviceOffline",
        }
    }

    /// 对应的**服务端错误码**（逐字对齐服务端 `CoordError::code()`）。
    ///
    /// | 变体 | 码 | 说明 |
    /// |---|---|---|
    /// | `InvalidDeviceId` | 50100 | 设备 ID 非法 |
    /// | `InvalidPublicKey` | 50102 | 公钥非法 |
    /// | `InvalidFingerprint` | 50102 | 服务端**无**独立变体，按公钥域兜底 |
    /// | `UnknownDevice` | 50101 | 设备不存在 |
    /// | `BindingMismatch` | 50117 | 绑定不匹配 |
    /// | `DeviceOffline` | 50004 | 设备心跳超时 |
    #[must_use]
    pub const fn server_code(self) -> u32 {
        match self {
            Self::InvalidDeviceId => 50100,
            Self::InvalidPublicKey | Self::InvalidFingerprint => 50102,
            Self::UnknownDevice => 50101,
            Self::BindingMismatch => 50117,
            Self::DeviceOffline => 50004,
        }
    }
}

/// 校验并保留设备 ID（**复用** [`DeviceId`]）。
fn parse_device_id(raw: &str) -> Result<String, CoordViewError> {
    DeviceId::new(raw)
        .map(|id| id.as_str().to_string())
        .map_err(|_| CoordViewError::InvalidDeviceId)
}

/// 校验并保留公钥（**复用** [`PublicKey`]）。
fn parse_public_key(raw: &str) -> Result<String, CoordViewError> {
    PublicKey::parse(raw)
        .map(|key| key.as_str().to_string())
        .map_err(|_| CoordViewError::InvalidPublicKey)
}

/// **只**校验指纹格式：恰好 16 位小写十六进制。
///
/// ⛔ 不重算指纹、⛔ 不与公钥做密码学交叉验证——指纹不是权威值。
fn parse_fingerprint(raw: &str) -> Result<String, CoordViewError> {
    let lower_hex = raw
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if raw.len() != FINGERPRINT_LEN || !lower_hex {
        return Err(CoordViewError::InvalidFingerprint);
    }
    Ok(raw.to_string())
}
