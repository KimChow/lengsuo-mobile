//! **门面板能力位 ⇄ link-core 协议能力位** 的对齐护栏。
//!
//! # 这个文件测什么
//!
//! 移动端门面（本仓 `ffi/src/lib.rs` 的 `pub mod capability`）有一组
//! **门面能力位**；协议协商用的是 `lengsuo-link-core` 的**协议能力位**
//! （`Capability`）。两者**位号不同、语义也不同**。
//!
//! ⚠️ 历史上两边都用 `1 << 0 / 1 << 1 / ...` ⇒ 一旦门面板位被直接塞进
//! `CapabilitySet::from_bits`，对端就会按**协议语义**解读它：
//! 协商**不报错**，但双方理解的能力集**完全错位**（静默漂移），
//! 且**直接决定通道是否可用**。
//!
//! 本文件把「门面板位 ⇄ 协议位」的对应关系**显式化并锁死**：
//! 漂移从此在**编译期 / 测试期**暴露，而不是在生产环境静默发生。
//!
//! # 权威位表（**从 link-core 源码提取**，位号只以 link-core 为准）
//!
//! 来源：`lengsuo-network` 的 `crates/lengsuo-link-core/src/negotiate/capabilities.rs`
//! （`pub enum Capability` 的 `bit_index()` / `known_mask()` / `from_bit_index()`），
//! 与协议规范 `docs/link-negotiate-spec.md` §4.2 一致。
//!
//! | 位号 | link-core 变体 | 语义 | 门面侧对应名 | 门面侧参与协商 |
//! |---|---|---|---|---|
//! | 0 | `Keepalive` | 支持 keepalive 保活帧（L3 `FrameType::Keepalive`） | ——（无） | 否 |
//! | 1 | `Renegotiate` | 支持在**已协商完成的通道**上继续协商 | ——（无） | 否 |
//! | 2 | `CiphertextOnlyRelay` | 中继**只转发密文**（不得看到明文业务内容） | ——（无） | 否 |
//! | 3 | `StreamBothDirections` | 支持**双向**流式业务（不只是请求/响应） | ——（无） | 否 |
//! | 4..63 | —— | （未分配，保留给未来的能力位） | ——（无） | 否 |
//!
//! 门面板自身的四位（**自有命名空间**，刻意避开协议位 `0..=3`）：
//!
//! | 门面板位 | 门面常量 | 语义 | 协议侧对应 | 参与协商 |
//! |---|---|---|---|---|
//! | 16 | `FRAME_CODEC` | 支持密文帧的编解码 | ——（无） | 否 |
//! | 17 | `CHANNEL_NEGOTIATION` | 支持通道协商（`p2p` / `relay`） | ——（无） | 否 |
//! | 18 | `CONNECTION_GENERATION` | 支持连接世代号（去重与补齐） | ——（无） | 否 |
//! | 19 | `EVENT_REPLAY` | 支持按游标回溯事件（`afterSeq` 语义） | ——（无） | 否 |
//!
//! # 采用了哪一套：**默认 A（fail-closed）**
//!
//! 卡面给了 A / B 两个选项，**待主理人拍板**；本卡按 **fail-closed 默认 A** 执行：
//!
//! - **默认 A**：门面板**不占用**协议位号（两个命名空间解耦），
//!   跨命名空间只能走**逐项显式对照表** + **未登记位 fail-closed 报错**；
//! - **默认 B**：门面板能力**逐项映射**到协议真实能力（映射不到的**不宣称**）。
//!
//! ⇒ 本仓**按 A 执行**，理由见 [`FACADE_CAPABILITY_MAP`] 的说明：
//! 门面板四项在协议侧**都没有**对应能力（协议侧是「keepalive / 重协商 /
//! 中继只转密文 / 双向流式」，门面板是「帧编解码 / 通道协商 / 连接世代号 /
//! 事件回放」，**语义两两不同**）⇒ B 的结果与 A **重合**（映射表为空）。
//!
//! # 护栏清单
//!
//! 1. **位表逐项断言**：每个参与协商的门面能力，其协议位号与 link-core 变体一致
//!    （期望值**从 link-core 公开 API 推导**，⛔ 不在本仓硬编码协议位号）；
//! 2. **位复用护栏**：门面板位若落在协议**已知位**范围内，则两者语义**必须一致**
//!    （违反 ⇒ 测试红）；
//! 3. **未知位 ⇒ 报错**：未登记位 / 无协议对应 ⇒ fail-closed（⛔ 不静默丢弃）；
//! 4. **回归护栏**：门面板任何常量都**不得**被静默解释成协议的另一个能力
//!    （例：`EVENT_REPLAY` 不得被解释成 `StreamBothDirections`）；
//! 5. **连通性**：走一次真实 `Negotiator` 流程，断言**双方**看到的能力集一致，
//!    且门面板**不参与**协商。

