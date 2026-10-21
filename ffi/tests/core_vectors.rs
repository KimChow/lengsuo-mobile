//! 门面层**会话与帧**的跨端向量测试。
//!
//! 覆盖：帧往返、字节级稳定、握手、世代号、未握手拒收发、篡改密文、载荷超长。
//! 错误名稳定性与不泄密钥在 `core_errors.rs`，协商在 `core_negotiate.rs`。
//!
//! # 固定测试密钥
//!
//! 静态私钥取 `[0x11; 32]` / `[0x22; 32]`（仅测试用，**不是**真实凭据）；
//! 对应公钥由 X25519 从私钥导出后固化为常量（见 `common`）。临时密钥种子、
//! index 同样固定，使场景可复现。
//!
//! ⚠️ **传输层发送计数器是会话内状态**：`lengsuo-link-core` 的 `Sender`
//! 每封一帧就把 64 位计数器 +1（AEAD nonce 不可复用）。因此**同一会话**里
//! 两次封同一明文，密文必然不同——这是安全要求，不是不稳定。真正的
//! 「同一密钥 + 同一明文 + 同一序号 ⇒ 同一字节」在**无状态帧编解码**上成立
//! （见 `wire_frame_encoding_is_byte_stable`），密文帧层面只能保证帧头稳定。

mod common;

use std::time::Instant;

use common::{
    establish_pair, initiator_config, responder_config, roundtrip, CLIENT_EPHEMERAL, CLIENT_INDEX,
    SERVER_EPHEMERAL, SERVER_INDEX,
};
use lengsuo_mobile_ffi::core::{
    decode, decode_exact, encode as encode_wire_frame, split_frames, FrameType, Session,
    SessionRole, SessionState, MAX_FRAME_PAYLOAD_LEN, MAX_PLAIN_LEN,
};

// ---------------------------------------------------------------- 1. 帧往返

#[test]
fn frame_roundtrip_between_facade_sessions() {
    let (mut initiator, mut responder) = establish_pair();

    assert_eq!(
        roundtrip(&mut initiator, &mut responder, 1, b"hello-acp"),
        b"hello-acp".to_vec()
    );
    assert_eq!(
        roundtrip(&mut responder, &mut initiator, 2, b"reply-acp"),
        b"reply-acp".to_vec()
    );
}

// ---------------------------------------------------------------- 2. 字节级稳定

#[test]
fn wire_frame_encoding_is_byte_stable() {
    let first = encode_wire_frame(FrameType::Data, 9, b"payload").expect("编码失败");
    let second = encode_wire_frame(FrameType::Data, 9, b"payload").expect("编码失败");
    assert_eq!(first, second, "无状态帧编解码必须逐字节稳定");

    let (frame, consumed) = decode(&first).expect("解码失败");
    assert_eq!(consumed, first.len());
    assert_eq!(frame.seq, 9);
    assert_eq!(frame.frame_type, FrameType::Data);
    assert_eq!(frame.payload.as_slice(), b"payload");
    assert_eq!(decode_exact(&first).expect("精确解码失败"), frame);

    let double = [first.clone(), second].concat();
    let frames = split_frames(&double).expect("粘包切分失败");
    assert_eq!(frames.len(), 2);
}

#[test]
fn sealed_frame_header_is_stable_while_transport_counter_advances() {
    let (mut initiator, _responder) = establish_pair();

    let first = initiator
        .seal_frame(FrameType::Data, 42, b"same-plaintext")
        .expect("第一次封帧失败");
    let second = initiator
        .seal_frame(FrameType::Data, 42, b"same-plaintext")
        .expect("第二次封帧失败");

    // 帧头（20 字节：magic/version/type/seq/payload_len/flags/CRC）必须稳定：
    // 同一帧类型 + 同一序号 + 同一明文长度 ⇒ 帧头逐字节相同。
    assert_eq!(&first[..20], &second[..20]);
    // 载荷（AEAD 密文）按协议随发送计数器变化 —— 这是 nonce 不可复用要求。
    assert_ne!(&first[20..], &second[20..]);
}

// ---------------------------------------------------------------- 3. 握手

