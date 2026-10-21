//! 会话门面：握手编排、密文帧收发、连接世代号。
//!
//! 这是移动端最常用的入口：**发起方 / 响应方**各自持有一个 [`Session`]，
//! 交换握手字节后即可双向收发密文帧。
//!
//! # 为什么走 `Session` 而不是 `frame::seal_frame`
//!
//! `lengsuo-link-core::frame::seal_frame` 需要调用方自己持有
//! `lengsuo-wireguard::transport::Sender`——那会把 `lengsuo-wireguard`
//! 拖进移动端依赖图。本门面**只依赖 `lengsuo-link-core`**，
//! 传输上下文由 `Session` 内部持有。
//!
//! # 确定性
//!
//! link-core **不生成随机数**：握手用的临时密钥种子、传输 index 全部由
//! 调用方传入（`create_initiation` / `create_response` 的 `seed` / `local_index`）。
//! ⇒ 同一组输入可以复现同一套握手字节。
//!
//! ⚠️ 握手报文里的 TAI64N 时间戳由下层取**墙钟**，会进入握手 transcript；
//! 因此「重放同一次握手得到同一会话密钥」**不是**本层的承诺。
//! 密文帧的 AEAD nonce 还随发送计数器单调前进（nonce 不可复用），
//! 所以同一会话两次封同一明文的密文**必然不同**——这是安全要求。
//!
//! # 世代号
//!
//! [`Generation`] 在**每次握手成功**后按 link-core 的规则推进：
//! 首次握手保持初始值（默认 0），之后每轮 +1。重连时若新建会话对象，
//! 用 [`SessionConfig::with_initial_generation`] 传入 `上一代.next()` 接续号段，
//! 否则会退回初始值、失去「区分旧连接迟到包」的意义。

use std::time::{Duration, Instant};

use lengsuo_link_core::session::{Session as LcSession, SessionConfig as LcSessionConfig};

use crate::core::error::CoreError;
use crate::core::frame::{FrameType, MAX_PLAIN_LEN};

pub use lengsuo_link_core::session::{Generation, SessionRole, SessionState};

/// 会话配置：调用方注入的**静态密钥材料**与策略参数。
///
/// `Debug` 只输出角色、世代号、以及「有没有密钥」，**绝不输出密钥字节**。
pub struct SessionConfig {
    inner: LcSessionConfig,
}

impl SessionConfig {
    /// 构造配置（本端静态私钥 + 对端静态公钥）。
    #[must_use]
    pub fn new(
        role: SessionRole,
        local_static_private: [u8; 32],
        remote_static_public: [u8; 32],
    ) -> Self {
        Self {
            inner: LcSessionConfig::new(role, local_static_private, remote_static_public),
        }
    }

    /// 附加预共享密钥（两端必须一致）。
    #[must_use]
    pub fn with_preshared_key(mut self, psk: [u8; 32]) -> Self {
        self.inner = self.inner.with_preshared_key(psk);
        self
    }

    /// 指定起始世代号（重连时新建会话对象用它接续号段）。
    #[must_use]
    pub fn with_initial_generation(mut self, generation: Generation) -> Self {
        self.inner = self.inner.with_initial_generation(generation);
        self
    }

    /// 覆盖握手洪水窗口（`Duration::ZERO` = 关闭会话层洪水检查）。
    #[must_use]
    pub fn with_handshake_flood_window(mut self, window: Duration) -> Self {
        self.inner = self.inner.with_handshake_flood_window(window);
        self
    }

    /// 本端角色。
    #[must_use]
    pub fn role(&self) -> SessionRole {
        self.inner.role
    }

    /// 起始世代号。
    #[must_use]
    pub fn initial_generation(&self) -> Generation {
        self.inner.initial_generation
    }
}

impl core::fmt::Debug for SessionConfig {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("SessionConfig")
            .field("role", &self.inner.role)
            .field("initial_generation", &self.inner.initial_generation)
            .field("has_preshared_key", &self.inner.preshared_key.is_some())
            .field("handshake_flood_window", &self.inner.handshake_flood_window)
            // 故意不输出任何密钥字节。
            .finish_non_exhaustive()
    }
}

/// 一个门面会话对象。
///
/// 状态机、传输上下文与错误映射全部由 `lengsuo-link-core` 持有；
/// 本类型只做**绑定无关的稳定包装**。
pub struct Session {
    inner: LcSession,
}

impl Session {
    /// 用注入的配置构造一个**未握手**的会话。
    ///
    /// # Errors
    /// 密钥材料非法（全零等）⇒ [`CoreError::InvalidKeyMaterial`]。
    pub fn new(config: SessionConfig) -> Result<Self, CoreError> {
        LcSession::new(config.inner)
            .map(|inner| Self { inner })
            .map_err(CoreError::from)
    }

    /// 本会话的角色。
    #[must_use]
    pub fn role(&self) -> SessionRole {
        self.inner.role()
    }

    /// 当前状态。
    #[must_use]
    pub fn state(&self) -> SessionState {
        self.inner.state()
    }

    /// 当前世代号（移动端据此区分旧连接的迟到包）。
    #[must_use]
    pub fn generation(&self) -> Generation {
        self.inner.generation()
    }

    /// 当前世代号的数值形式。
    #[must_use]
    pub fn generation_value(&self) -> u64 {
        self.inner.generation().value()
    }

    /// 是否已建立（可以收发）。
    #[must_use]
    pub fn is_established(&self) -> bool {
        self.inner.is_established()
    }

    /// 是否已失效。
    #[must_use]
    pub fn is_invalidated(&self) -> bool {
        self.inner.is_invalidated()
    }

