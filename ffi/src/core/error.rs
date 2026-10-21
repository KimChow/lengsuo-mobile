//! 门面层的**稳定错误类型**。
//!
//! # 为什么门面要有自己的错误
//!
//! `lengsuo-link-core` 暴露了 `FrameError` / `SessionError` / `NegotiateError`
//! 三套错误枚举。若把它们直接透传到 FFI 边界，绑定层（现成框架 / 手写 ABI）
//! 就被迫依赖**具体错误枚举的形状**——下层一改，跨端接口跟着改。
//!
//! ⇒ 本模块把三类错误**逐项映射**成 [`CoreError`]，只保留稳定的**名字**：
//!
//! - 名字一旦发布**不再改动**（只增不改），跨端按 [`CoreError::stable_name`]
//!   做分支，不解析 `Display` 文本；
//! - 变体**全部无字段**——错误里不可能携带密钥字节、载荷明文或密文内容
//!   （长度、计数器这类元数据也不带，避免成为侧信道）；
//! - 映射是**穷尽**的（`NegotiateError` 标了 `#[non_exhaustive]`，留兜底
//!   `Internal`）；下层新增变体会在升级依赖时暴露，不会被静默吞掉。
//!
//! # 与 `capability` 位的关系
//!
//! 本 crate 顶层 `capability` 模块是「**本端绑定了哪些能力**」，与协议协商
//! 用的 `Capability` 位是**两套东西**；错误名同样不复用、不合并。

use lengsuo_link_core::frame::FrameError;
use lengsuo_link_core::negotiate::NegotiateError;
use lengsuo_link_core::session::SessionError;

/// 门面层稳定错误。
///
/// `stable_name()` 返回的字符串是**跨端契约**（Swift / Kotlin 按它分支）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum CoreError {
    /// 线格式非法：切帧失败、帧头字段非法、非运输报文、接收者不匹配等。
    #[error("帧格式非法")]
    InvalidFrame,

    /// AEAD / 握手 MAC 认证失败：**不回退明文**，会话随即失效。
    #[error("认证失败")]
    AuthFailed,

    /// 重放窗口 / 握手时间戳拒绝：丢包，会话仍可能有效。
    #[error("重放被拒绝")]
    ReplayRejected,

    /// 当前状态不允许该操作（含会话已失效）。
    #[error("当前状态不允许该操作")]
    InvalidState,

    /// 载荷长度超过上限：写入前拒绝，不截断。
    #[error("载荷超长")]
    PayloadTooLong,

    /// 协商失败：模式不被接受 / 不支持等。
    #[error("协商失败")]
    NegotiationFailed,

    /// 协商超时（由调用方传入时间戳判定）。
    #[error("协商超时")]
    Timeout,

    /// 密钥材料非法（长度不对、全零等）。
    #[error("密钥材料非法")]
    InvalidKeyMaterial,

    /// 角色不匹配（让响应方去发起、让发起方去应答等）。
    #[error("角色不匹配")]
    RoleMismatch,

    /// 握手遇到未知对端（静态公钥与配置不符）。
    #[error("握手未知对端")]
    UnknownPeer,

    /// 其他握手失败（长度不符、类型不符、洪水、状态失步等）。
    #[error("握手失败")]
    HandshakeFailed,

    /// 世代号不匹配：对端带着另一代连接来协商。
    #[error("世代号不匹配")]
    GenerationMismatch,

    /// 通道模式无交集：**不**回退任何默认模式。
    #[error("通道模式无交集")]
    NoCommonChannelMode,

    /// 协议 / 协商消息版本不受支持。
    #[error("协议版本不受支持")]
    UnsupportedVersion,

    /// 协商消息格式非法（长度不足、魔数不符、字段越界等）。
    #[error("协商消息格式非法")]
    MalformedMessage,

    /// 发送计数器已用尽：绝不回绕复用。
    #[error("发送计数器已用尽")]
    CounterExhausted,

    /// 传输上下文已永久 poison。
    #[error("传输上下文已永久 poison")]
    Poisoned,

    /// AEAD 封装失败（理论不可达）。
    #[error("AEAD 封装失败")]
    SealFailed,

    /// 密文通道失败（协商层把下层会话错误压成一条）。
    #[error("密文通道失败")]
    TransportFailed,

    /// 内部状态不一致（理论不可达，出现即视为实现缺陷）。
    #[error("内部状态不一致")]
    Internal,
}