mod common;

use std::time::Instant;

use common::{initiator_negotiator, responder_negotiator};
use lengsuo_mobile_ffi::capability;
use lengsuo_mobile_ffi::core::negotiate::{capability_set_from_facade, FACADE_CAPABILITY_MAP};
use lengsuo_mobile_ffi::core::{
    Capability, CapabilitySet, ChannelMode, Generation, ModeSet, NegotiatorRole,
};

/// link-core 的四个能力**变体名**（来自其公开 API）。
///
/// ⛔ 这里**只硬编码变体名**，**不硬编码位号** —— 位号一律由
/// `Capability::bit_index()` / `Capability::bit()` 从 link-core 推导。
const LINK_CORE_CAPABILITIES: [Capability; 4] = [
    Capability::Keepalive,
    Capability::Renegotiate,
    Capability::CiphertextOnlyRelay,
    Capability::StreamBothDirections,
];

/// 门面板的四个常量（**本仓自有**，逐项显式列出）。
const FACADE_BITS: [u32; 4] = [
    capability::FRAME_CODEC,
    capability::CHANNEL_NEGOTIATION,
    capability::CONNECTION_GENERATION,
    capability::EVENT_REPLAY,
];

/// 单个位掩码的位号（`None` = 不是单个位）。
fn single_bit_index(mask: u32) -> Option<u8> {
    if mask == 0 || mask.count_ones() != 1 {
        return None;
    }
    Some(mask.trailing_zeros() as u8)
}

/// **位复用护栏**的核心判据：门面板位若落在协议**已知位**范围内，
/// 则它**必须**在对照表里显式登记为**同一个**协议能力 —— 否则即为违例。
///
/// 返回 `true` = **违例**（位复用）。
fn facade_bit_reuse_conflict(facade_bit: u32) -> bool {
    let Some(index) = single_bit_index(facade_bit) else {
        return false;
    };
    let Some(protocol_capability) = Capability::from_bit_index(index) else {
        // 不在协议的**已知**位范围内 ⇒ 不构成「同名位」问题。
        return false;
    };
    // 必须显式登记为同一个协议能力，才算语义一致。
    !FACADE_CAPABILITY_MAP.iter().any(|&(mapped_bit, mapped)| {
        mapped_bit == facade_bit && mapped == Some(protocol_capability)
    })
}

/// 门面板 → 协议能力集 的换算结果（便于断言）。
fn convert(mask: u32) -> Result<CapabilitySet, String> {
    capability_set_from_facade(mask).map_err(|error| error.stable_name().to_owned())
}

// ---------------------------------------------------------------- 1. 位表逐项断言

#[test]
fn link_core_capability_table_is_consistent_with_its_public_api() {
    // 本仓这张表必须**完整覆盖** link-core 已发布的全部能力位
    // （期望值来自 link-core 的 `known_mask()`，⛔ 不是本仓硬编码的位号）。
    let table_mask = LINK_CORE_CAPABILITIES
        .iter()
        .fold(0u64, |acc, capability| acc | capability.bit());
    assert_eq!(
        table_mask,
        Capability::known_mask(),
        "本仓位表必须完整覆盖 link-core 的已知能力位"
    );

    for capability in LINK_CORE_CAPABILITIES {
        let name = capability.as_str();
        // 位号 ⇄ 位掩码：由 link-core 公开 API 互相推导。
        assert_eq!(
            capability.bit(),
            1u64 << capability.bit_index(),
            "{name}：位掩码与位号不一致"
        );
        // 位号 → 变体：往返一致。
        assert_eq!(
            Capability::from_bit_index(capability.bit_index()),
            Some(capability),
            "{name}：位号回解不一致"
        );
        // 已发布位必须落在 `known_mask()` 内。
        assert_eq!(
            capability.bit() & Capability::known_mask(),
            capability.bit(),
            "{name}：已发布能力位必须落在 known_mask 内"
        );
    }
}

