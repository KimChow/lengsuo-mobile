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
//! # 门面板 ⇄ 协议能力位：**显式对照表 + fail-closed 换算**
//!
//! 本仓顶层 [`crate::capability`] 的**门面板**位（绑定层的能力声明）与
//! 协议能力位是**两个命名空间**：位号不同、语义也不同
//! （例：门面板的「事件回放」在协议侧**根本不存在**）。
//!
//! ⇒ 两者**不得**互相塞：跨命名空间的换算**只能**走
//! [`capability_set_from_facade`]，它按 [`FACADE_CAPABILITY_MAP`] 逐项换算，
//! **未登记位 / 无协议对应 ⇒ 报错**（⛔ 不静默丢弃、不静默当已知位）。
//!
//! # fail-closed
//!
//! - 通道模式**无交集** ⇒ [`CoreError::NoCommonChannelMode`]，
//!   ⛔ **绝不**回退到默认 `p2p`；
//! - 门面板位换算不出协议能力 ⇒ [`CoreError::NegotiationFailed`]，
//!   ⛔ **绝不**用 `known_only()` 之类静默丢弃；
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

/// 门面板位 → 协议能力位 的**逐项显式对照表**。
///
/// # 为什么要有这张表
///
/// 门面板（[`crate::capability`]）与协议能力集（[`Capability`]）是**两个命名空间**，
/// 位号**不得**互相复用。跨命名空间的换算**只能**走这张表 ——
/// ⛔ 不得靠「位号碰巧一致」隐式对齐。
///
/// # 每一项的含义
///
/// `(门面板位, 该门面板位对应的协议能力)`：
///
/// - `Some(cap)` ⇒ 该门面板能力**对应**协议能力 `cap`，换算成协议位 `cap.bit()`；
/// - `None` ⇒ 该门面板能力在**协议侧没有对应能力** ⇒ 它**不得**占用任何协议位号，
///   换算时**必须报错**（fail-closed）。
///
/// # 当前状态：四项全为「协议侧无对应」
///
/// 门面板的四项分别描述「帧编解码 / 通道协商 / 连接世代号 / 事件回放」，
/// 协议侧的四项则是「keepalive / 重协商 / 中继只转密文 / 双向流式」——
/// **语义两两不同**（例如「事件回放」与「双向流式」毫无关系）。
/// ⇒ **门面板不参与协议协商**。
pub const FACADE_CAPABILITY_MAP: [(u32, Option<Capability>); 4] = [
    (crate::capability::FRAME_CODEC, None),
    (crate::capability::CHANNEL_NEGOTIATION, None),
    (crate::capability::CONNECTION_GENERATION, None),
    (crate::capability::EVENT_REPLAY, None),
];

/// 编译期护栏：门面板**不得**与 link-core **已发布**的能力位重叠。
///
/// 重叠即「同名位不同义」⇒ 对端会按协议语义误解 ⇒ 静默漂移。
/// 这条断言让漂移在**编译期**就暴露，而不是等到协商时。
const _: () = assert!(
    crate::capability::ALL_DEFINED as u64 & Capability::known_mask() == 0,
    "门面板位与 link-core 已发布的能力位重叠 ⇒ 位复用漂移"
);

/// 编译期护栏：门面板每一位都必须是**单个位**（不是位掩码组合）。
const _: () = assert!(
    crate::capability::FRAME_CODEC.count_ones() == 1
        && crate::capability::CHANNEL_NEGOTIATION.count_ones() == 1
        && crate::capability::CONNECTION_GENERATION.count_ones() == 1
        && crate::capability::EVENT_REPLAY.count_ones() == 1,
    "门面板常量必须是单个位"
);

/// 把**门面板**位掩码换算成**协议能力集**（fail-closed）。
///
/// # 换算规则
///
/// - 门面板的每一位必须在 [`FACADE_CAPABILITY_MAP`] 中**登记**，
///   且登记为 `Some(cap)` ⇒ 换算成协议位 `cap.bit()`；
/// - **未登记位**（不在表里）⇒ **报错**；
/// - **登记为「协议侧无对应」**的门面板位 ⇒ **报错**
///   （⛔ 不得跳过、不得静默丢弃）。
///
/// # Errors
///
/// 掩码含**未登记位**或**无协议对应**的门面板位 ⇒ [`CoreError::NegotiationFailed`]
/// （⛔ **不**用 `known_only()` 之类静默丢弃未知位）。
///
/// # 为什么这么严格
///
/// 门面板与协议能力集是两个命名空间；一旦把门面板位**当成**协议位塞进
/// `CapabilitySet::from_bits`，对端会按**协议语义**解读 ⇒ 协商**不报错**，
/// 但双方理解的能力集**完全错位**。⇒ 换算必须**逐项显式**且**fail-closed**。
pub fn capability_set_from_facade(mask: u32) -> Result<CapabilitySet, CoreError> {
    capability_set_from_facade_with(mask, &FACADE_CAPABILITY_MAP)
}

/// [`capability_set_from_facade`] 的可注入对照表版本（内部实现）。
fn capability_set_from_facade_with(
    mask: u32,
    map: &[(u32, Option<Capability>)],
) -> Result<CapabilitySet, CoreError> {
    let mut bits = 0u64;
    for index in 0..32 {
        let bit = 1u32 << index;
        if mask & bit == 0 {
            continue;
        }
        let entry = map.iter().find(|entry| entry.0 == bit);
        // 未登记位 ⇒ fail-closed（⛔ 不静默丢弃）。
        let Some(&(_, capability)) = entry else {
            return Err(CoreError::NegotiationFailed);
        };
        // 登记为「协议侧无对应」⇒ 同样 fail-closed（⛔ 不静默跳过）。
        let Some(capability) = capability else {
            return Err(CoreError::NegotiationFailed);
        };
        bits |= capability.bit();
    }
    Ok(CapabilitySet::from_bits(bits))
}

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

#[cfg(test)]
mod tests {
    use super::*;

    /// 换算机制本身必须**正确**：登记了映射 ⇒ 换算出的协议位等于
    /// **link-core 公开 API** 给出的位（⛔ 期望值不得在本仓硬编码位号）。
    #[test]
    fn registered_mapping_converts_to_the_link_core_bit() {
        let map = [(crate::capability::FRAME_CODEC, Some(Capability::Keepalive))];
        let set = capability_set_from_facade_with(crate::capability::FRAME_CODEC, &map)
            .expect("已登记的映射应当换算成功");
        assert_eq!(set.bits(), Capability::Keepalive.bit());
        assert!(set.contains(Capability::Keepalive));
        assert_eq!(set.unknown_bits(), 0);
    }

    /// 未登记位 ⇒ **报错**（⛔ 不静默丢弃、不静默当已知位）。
    #[test]
    fn unregistered_bit_is_rejected() {
        let map = [(crate::capability::FRAME_CODEC, Some(Capability::Keepalive))];
        let error = capability_set_from_facade_with(1 << 20, &map).expect_err("未登记位必须报错");
        assert_eq!(error.stable_name(), "NegotiationFailed");
    }

    /// 登记为「协议侧无对应」⇒ **报错**（⛔ 不静默跳过）。
    #[test]
    fn mapping_without_protocol_counterpart_is_rejected() {
        let map = [(crate::capability::EVENT_REPLAY, None)];
        let error = capability_set_from_facade_with(crate::capability::EVENT_REPLAY, &map)
            .expect_err("无协议对应的门面板位必须报错");
        assert_eq!(error.stable_name(), "NegotiationFailed");
    }
}
