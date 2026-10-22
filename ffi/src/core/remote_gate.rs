//! 移动端 Rust 侧「远程授权门」（N11 **客户端执行侧**）。
//!
//! # 这一层为什么存在
//!
//! 服务端已落地 N11 的**签发侧**范围模型（`crates/lengsuo-coord/src/scope.rs`，
//! 提交 `f06e4b4`，main = `b0b637b`），移动端此前**没有任何执行侧的门**。
//! 本模块补上执行侧：判定「某一档远程授权此刻能不能用」。
//!
//! # 范围表 = **跨端契约**，逐字钉死
//!
//! 权威出处是服务端 `b0b637b` 的 `crates/lengsuo-coord/src/scope.rs`。
//! 本模块**不自创**任何名字、位序或码值：
//!
//! | scope | 权限码（`域:动作`） | 位 |
//! |---|---|---|
//! | `SessionView` | `session:view` | `0b0000_0001` |
//! | `ToolApprove` | `tool:approve` | `0b0000_0010` |
//! | `Notification` | `notification:receive` | `0b0000_0100` |
//!
//! - 已知掩码 `KNOWN_MASK = 0b0000_0111`：**三档，没有第四档**；
//! - `notification` 在范围表里是单段词，但权限码必须是 `域:动作` **两段**
//!   才判为合法 ⇒ 服务端选用两段形态 `notification:receive`
//!   （服务端 `scope.rs` 的注释已写明该理由）⇒ **本模块用同一个码**。
//!
//! # fail-closed
//!
//! 远程授权范围**由服务端签发**，客户端本地门执行：
//!
//! - 凭据缺失 / 过期 / 协调面不可达 ⇒ **一律拒绝**，
//!   ⛔ **不降级**为明文、⛔ **不降级**为无鉴权；
//! - ⛔ **不引入任何降级开关**（没有「无凭据也放行」的后门）；
//! - ⛔ **未知位拒绝**：[`RemoteScopeSet::from_bits`] 遇到 `KNOWN_MASK`
//!   之外的位 ⇒ **报错**，⛔ 不得静默丢弃。
//!
//! # 不做包含关系推断
//!
//! [`RemoteScope::implies`] **恒为 `false`**：契约没定义谁蕴含谁。
//! 猜「工具审批蕴含会话查看」会静默**放宽**授权，猜反了会静默**收窄**产品
//! ⇒ 两端一律留白，等主理人定。需要精确判定时用 [`RemoteScopeSet::contains`]。

use std::fmt;

use thiserror::Error;

/// 远程授权范围里的一档能力。
///
/// **有且只有三档**（契约冻结）：会话查看 / 工具审批 / 通知。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RemoteScope {
    /// 会话查看。
    SessionView,
    /// 工具审批。
    ToolApprove,
    /// 通知。
    Notification,
}

impl RemoteScope {
    /// 全部档位，**按声明顺序**（顺序同时决定位序与遍历顺序）。
    pub const ALL: [RemoteScope; 3] = [
        RemoteScope::SessionView,
        RemoteScope::ToolApprove,
        RemoteScope::Notification,
    ];

    /// 本档在 [`RemoteScopeSet`] 里占的位。
    #[must_use]
    pub const fn bit(self) -> u8 {
        match self {
            RemoteScope::SessionView => 0b0000_0001,
            RemoteScope::ToolApprove => 0b0000_0010,
            RemoteScope::Notification => 0b0000_0100,
        }
    }

    /// 授予本档的权限码（`域:动作`）。
    #[must_use]
    pub const fn permission(self) -> &'static str {
        match self {
            RemoteScope::SessionView => PERM_SESSION_VIEW,
            RemoteScope::ToolApprove => PERM_TOOL_APPROVE,
            RemoteScope::Notification => PERM_NOTIFICATION_RECEIVE,
        }
    }

    /// 稳定名字（**跨端契约**：一旦发布不再改动）。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            RemoteScope::SessionView => "SessionView",
            RemoteScope::ToolApprove => "ToolApprove",
            RemoteScope::Notification => "Notification",
        }
    }

    /// 本档是否蕴含另一档。
    ///
    /// **恒为 `false`**：契约未定义远程档位之间的包含关系，猜一个方向会静默
    /// 放宽授权（fail-open）或静默收窄产品 ⇒ 两端一律留白，等主理人定。
    #[must_use]
    pub const fn implies(self, _other: RemoteScope) -> bool {
        false
    }
}