#[test]
fn every_facade_bit_is_registered_exactly_once_in_the_explicit_map() {
    assert_eq!(
        FACADE_CAPABILITY_MAP.len(),
        FACADE_BITS.len(),
        "对照表必须逐项登记门面板的每一位"
    );

    for facade_bit in FACADE_BITS {
        let matches = FACADE_CAPABILITY_MAP
            .iter()
            .filter(|&&(mapped_bit, _)| mapped_bit == facade_bit)
            .count();
        assert_eq!(
            matches, 1,
            "门面板位 {facade_bit:#x} 必须在对照表中**恰好**登记一次（实际 {matches} 次）"
        );
    }

    // 对照表里**不得**出现门面板之外的位。
    for &(mapped_bit, _) in &FACADE_CAPABILITY_MAP {
        assert!(
            FACADE_BITS.contains(&mapped_bit),
            "对照表出现未定义的门面板位 {mapped_bit:#x}"
        );
    }

    // 对照表登记的协议能力**必须**是 link-core 已发布的能力。
    for &(facade_bit, mapped) in &FACADE_CAPABILITY_MAP {
        let Some(mapped) = mapped else {
            continue;
        };
        assert_eq!(
            mapped.bit() & Capability::known_mask(),
            mapped.bit(),
            "门面板位 {facade_bit:#x} 映射到未发布的协议能力"
        );
    }
}

#[test]
fn mapped_facade_bits_use_the_link_core_protocol_bit() {
    // 每个**参与协商**的门面能力，其协议位号必须与 link-core 变体一致。
    // （期望值来自 link-core 公开 API，⛔ 不在本仓硬编码协议位号。）
    for &(facade_bit, mapped) in &FACADE_CAPABILITY_MAP {
        let Some(mapped) = mapped else {
            continue;
        };
        let converted =
            capability_set_from_facade(facade_bit).expect("已登记映射的门面板位应当换算成功");
        assert_eq!(
            converted.bits(),
            mapped.bit(),
            "门面板位 {facade_bit:#x} 的协议位必须等于 link-core 变体的位"
        );
        assert_eq!(
            converted.bits(),
            1u64 << mapped.bit_index(),
            "门面板位 {facade_bit:#x} 的协议位必须与 link-core 位号一致"
        );
        assert!(converted.contains(mapped));
        assert_eq!(converted.unknown_bits(), 0, "换算结果不得含未知位");
    }
}

// ---------------------------------------------------------------- 2. 位复用护栏

#[test]
fn facade_bits_never_reuse_the_link_core_known_bit_range() {
    // 门面板整体**不得**与 link-core 已发布的能力位重叠。
    assert_eq!(
        capability::ALL_DEFINED as u64 & Capability::known_mask(),
        0,
        "门面板位与 link-core 已发布能力位重叠 ⇒ 位复用漂移"
    );

    for facade_bit in FACADE_BITS {
        assert!(
            !facade_bit_reuse_conflict(facade_bit),
            "门面板位 {facade_bit:#x} 落在协议已知位范围内却语义不同 ⇒ 位复用"
        );
    }
}

#[test]
fn the_bit_reuse_guardrail_actually_rejects_a_reused_bit() {
    // 反向验证：护栏**不是**空跑。
    // 把门面板位假想地放回协议位 0（`Keepalive`）⇒ 必须被判为违例。
    let hypothetical_reuse = Capability::Keepalive.bit() as u32;
    assert_eq!(single_bit_index(hypothetical_reuse), Some(0));
    assert!(
        facade_bit_reuse_conflict(hypothetical_reuse),
        "护栏必须把「门面板位复用协议位 0」判为违例"
    );

    // 放到协议位范围之外（未分配区）⇒ 不构成「同名位」问题。
    assert!(!facade_bit_reuse_conflict(1u32 << 16));
}

// ---------------------------------------------------------------- 3. 未知位 fail-closed

#[test]
fn unregistered_bits_are_rejected_instead_of_being_dropped() {
    // 未登记的门面板位 ⇒ **报错**。
    let error = convert(1u32 << 20).expect_err("未登记位必须报错");
    assert_eq!(error, "NegotiationFailed");

    // 门面板全量 ⇒ 也必须报错（当前对照表四项全为「协议侧无对应」）。
    let error = convert(capability::ALL_DEFINED).expect_err("门面板全量必须报错");
    assert_eq!(error, "NegotiationFailed");

    // ⛔ **不得**静默丢弃：对照「naive 实现」的做法 ——
    // `known_only()` 会把门面板位悄悄掩掉，而本门面层必须**报错**。
    let naive = CapabilitySet::from_bits(capability::ALL_DEFINED as u64).known_only();
    assert_eq!(
        naive.bits() & u64::from(capability::ALL_DEFINED),
        0,
        "对照：门面板位不得被 known_only() 当成协议能力保留"
    );
    assert!(
        capability_set_from_facade(capability::ALL_DEFINED).is_err(),
        "门面板全量必须报错，而不是静默变成空集"
    );
}

