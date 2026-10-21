//! 门面层**错误契约**测试。
//!
//! 覆盖：
//! - 所有 [`CoreError`] 变体的**稳定名字**（跨端按名字分支，不解析文本）；
//! - 触发路径上的名字与契约一致（不是只在枚举上写死）；
//! - 错误 / Debug 输出**不含密钥字节或明文**。

mod common;

use std::time::Instant;

use common::{
    establish_pair, hex, initiator_config, responder_config, CLIENT_STATIC_PRIVATE,
    SERVER_EPHEMERAL, SERVER_INDEX, SERVER_STATIC_PRIVATE,
};
use lengsuo_mobile_ffi::core::{CoreError, FrameType, Session, WireFrame};

#[test]
fn stable_error_names_are_fixed() {
    let expected: [(CoreError, &str); 20] = [
        (CoreError::InvalidFrame, "InvalidFrame"),
        (CoreError::AuthFailed, "AuthFailed"),
        (CoreError::ReplayRejected, "ReplayRejected"),
        (CoreError::InvalidState, "InvalidState"),
        (CoreError::PayloadTooLong, "PayloadTooLong"),
        (CoreError::NegotiationFailed, "NegotiationFailed"),
        (CoreError::Timeout, "Timeout"),
        (CoreError::InvalidKeyMaterial, "InvalidKeyMaterial"),
        (CoreError::RoleMismatch, "RoleMismatch"),
        (CoreError::UnknownPeer, "UnknownPeer"),
        (CoreError::HandshakeFailed, "HandshakeFailed"),
        (CoreError::GenerationMismatch, "GenerationMismatch"),
        (CoreError::NoCommonChannelMode, "NoCommonChannelMode"),
        (CoreError::UnsupportedVersion, "UnsupportedVersion"),
        (CoreError::MalformedMessage, "MalformedMessage"),
        (CoreError::CounterExhausted, "CounterExhausted"),
        (CoreError::Poisoned, "Poisoned"),
        (CoreError::SealFailed, "SealFailed"),
        (CoreError::TransportFailed, "TransportFailed"),
        (CoreError::Internal, "Internal"),
    ];
    for (error, name) in expected {
        assert_eq!(error.stable_name(), name);
    }

    // 触发路径上的名字也必须与契约一致（不是只在枚举上写死）。
    let mut responder = Session::new(responder_config()).expect("会话构造失败");
    assert_eq!(
        responder
            .create_response(SERVER_EPHEMERAL, SERVER_INDEX, Instant::now())
            .expect_err("没有在途 initiation 不得产 response")
            .stable_name(),
        "InvalidState"
    );
}

#[test]
fn errors_and_debug_outputs_never_leak_key_bytes_or_plaintext() {
    let key_hexes = [hex(&CLIENT_STATIC_PRIVATE), hex(&SERVER_STATIC_PRIVATE)];
    let variants = [
        CoreError::InvalidFrame,
        CoreError::AuthFailed,
        CoreError::ReplayRejected,
        CoreError::InvalidState,
        CoreError::PayloadTooLong,
        CoreError::NegotiationFailed,
        CoreError::Timeout,
        CoreError::InvalidKeyMaterial,
        CoreError::RoleMismatch,
        CoreError::UnknownPeer,
        CoreError::HandshakeFailed,
        CoreError::GenerationMismatch,
        CoreError::NoCommonChannelMode,
        CoreError::UnsupportedVersion,
        CoreError::MalformedMessage,
        CoreError::CounterExhausted,
        CoreError::Poisoned,
        CoreError::SealFailed,
        CoreError::TransportFailed,
        CoreError::Internal,
    ];
    for error in variants {
        for rendered in [format!("{error:?}"), format!("{error}")] {
            for key_hex in &key_hexes {
                assert!(
                    !rendered.contains(key_hex.as_str()),
                    "错误输出不得包含密钥字节：{rendered}"
                );
            }
            assert!(!rendered.contains("top-secret-payload"));
        }
    }

    // 配置 / 会话 / 已解帧 / 线帧的 Debug 同样不得输出密钥或明文。
    let (mut initiator, mut responder) = establish_pair();
    let config_debug = format!("{:?}", initiator_config());
    let session_debug = format!("{initiator:?}");
    let wire = initiator
        .seal_frame(FrameType::Data, 1, b"top-secret-payload")
        .expect("封帧失败");
    let opened = responder.open_frame(&wire).expect("解帧失败");
    let opened_debug = format!("{opened:?}");
    let frame_debug = format!("{:?}", WireFrame::new(FrameType::Data, 1, vec![0xAB; 8]));
    for rendered in [config_debug, session_debug, opened_debug, frame_debug] {
        for key_hex in &key_hexes {
            assert!(
                !rendered.contains(key_hex.as_str()),
                "Debug 输出不得包含密钥字节：{rendered}"
            );
        }
        assert!(!rendered.contains("top-secret-payload"));
    }
}