impl fmt::Display for RemoteScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// 授予「会话查看」档的权限码。
pub const PERM_SESSION_VIEW: &str = "session:view";
/// 授予「工具审批」档的权限码。
pub const PERM_TOOL_APPROVE: &str = "tool:approve";
/// 授予「通知」档的权限码。
pub const PERM_NOTIFICATION_RECEIVE: &str = "notification:receive";

/// 映射到远程档位的**全部**权限码（三项，顺序同上）。
pub const REMOTE_SCOPE_PERMISSIONS: [&str; 3] = [
    PERM_SESSION_VIEW,
    PERM_TOOL_APPROVE,
    PERM_NOTIFICATION_RECEIVE,
];

/// 位集合里**已知**的位掩码（三档全覆盖）。
///
/// ⛔ 高于本掩码的位是**未知位**：非法输入，必须报错，⛔ 不得静默丢弃。
pub const KNOWN_MASK: u8 = 0b0000_0111;

/// 远程授权档位集合（位集合，`u8` 掩码承载）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct RemoteScopeSet {
    bits: u8,
}

impl RemoteScopeSet {
    /// 空集合。
    #[must_use]
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    /// 由原始位掩码构造集合。
    ///
    /// ⛔ **未知位 fail-closed**：掩码里出现 `KNOWN_MASK` 之外的位 ⇒ **报错**，
    /// ⛔ **不得静默丢弃**（那是掩盖坏调用方 / 坏签发）。
    ///
    /// # Errors
    /// 掩码含未知位 ⇒ [`RemoteGateError::UnknownScopeBits`]。
    pub fn from_bits(bits: u8) -> Result<Self, RemoteGateError> {
        if bits & !KNOWN_MASK != 0 {
            return Err(RemoteGateError::UnknownScopeBits { bits });
        }
        Ok(Self { bits })
    }

    /// 由权限码列表派生集合。
    ///
    /// 不在映射表里的权限码**不授予任何档位**（别的域的码不得放宽远程范围）。
    #[must_use]
    pub fn of_permissions(codes: &[&str]) -> Self {
        let mut set = Self::empty();
        for code in codes {
            if let Some(scope) = scope_of_permission(code) {
                set.insert(scope);
            }
        }
        set
    }

    /// 原始位掩码。
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.bits
    }

    /// 是否**精确**持有该档。
    #[must_use]
    pub const fn contains(self, scope: RemoteScope) -> bool {
        self.bits & scope.bit() != 0
    }

    /// 加入一档；集合变化时返回 `true`。
    pub fn insert(&mut self, scope: RemoteScope) -> bool {
        let before = self.bits;
        self.bits |= scope.bit();
        before != self.bits
    }

    /// 移除一档；集合变化时返回 `true`。
    pub fn remove(&mut self, scope: RemoteScope) -> bool {
        let before = self.bits;
        self.bits &= !scope.bit();
        before != self.bits
    }

    /// 集合里的档数。
    #[must_use]
    pub const fn len(self) -> usize {
        self.bits.count_ones() as usize
    }

    /// 集合是否为空。
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }

    /// 按**声明顺序**遍历集合里的档位。
    pub fn iter(self) -> impl Iterator<Item = RemoteScope> {
        RemoteScope::ALL
            .into_iter()
            .filter(move |scope| self.contains(*scope))
    }
}

/// 权限码 → 档位；不在映射表里则**不授予任何档位**。
#[must_use]
pub fn scope_of_permission(code: &str) -> Option<RemoteScope> {
    match code {
        PERM_SESSION_VIEW => Some(RemoteScope::SessionView),
        PERM_TOOL_APPROVE => Some(RemoteScope::ToolApprove),
        PERM_NOTIFICATION_RECEIVE => Some(RemoteScope::Notification),
        _ => None,
    }
}

/// 档位 → 授予它的权限码。
#[must_use]
pub const fn permission_of_scope(scope: RemoteScope) -> &'static str {
    scope.permission()
}

/// 远程授权门的**状态**（客户端执行侧）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RemoteGateState {
    /// 有有效凭据：携带服务端签发的档位集合。
    Authorized { scopes: RemoteScopeSet },
    /// 凭据缺失。
    CredentialMissing,
    /// 凭据过期。
    CredentialExpired,
    /// 协调面不可达：拿不到 / 校验不了签发结果。
    CoordinationUnreachable,
}

/// 远程授权门：客户端**执行侧**的判定。
///
/// ⛔ **没有降级开关**：没有任何字段能让「无凭据也放行」。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RemoteGate {
    state: RemoteGateState,
}