#[test]
fn empty_facade_mask_converts_to_an_empty_capability_set() {
    // 唯一的成功路径：空掩码 ⇒ 空能力集（无位可换算 ⇒ 无歧义）。
    let set = capability_set_from_facade(0).expect("空掩码应当换算成功");
    assert!(set.is_empty());
    assert_eq!(set.bits(), 0);
}

// ---------------------------------------------------------------- 4. 回归护栏

#[test]
fn no_facade_constant_is_silently_read_as_another_protocol_capability() {
    for facade_bit in FACADE_BITS {
        // 每个门面板常量单独换算 ⇒ **必须报错**（不得静默映射成某个协议能力）。
        let converted = capability_set_from_facade(facade_bit);
        assert!(
            converted.is_err(),
            "门面板位 {facade_bit:#x} 不得被静默解释成协议能力"
        );

        // 且门面板位**本身**不得等于任何协议能力位。
        for capability in LINK_CORE_CAPABILITIES {
            assert_ne!(
                u64::from(facade_bit),
                capability.bit(),
                "门面板位 {facade_bit:#x} 与协议能力 {} 位号相同 ⇒ 位复用",
                capability.as_str()
            );
        }
    }

    // 卡面点名的例子：`EVENT_REPLAY` 不得被解释成 `StreamBothDirections`。
    assert!(
        capability_set_from_facade(capability::EVENT_REPLAY).is_err(),
        "EVENT_REPLAY 不得被解释成协议能力"
    );
    assert_eq!(
        capability::EVENT_REPLAY as u64 & Capability::StreamBothDirections.bit(),
        0,
        "EVENT_REPLAY 不得与 StreamBothDirections 共用位号"
    );
}

// ---------------------------------------------------------------- 5. 协商路径连通性

#[test]
fn negotiated_capability_sets_agree_on_both_sides() {
    let both_modes = ModeSet::empty()
        .with(ChannelMode::P2p)
        .with(ChannelMode::Relay);
    let mut initiator = initiator_negotiator(
        Capability::Keepalive.bit() | Capability::Renegotiate.bit(),
        both_modes,
    );
    let mut responder = responder_negotiator(
        Capability::Keepalive.bit() | Capability::StreamBothDirections.bit(),
        both_modes,
    );
    let now = Instant::now();

    let proposal = initiator.propose(ChannelMode::P2p, now).expect("提案失败");
    let response = responder.accept_proposal(&proposal, now).expect("应答失败");
    let result = initiator
        .accept_response(&response, now)
        .expect("确认应答失败");

    // 双方看到的**共同能力集**一致。
    let expected = CapabilitySet::from_bits(Capability::Keepalive.bit());
    assert_eq!(
        result.capabilities(),
        expected,
        "发起方看到的共同能力集不对"
    );
    let responder_result = responder.result().expect("响应方应当有结果");
    assert_eq!(
        responder_result.capabilities(),
        expected,
        "响应方看到的共同能力集不对"
    );
    assert_eq!(result.capabilities(), responder_result.capabilities());

    // 双方看到的**对端能力集**一致。
    assert_eq!(
        initiator.peer_capabilities(),
        Some(CapabilitySet::from_bits(
            Capability::Keepalive.bit() | Capability::StreamBothDirections.bit()
        ))
    );
    assert_eq!(
        responder.peer_capabilities(),
        Some(CapabilitySet::from_bits(
            Capability::Keepalive.bit() | Capability::Renegotiate.bit()
        ))
    );

    // 无未知位 ⇒ 双方对能力集的理解**没有**落差。
    assert_eq!(initiator.peer_unknown_capabilities().bits(), 0);
    assert_eq!(responder.peer_unknown_capabilities().bits(), 0);

    // 门面板**不参与**协商：协商结果里不得出现任何门面板位。
    assert_eq!(
        result.capabilities().bits() & capability::ALL_DEFINED as u64,
        0,
        "门面板位不得混入协议协商结果"
    );

    // 协商结果的位必须全部是 link-core 的**已发布**能力位。
    assert_eq!(result.capabilities().unknown_bits(), 0);
    assert_eq!(result.generation(), Generation::INITIAL);
    assert_eq!(initiator.role(), NegotiatorRole::Initiator);
}