impl CoreError {
    /// 稳定名字（**跨端契约**：一旦发布不再改动）。
    #[must_use]
    pub const fn stable_name(self) -> &'static str {
        match self {
            Self::InvalidFrame => "InvalidFrame",
            Self::AuthFailed => "AuthFailed",
            Self::ReplayRejected => "ReplayRejected",
            Self::InvalidState => "InvalidState",
            Self::PayloadTooLong => "PayloadTooLong",
            Self::NegotiationFailed => "NegotiationFailed",
            Self::Timeout => "Timeout",
            Self::InvalidKeyMaterial => "InvalidKeyMaterial",
            Self::RoleMismatch => "RoleMismatch",
            Self::UnknownPeer => "UnknownPeer",
            Self::HandshakeFailed => "HandshakeFailed",
            Self::GenerationMismatch => "GenerationMismatch",
            Self::NoCommonChannelMode => "NoCommonChannelMode",
            Self::UnsupportedVersion => "UnsupportedVersion",
            Self::MalformedMessage => "MalformedMessage",
            Self::CounterExhausted => "CounterExhausted",
            Self::Poisoned => "Poisoned",
            Self::SealFailed => "SealFailed",
            Self::TransportFailed => "TransportFailed",
            Self::Internal => "Internal",
        }
    }

    /// 本错误是否代表「会话/传输上下文已经不可继续使用」。
    ///
    /// 上层据此决定「丢弃当前会话」还是「只丢这一个包」。
    #[must_use]
    pub const fn is_fatal(self) -> bool {
        matches!(
            self,
            Self::AuthFailed | Self::Poisoned | Self::SealFailed | Self::CounterExhausted
        )
    }
}

impl From<FrameError> for CoreError {
    fn from(error: FrameError) -> Self {
        use FrameError as Inner;
        match error {
            Inner::Incomplete { .. }
            | Inner::BadMagic { .. }
            | Inner::UnknownFrameType { .. }
            | Inner::ReservedFlags { .. }
            | Inner::HeaderCrcMismatch { .. }
            | Inner::LengthMismatch { .. }
            | Inner::Desync { .. }
            | Inner::NotTransport(_)
            | Inner::ReceiverMismatch { .. } => Self::InvalidFrame,
            Inner::UnsupportedVersion { .. } => Self::UnsupportedVersion,
            Inner::PayloadTooLong { .. } | Inner::TooLong { .. } => Self::PayloadTooLong,
            Inner::KeyLength { .. } => Self::InvalidKeyMaterial,
            Inner::CounterExhausted { .. } => Self::CounterExhausted,
            Inner::Poisoned => Self::Poisoned,
            Inner::AuthFailed => Self::AuthFailed,
            Inner::SealFailed => Self::SealFailed,
            Inner::ReplayRejected => Self::ReplayRejected,
        }
    }
}

impl From<SessionError> for CoreError {
    fn from(error: SessionError) -> Self {
        use SessionError as Inner;
        match error {
            Inner::InvalidState { .. } | Inner::Invalidated => Self::InvalidState,
            Inner::RoleMismatch { .. } => Self::RoleMismatch,
            Inner::InvalidKeyMaterial(_) | Inner::KeyLength { .. } => Self::InvalidKeyMaterial,
            Inner::HandshakeSetup(_)
            | Inner::HandshakeLength { .. }
            | Inner::HandshakeType(_)
            | Inner::HandshakeState(_)
            | Inner::HandshakeFlood
            | Inner::InitiatorFailed(_) => Self::HandshakeFailed,
            Inner::HandshakeAuthFailed | Inner::HandshakeMacFailed => Self::AuthFailed,
            Inner::HandshakeUnknownPeer => Self::UnknownPeer,
            Inner::HandshakeReplay => Self::ReplayRejected,
            Inner::TooLong { .. } => Self::PayloadTooLong,
            Inner::CounterExhausted { .. } => Self::CounterExhausted,
            Inner::Poisoned => Self::Poisoned,
            Inner::AuthFailed => Self::AuthFailed,
            Inner::SealFailed => Self::SealFailed,
            Inner::NotTransport(_) | Inner::ReceiverMismatch { .. } => Self::InvalidFrame,
            Inner::ReplayRejected => Self::ReplayRejected,
            Inner::Frame(_) => Self::InvalidFrame,
        }
    }
}

impl From<NegotiateError> for CoreError {
    fn from(error: NegotiateError) -> Self {
        use NegotiateError as Inner;
        match error {
            Inner::Internal(_) => Self::Internal,
            Inner::MalformedMessage { .. } => Self::MalformedMessage,
            Inner::UnsupportedVersion { .. } => Self::UnsupportedVersion,
            Inner::PayloadTooLarge { .. } => Self::PayloadTooLong,
            Inner::GenerationMismatch { .. } => Self::GenerationMismatch,
            Inner::NoCommonChannelMode { .. } => Self::NoCommonChannelMode,
            Inner::ChannelModeNotAccepted { .. } | Inner::ChannelModeNotSupported { .. } => {
                Self::NegotiationFailed
            }
            Inner::InvalidState { .. } => Self::InvalidState,
            Inner::RoleMismatch { .. } => Self::RoleMismatch,
            Inner::WrongFrameType { .. } => Self::InvalidFrame,
            Inner::Transport { .. } => Self::TransportFailed,
            // `NegotiateError` 标了 `#[non_exhaustive]`：未来新增变体在这里
            // 归为内部错误，绝不静默当成成功。
            _ => Self::Internal,
        }
    }
}