impl RemoteGate {
    /// 有效凭据 + 已签发的档位集合。
    #[must_use]
    pub const fn authorized(scopes: RemoteScopeSet) -> Self {
        Self {
            state: RemoteGateState::Authorized { scopes },
        }
    }

    /// 有效凭据 + 已签发的权限码 ⇒ 派生档位集合。
    #[must_use]
    pub fn authorized_by_permissions(codes: &[&str]) -> Self {
        Self::authorized(RemoteScopeSet::of_permissions(codes))
    }

    /// 凭据缺失 ⇒ 一律拒绝。
    #[must_use]
    pub const fn credential_missing() -> Self {
        Self {
            state: RemoteGateState::CredentialMissing,
        }
    }

    /// 凭据过期 ⇒ 一律拒绝。
    #[must_use]
    pub const fn credential_expired() -> Self {
        Self {
            state: RemoteGateState::CredentialExpired,
        }
    }

    /// 协调面不可达 ⇒ 一律拒绝。
    #[must_use]
    pub const fn coordination_unreachable() -> Self {
        Self {
            state: RemoteGateState::CoordinationUnreachable,
        }
    }

    /// 当前状态。
    #[must_use]
    pub const fn state(&self) -> RemoteGateState {
        self.state
    }

    /// 当前**已签发**的档位集合。
    ///
    /// 非 [`RemoteGateState::Authorized`] 状态 ⇒ **空集**
    /// （⛔ 不回落到任何档位）。
    #[must_use]
    pub fn scopes(&self) -> RemoteScopeSet {
        match self.state {
            RemoteGateState::Authorized { scopes } => scopes,
            _ => RemoteScopeSet::empty(),
        }
    }
}

/// 要求某一档远程授权，**fail-closed**。
///
/// - 非有效凭据状态 ⇒ 一律 [`RemoteGateError`]，⛔ 不降级；
/// - 有效凭据但缺该档 ⇒ [`RemoteGateError::ScopeMissing`]（点名缺哪一档）。
pub fn require_scope(gate: &RemoteGate, scope: RemoteScope) -> Result<(), RemoteGateError> {
    match gate.state() {
        RemoteGateState::Authorized { scopes } => {
            if scopes.contains(scope) {
                Ok(())
            } else {
                Err(RemoteGateError::ScopeMissing {
                    scope,
                    permission: permission_of_scope(scope),
                })
            }
        }
        RemoteGateState::CredentialMissing => Err(RemoteGateError::CredentialMissing),
        RemoteGateState::CredentialExpired => Err(RemoteGateError::CredentialExpired),
        RemoteGateState::CoordinationUnreachable => Err(RemoteGateError::CoordinationUnreachable),
    }
}

/// 远程授权门的错误。
///
/// ⛔ 不携带任何密钥字节 / 载荷内容：只带**档位名**与**权限码**
/// （二者都是公开契约常量，泄露无意义），以便调用方一眼看出缺哪一档。
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum RemoteGateError {
    /// 缺少某一档远程授权（凭据有效，但没签这一档）。
    #[error("缺少远程授权档位 {scope}（权限码 {permission}）")]
    ScopeMissing {
        /// 缺的那一档。
        scope: RemoteScope,
        /// 该档对应的权限码。
        permission: &'static str,
    },

    /// 位掩码含**未知位**（⛔ 拒绝，不静默丢弃）。
    #[error("未知的授权位掩码 {bits:#04x}")]
    UnknownScopeBits {
        /// 被拒绝的原始掩码。
        bits: u8,
    },

    /// 凭据缺失 ⇒ 拒绝。
    #[error("凭据缺失，拒绝放行")]
    CredentialMissing,

    /// 凭据过期 ⇒ 拒绝。
    #[error("凭据过期，拒绝放行")]
    CredentialExpired,

    /// 协调面不可达 ⇒ 拒绝。
    #[error("协调面不可达，拒绝放行")]
    CoordinationUnreachable,
}

impl RemoteGateError {
    /// 稳定名字（**跨端契约**：一旦发布不再改动）。
    #[must_use]
    pub const fn stable_name(&self) -> &'static str {
        match self {
            Self::ScopeMissing { .. } => "ScopeMissing",
            Self::UnknownScopeBits { .. } => "UnknownScopeBits",
            Self::CredentialMissing => "CredentialMissing",
            Self::CredentialExpired => "CredentialExpired",
            Self::CoordinationUnreachable => "CoordinationUnreachable",
        }
    }
}
