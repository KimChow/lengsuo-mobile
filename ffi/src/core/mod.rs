//! **绑定无关**的核心门面层。
//!
//! # 这一层为什么存在
//!
//! 移动端最终要通过某种 FFI 机制（现成绑定框架 / 手写 ABI / 未来的其它方案）
//! 调用 Rust 协议栈。绑定机制的选择**不应污染业务逻辑**：
//!
//! ```text
//!   Swift / Kotlin
//!        │  （绑定层：具体方案尚未定稿，不在本模块）
//!        ▼
//!   lengsuo_mobile_ffi::core   ← 本模块：纯 Rust、绑定无关
//!        │
//!        ▼
//!   lengsuo-link-core          ← 协议语义的唯一实现
//! ```
//!
//! ⇒ 本模块是**唯一**需要被绑定层包装的一层。将来换绑定方案时，
//! 只替换上面的薄壳，本模块一行不改。
//!
//! # 纪律（逐条可检查）
//!
//! - ⛔ **不写未检查代码**（crate 级属性已禁止）；
//! - ⛔ **无 C ABI 导出、无绑定宏**：不出现任何绑定机制；
//! - ⛔ **无平台依赖、无 IO**（socket / 文件 / 线程 / 计时器）：
//!   时间一律由调用方传入 `Instant`；
//! - ⛔ **不新增依赖**：只用 `lengsuo-link-core` + `thiserror`；
//! - ⛔ **不直接依赖 `lengsuo-wireguard`**：传输密钥的持有与使用全在
//!   `lengsuo-link-core` 内部，移动端依赖图里不应出现它；
//! - ⛔ **不重写协议**：帧格式、握手、协商全部委托给 link-core；
//! - ⛔ **错误不泄密钥**：[`CoreError`] 变体全部无字段，
//!   `Debug` / `Display` 只输出稳定名字与固定文本。
//!
//! # 模块划分
//!
//! | 模块 | 内容 |
//! |---|---|
//! | [`error`] | 稳定错误类型（名字即跨端契约） |
//! | [`frame`] | 无状态帧编解码（切帧 / 编码 / 解码） |
//! | [`session`] | 会话门面（握手 / 封帧 / 解帧 / 世代号） |
//! | [`negotiate`] | 通道协商门面（能力集 / 模式 / 结果 / 超时） |
//! | [`remote_gate`] | 远程授权门（N11 范围表 / fail-closed 执行侧） |
//! | [`device`] | 设备身份模型（设备 ID / 公钥 / NAT 候选 / 指纹） |
//! | [`unlock`] | 凭据解锁客户端模型（请求体 / 成功响应 / 落码反向映射） |
//!
//! # 最小用法
//!
//! ```text
//! 发起方: SessionConfig::new(Initiator, local_priv, remote_pub)
//!         → Session::new → create_initiation(seed, index, now)
//! 响应方: SessionConfig::new(Responder, local_priv, remote_pub)
//!         → Session::new → consume_initiation(wire, now)
//!         → create_response(seed, index, now)
//! 发起方: consume_response(wire, now)
//! 双方:   seal_frame / open_frame 双向收发
//! ```
//!
//! # 与顶层 `capability` 模块的关系
//!
//! 顶层 [`crate::capability`] 是「**本端绑定了哪些能力**」（绑定层的进度声明）；
//! 本模块协商用的 [`Capability`] 是**协议能力位**。两套位**不合并、不改名**。

pub mod coord_view;
pub mod device;
pub mod error;
pub mod frame;
pub mod negotiate;
pub mod remote_gate;
pub mod session;
pub mod unlock;

pub use device::{
    fnv1a64_hex, DeviceError, DeviceId, NatCandidate, PublicKey, MAX_DEVICE_ID_LEN,
    MAX_NAT_ADDRESS_LEN, MAX_PUBLIC_KEY_LEN, MIN_PUBLIC_KEY_LEN, SERVER_ERROR_CODES,
};
pub use error::CoreError;
pub use frame::{
    decode, decode_exact, encode, split_frames, FrameType, WireFrame, MAX_FRAME_LEN,
    MAX_FRAME_PAYLOAD_LEN, MAX_PLAIN_LEN,
};
pub use negotiate::{
    Capability, CapabilitySet, ChannelMode, ModeSet, NegotiateState, NegotiationResult, Negotiator,
    NegotiatorRole,
};
pub use remote_gate::{
    permission_of_scope, require_scope, scope_of_permission, RemoteGate, RemoteGateError,
    RemoteGateState, RemoteScope, RemoteScopeSet, KNOWN_MASK, PERM_NOTIFICATION_RECEIVE,
    PERM_SESSION_VIEW, PERM_TOOL_APPROVE, REMOTE_SCOPE_PERMISSIONS,
};
pub use session::{Generation, OpenedFrame, Session, SessionConfig, SessionRole, SessionState};
pub use unlock::{
    UnlockModelError, UnlockRequest, UnlockResponse, UnlockServerError, UnlockedSecret,
    CREDENTIAL_ID_HEX_LEN, SUCCESS_CODE, SUCCESS_MSG, UNLOCK_PATH,
};
