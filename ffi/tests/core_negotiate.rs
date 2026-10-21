//! 门面层**通道协商**测试。
//!
//! 覆盖：能力集有交集 ⇒ 成功；通道模式无交集 ⇒ 明确失败且**不回退默认模式**；
//! 超时 ⇒ 稳定 `Timeout`。

mod common;

use std::time::{Duration, Instant};

use common::{initiator_negotiator, responder_negotiator};
use lengsuo_mobile_ffi::core::{
    Capability, CapabilitySet, ChannelMode, Generation, ModeSet, NegotiationResult,
};

#[test]
fn negotiation_succeeds_when_capabilities_overlap() {
    let both_modes = ModeSet::empty()
        .with(ChannelMode::P2p)
        .with(ChannelMode::Relay);
    let mut initiator = initiator_negotiator(
        Capability::Keepalive.bit() | Capability::Renegotiate.bit(),
        both_modes,
    );
    let mut responder = responder_negotiator(Capability::Keepalive.bit(), both_modes);
    let now = Instant::now();

    let proposal = initiator.propose(ChannelMode::P2p, now).expect("提案失败");
    let response = responder.accept_proposal(&proposal, now).expect("应答失败");
    let result: NegotiationResult = initiator
        .accept_response(&response, now)
        .expect("确认应答失败");

    assert_eq!(result.channel_mode(), ChannelMode::P2p);
    assert_eq!(
        result.capabilities(),
        CapabilitySet::from_bits(Capability::Keepalive.bit())
    );
    assert_eq!(result.generation(), Generation::INITIAL);
    assert!(initiator.is_complete());
    assert!(responder.is_complete());
}

#[test]
fn negotiation_fails_when_channel_modes_do_not_overlap() {
    let mut initiator = initiator_negotiator(
        Capability::Keepalive.bit(),
        ModeSet::empty().with(ChannelMode::P2p),
    );
    let mut responder = responder_negotiator(
        Capability::Keepalive.bit(),
        ModeSet::empty().with(ChannelMode::Relay),
    );
    let now = Instant::now();

    let proposal = initiator.propose(ChannelMode::P2p, now).expect("提案失败");
    let error = responder
        .accept_proposal(&proposal, now)
        .expect_err("通道模式无交集必须失败");
    assert_eq!(error.stable_name(), "NoCommonChannelMode");
    // fail-closed：不得回退默认模式，也不得留下任何协商结果。
    assert!(responder.is_failed());
    assert!(responder.result().is_none());
}

#[test]
fn negotiation_timeout_is_reported_as_stable_error() {
    let mut initiator = initiator_negotiator(
        Capability::Keepalive.bit(),
        ModeSet::empty().with(ChannelMode::P2p),
    );
    let start = Instant::now();
    initiator
        .propose(ChannelMode::P2p, start)
        .expect("提案失败");

    let later = start
        .checked_add(Duration::from_secs(2))
        .expect("时间加法失败");
    assert!(initiator.is_timed_out(later, Duration::from_secs(1)));
    assert_eq!(
        initiator
            .check_timeout(later, Duration::from_secs(1))
            .expect_err("超时必须报错")
            .stable_name(),
        "Timeout"
    );
    assert!(!initiator.is_timed_out(later, Duration::from_secs(5)));
}
