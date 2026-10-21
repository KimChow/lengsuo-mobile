//! 通道协商门面：能力集、通道模式、协商结果。
//!
//! 协商发生在**会话建立之后**：双方把协商消息封进 `FrameType::Control`
//! 帧互换，决定走 `p2p` 还是 `relay`、共同支持哪些可选能力。
//!
//! # 能力位与模式位**一律复用** link-core 的定义
//!
//! - 能力位：[`Capability`]（`Keepalive=0 / Renegotiate=1 /
//!   CiphertextOnlyRelay=2 / StreamBothDirections=3`，⛔ 位号已发布不可改）；
//! - 模式位：[`ChannelMode`]（`p2p` / `relay`）。
//!
//! 门面**不另编一套位定义**，只做稳定包装与错误映射。
//!
//! # fail-closed
//!
//! - 通道模式**无交集** ⇒ [`CoreError::NoCommonChannelMode`]，
//!   ⛔ **绝不**回退到默认 `p2p`；
//! - 协商失败后**没有任何结果**可回退；
//! - 超时由调用方传入时间戳判定（本层不起计时器），
//!   [`Negotiator::check_timeout`] 把它变成稳定的 [`CoreError::Timeout`]。

use std::time::{Duration, Instant};

use lengsuo_link_core::negotiate::Negotiator as LcNegotiator;
use lengsuo_link_core::session::Generation;

use crate::core::error::CoreError;

pub use lengsuo_link_core::negotiate::{
    Capability, CapabilitySet, ChannelMode, ModeSet, NegotiateState, NegotiationResult,
    NegotiatorRole,
};

/// 协商器门面：一端的状态机。
///
/// 不持有 IO 资源；协商消息的收发由调用方经会话的 `Control` 帧完成。
#[derive(Debug)]
pub struct Negotiator {
    inner: LcNegotiator,
}

impl Negotiator {
    /// 构造协商器。
    ///
    /// - `local_capabilities`：本端支持的**可选特性**位集；
    /// - `local_modes`：本端支持的**通道模式**集；
    /// - `generation`：来自会话门面的世代号（两端必须一致）。
    #[must_use]
    pub fn new(
        role: NegotiatorRole,
        local_capabilities: CapabilitySet,
        local_modes: ModeSet,
        generation: Generation,
    ) -> Self {
        Self {
            inner: LcNegotiator::new(role, local_capabilities, local_modes, generation),
        }
    }

    /// 本端角色。
    #[must_use]
    pub fn role(&self) -> NegotiatorRole {
        self.inner.role()
    }

    /// 当前状态。
    #[must_use]
    pub fn state(&self) -> NegotiateState {
        self.inner.state()
    }

    /// 本端能力集。
    #[must_use]
    pub fn local_capabilities(&self) -> CapabilitySet {
        self.inner.local_capabilities()
    }

    /// 本端模式集。
    #[must_use]
    pub fn local_modes(&self) -> ModeSet {
        self.inner.local_modes()
    }

    /// 对端能力集（已知位部分）。
    #[must_use]
    pub fn peer_capabilities(&self) -> Option<CapabilitySet> {
        self.inner.peer_capabilities()
    }

    /// 对端能力集里本实现**不认识**的位（被忽略并记录）。
    #[must_use]
    pub fn peer_unknown_capabilities(&self) -> CapabilitySet {
        self.inner.peer_unknown_capabilities()
    }

    /// 协商结果（未完成时为 `None`）。
    #[must_use]
    pub fn result(&self) -> Option<NegotiationResult> {
        self.inner.result()
    }

    /// 协商是否已完成（结果可用）。
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.inner.is_complete()
    }

    /// 协商是否已失败。
    #[must_use]
    pub fn is_failed(&self) -> bool {
        self.inner.is_failed()
    }

    /// 本端请求的模式（发起方）。
    #[must_use]
    pub fn requested_mode(&self) -> Option<ChannelMode> {
        self.inner.requested_mode()
    }

    /// 发起方：产出**提案**明文载荷（由调用方封进控制帧）。
    ///
    /// # Errors
    /// 角色 / 状态不符、本端不支持请求的模式、编码超长 ⇒ 对应的 [`CoreError`]。
    pub fn propose(&mut self, requested: ChannelMode, now: Instant) -> Result<Vec<u8>, CoreError> {
        self.inner.propose(requested, now).map_err(CoreError::from)
    }

    /// 响应方：消费**提案**并产出**应答**明文载荷。
    ///
    /// # Errors
    /// 提案非法、世代号不匹配、**通道模式无交集** ⇒ 对应的 [`CoreError`]。
    pub fn accept_proposal(&mut self, proposal: &[u8], now: Instant) -> Result<Vec<u8>, CoreError> {
        self.inner
            .accept_proposal(proposal, now)
            .map_err(CoreError::from)
    }

    /// 发起方：消费**应答**并确定协商结果。
    ///
    /// # Errors
    /// 应答非法、世代号不匹配、对端选中的模式本端不支持 ⇒ 对应的 [`CoreError`]。
    pub fn accept_response(
        &mut self,
        response: &[u8],
        now: Instant,
    ) -> Result<NegotiationResult, CoreError> {
        self.inner
            .accept_response(response, now)
            .map_err(CoreError::from)
    }

    /// 超时判定（不报错，只回答「是否已超时」）。
    #[must_use]
    pub fn is_timed_out(&self, now: Instant, timeout: Duration) -> bool {
        self.inner.is_timed_out(now, timeout)
    }

    /// 超时检查：已超时 ⇒ [`CoreError::Timeout`]。
    ///
    /// # Errors
    /// 协商仍未终结且已超过 `timeout` ⇒ [`CoreError::Timeout`]。
    pub fn check_timeout(&self, now: Instant, timeout: Duration) -> Result<(), CoreError> {
        if self.is_timed_out(now, timeout) {
            Err(CoreError::Timeout)
        } else {
            Ok(())
        }
    }
}