#[test]
fn handshake_establishes_both_sides() {
    let (initiator, responder) = establish_pair();

    assert_eq!(initiator.role(), SessionRole::Initiator);
    assert_eq!(responder.role(), SessionRole::Responder);
    assert!(initiator.is_established());
    assert!(responder.is_established());
    assert_eq!(initiator.state(), SessionState::Established);
    assert_eq!(responder.state(), SessionState::Established);
    assert!(!initiator.is_invalidated());
    assert!(!responder.is_invalidated());
}

// ---------------------------------------------------------------- 4. 世代号

#[test]
fn generation_is_exposed_and_does_not_regress_on_reconnect() {
    let (first_initiator, _first_responder) = establish_pair();
    let first_generation = first_initiator.generation();
    assert_eq!(first_initiator.generation_value(), first_generation.value());

    // 模拟「重连时新建会话对象」：显式接续上一代 + 1。
    let next_generation = first_generation.next();
    let config = initiator_config().with_initial_generation(next_generation);
    let mut initiator = Session::new(config).expect("重连会话构造失败");
    let mut responder = Session::new(responder_config()).expect("重连响应方构造失败");

    let now = Instant::now();
    let initiation = initiator
        .create_initiation(CLIENT_EPHEMERAL, CLIENT_INDEX, now)
        .expect("重连 initiation 失败");
    responder
        .consume_initiation(&initiation, now)
        .expect("重连消费 initiation 失败");
    let response = responder
        .create_response(SERVER_EPHEMERAL, SERVER_INDEX, now)
        .expect("重连 response 失败");
    initiator
        .consume_response(&response, now)
        .expect("重连消费 response 失败");

    assert_eq!(initiator.generation_value(), next_generation.value());
    assert!(
        initiator.generation_value() >= first_generation.value(),
        "世代号不得回退"
    );
}

// ---------------------------------------------------------------- 5. 未握手拒收发

#[test]
fn sealing_or_opening_before_handshake_is_rejected() {
    let mut initiator = Session::new(initiator_config()).expect("会话构造失败");

    let seal_error = initiator
        .seal_frame(FrameType::Data, 1, b"too-early")
        .expect_err("未握手不得封帧");
    assert_eq!(seal_error.stable_name(), "InvalidState");

    let open_error = initiator.open_frame(&[0u8; 1]).expect_err("未握手不得解帧");
    assert_eq!(open_error.stable_name(), "InvalidState");
}

// ---------------------------------------------------------------- 6. 篡改密文

#[test]
fn tampered_ciphertext_is_rejected_without_plaintext() {
    let (mut initiator, mut responder) = establish_pair();
    let mut wire = initiator
        .seal_frame(FrameType::Data, 1, b"top-secret-payload")
        .expect("封帧失败");

    // 只翻转密文载荷的最后一个字节（认证标签）：帧头 CRC 仍然有效，
    // 必须在 AEAD 层被拒绝，而不是被当成明文返回。
    let last = wire.len() - 1;
    wire[last] ^= 0x01;

    let error = responder
        .open_frame(&wire)
        .expect_err("篡改后的密文必须被拒绝");
    assert_eq!(error.stable_name(), "AuthFailed");
    // 认证失败 ⇒ 会话立即失效，后续收发也被拒绝。
    assert!(responder.is_invalidated());
    assert_eq!(
        responder
            .seal_frame(FrameType::Data, 2, b"after-tamper")
            .expect_err("失效会话不得继续封帧")
            .stable_name(),
        "InvalidState"
    );
}

// ---------------------------------------------------------------- 7. 载荷超长

#[test]
fn oversized_payload_is_rejected_before_write() {
    let (mut initiator, _responder) = establish_pair();

    let oversized = vec![0u8; MAX_PLAIN_LEN + 1];
    let error = initiator
        .seal_frame(FrameType::Data, 1, &oversized)
        .expect_err("超长明文必须被拒绝");
    assert_eq!(error.stable_name(), "PayloadTooLong");

    // 拒绝发生在写入前：会话仍然可用，随后的小载荷能正常封装。
    assert!(initiator.seal_frame(FrameType::Data, 2, b"small").is_ok());

    // 无状态帧层同样拒绝超长载荷（密文载荷上限）。
    let oversized_frame_payload = vec![0u8; MAX_FRAME_PAYLOAD_LEN + 1];
    let wire_error = encode_wire_frame(FrameType::Data, 3, &oversized_frame_payload)
        .expect_err("超长帧载荷必须被拒绝");
    assert_eq!(wire_error.stable_name(), "PayloadTooLong");
}