    /// **发起方**：构造 initiation 报文。
    ///
    /// `seed` 是本轮临时私钥种子（32 字节，由调用方提供）；
    /// `local_index` 本轮必须是**新**值。
    ///
    /// # Errors
    /// 角色不符 / 已有握手在途 / 密钥非法 ⇒ 对应的 [`CoreError`]。
    pub fn create_initiation(
        &mut self,
        seed: [u8; 32],
        local_index: u32,
        now: Instant,
    ) -> Result<Vec<u8>, CoreError> {
        self.inner
            .create_initiation(seed, local_index, now)
            .map_err(CoreError::from)
    }

    /// **发起方**：消费 response 报文并落地传输上下文。
    ///
    /// # Errors
    /// 没有在途 initiation / 响应报文非法或认证失败 ⇒ 对应的 [`CoreError`]。
    pub fn consume_response(&mut self, wire: &[u8], now: Instant) -> Result<(), CoreError> {
        self.inner
            .consume_response(wire, now)
            .map_err(CoreError::from)
    }

    /// **响应方**：消费对端发来的 initiation 报文。
    ///
    /// # Errors
    /// 角色不符 / 已有握手在途 / 长度或 MAC 校验失败 / 洪水 / 重放 ⇒
    /// 对应的 [`CoreError`]。
    pub fn consume_initiation(&mut self, wire: &[u8], now: Instant) -> Result<(), CoreError> {
        self.inner
            .consume_initiation(wire, now)
            .map_err(CoreError::from)
    }

    /// **响应方**：构造 response 报文并落地传输上下文。
    ///
    /// # Errors
    /// 角色不符 / 没有在途 initiation / 状态失步 ⇒ 对应的 [`CoreError`]。
    pub fn create_response(
        &mut self,
        seed: [u8; 32],
        local_index: u32,
        now: Instant,
    ) -> Result<Vec<u8>, CoreError> {
        self.inner
            .create_response(seed, local_index, now)
            .map_err(CoreError::from)
    }

    /// 放弃当前在途握手（对端拒绝 / 超时 / 链路中断时调用）。
    ///
    /// 返回 `true` 表示确实丢弃了一轮在途握手。不作废会话，也不推进世代号。
    pub fn abort_handshake(&mut self) -> bool {
        self.inner.abort_handshake()
    }

    /// 显式作废当前会话（上层主动断链时调用）。
    pub fn invalidate(&mut self) {
        self.inner.invalidate();
    }

    /// 封一帧明文 → 线上字节。
    ///
    /// # Errors
    /// 未握手 / 已失效 ⇒ [`CoreError::InvalidState`]；
    /// 明文超长 ⇒ [`CoreError::PayloadTooLong`]（写入前拒绝）；
    /// 传输层失败 ⇒ 对应的 [`CoreError`]。
    pub fn seal_frame(
        &mut self,
        frame_type: FrameType,
        seq: u64,
        plain: &[u8],
    ) -> Result<Vec<u8>, CoreError> {
        // 在门面边界先做长度检查：link-core 的 `SessionError::Frame` 会把
        // `FrameError::PayloadTooLong` 压成字符串（帧层错误在会话层只保留
        // fatal 语义），若不在这里拦截，跨端会拿到 `InvalidFrame` 而不是
        // 稳定的 `PayloadTooLong`。检查发生在委托之前 ⇒ 不消耗发送计数器。
        if plain.len() > MAX_PLAIN_LEN {
            return Err(CoreError::PayloadTooLong);
        }
        self.inner
            .seal_frame(frame_type, seq, plain)
            .map_err(CoreError::from)
    }

    /// 解一帧线上字节 → 明文。
    ///
    /// # Errors
    /// 未握手 / 已失效 ⇒ [`CoreError::InvalidState`]；
    /// 帧非法 ⇒ [`CoreError::InvalidFrame`]；
    /// 认证失败 ⇒ [`CoreError::AuthFailed`]（不返回明文，会话随即失效）；
    /// 重放 ⇒ [`CoreError::ReplayRejected`]。
    pub fn open_frame(&mut self, buf: &[u8]) -> Result<OpenedFrame, CoreError> {
        self.inner
            .open_frame(buf)
            .map(OpenedFrame::from_inner)
            .map_err(CoreError::from)
    }
}

impl core::fmt::Debug for Session {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Session")
            .field("role", &self.role())
            .field("state", &self.state())
            .field("generation", &self.generation())
            // 故意不输出 config / transport（含密钥材料）。
            .finish_non_exhaustive()
    }
}

/// 解帧结果：帧头字段 + 明文。
///
/// `Debug` **只输出类型、序号与明文长度**，绝不输出明文内容。
#[derive(Clone, PartialEq, Eq)]
pub struct OpenedFrame {
    /// 帧类型。
    pub frame_type: FrameType,
    /// 帧序号。
    pub seq: u64,
    /// 认证解密后的明文。
    pub plaintext: Vec<u8>,
}

impl OpenedFrame {
    fn from_inner(frame: lengsuo_link_core::frame::OpenedFrame) -> Self {
        Self {
            frame_type: frame.frame_type,
            seq: frame.seq,
            plaintext: frame.plaintext,
        }
    }
}

impl core::fmt::Debug for OpenedFrame {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("OpenedFrame")
            .field("frame_type", &self.frame_type)
            .field("seq", &self.seq)
            .field("plaintext_len", &self.plaintext.len())
            // 故意不输出 plaintext。
            .finish_non_exhaustive()
    }
}
