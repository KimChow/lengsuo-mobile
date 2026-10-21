//! 门面向量测试的**共用夹具**。
//!
//! 这个目录下的文件不会被 Cargo 当作独立测试目标；各测试文件用 `mod common;`
//! 引入。固定密钥 / index / 临时种子都放在这里，保证所有场景可复现。

#![allow(dead_code)] // 每个测试二进制只用到其中一部分夹具。

use std::time::Instant;

use lengsuo_mobile_ffi::core::{
    CapabilitySet, FrameType, Generation, ModeSet, Negotiator, NegotiatorRole, OpenedFrame,
    Session, SessionConfig, SessionRole,
};

/// 发起方静态私钥（`[0x11; 32]`，仅测试用，**不是**真实凭据）。
pub const CLIENT_STATIC_PRIVATE: [u8; 32] = [0x11; 32];
/// 响应方静态私钥（`[0x22; 32]`，仅测试用）。
pub const SERVER_STATIC_PRIVATE: [u8; 32] = [0x22; 32];

/// 由 [`CLIENT_STATIC_PRIVATE`] 经 X25519 导出的公钥。
pub const CLIENT_STATIC_PUBLIC: [u8; 32] = [
    0x7b, 0x4e, 0x90, 0x9b, 0xbe, 0x7f, 0xfe, 0x44, 0xc4, 0x65, 0xa2, 0x20, 0x03, 0x7d, 0x60, 0x8e,
    0xe3, 0x58, 0x97, 0xd3, 0x1e, 0xf9, 0x72, 0xf0, 0x7f, 0x74, 0x89, 0x2c, 0xb0, 0xf7, 0x3f, 0x13,
];

/// 由 [`SERVER_STATIC_PRIVATE`] 经 X25519 导出的公钥。
pub const SERVER_STATIC_PUBLIC: [u8; 32] = [
    0x0f, 0xaa, 0x68, 0x4e, 0xd2, 0x88, 0x67, 0xb9, 0x7f, 0x4a, 0x6a, 0x2d, 0xee, 0x5d, 0xf8, 0xce,
    0x97, 0x4e, 0x76, 0xb7, 0x01, 0x8e, 0x3f, 0x22, 0xa1, 0xc4, 0xcf, 0x26, 0x78, 0x57, 0x0f, 0x20,
];

/// 发起方临时密钥种子（固定，保证场景可复现）。
pub const CLIENT_EPHEMERAL: [u8; 32] = [0xAA; 32];
/// 响应方临时密钥种子（固定）。
pub const SERVER_EPHEMERAL: [u8; 32] = [0xBB; 32];

/// 发起方传输 index。
pub const CLIENT_INDEX: u32 = 0x1000_0001;
/// 响应方传输 index。
pub const SERVER_INDEX: u32 = 0x2000_0002;

/// 发起方门面配置。
pub fn initiator_config() -> SessionConfig {
    SessionConfig::new(
        SessionRole::Initiator,
        CLIENT_STATIC_PRIVATE,
        SERVER_STATIC_PUBLIC,
    )
}

/// 响应方门面配置。
pub fn responder_config() -> SessionConfig {
    SessionConfig::new(
        SessionRole::Responder,
        SERVER_STATIC_PRIVATE,
        CLIENT_STATIC_PUBLIC,
    )
}

/// 用固定密钥在门面层跑完一次完整握手。
pub fn establish_pair() -> (Session, Session) {
    let now = Instant::now();
    let mut initiator = Session::new(initiator_config()).expect("发起方会话构造失败");
    let mut responder = Session::new(responder_config()).expect("响应方会话构造失败");

    let initiation = initiator
        .create_initiation(CLIENT_EPHEMERAL, CLIENT_INDEX, now)
        .expect("发起方产 initiation 失败");
    responder
        .consume_initiation(&initiation, now)
        .expect("响应方消费 initiation 失败");
    let response = responder
        .create_response(SERVER_EPHEMERAL, SERVER_INDEX, now)
        .expect("响应方产 response 失败");
    initiator
        .consume_response(&response, now)
        .expect("发起方消费 response 失败");

    (initiator, responder)
}

/// 封一帧再解开，返回明文。
pub fn roundtrip(sender: &mut Session, receiver: &mut Session, seq: u64, plain: &[u8]) -> Vec<u8> {
    let wire = sender
        .seal_frame(FrameType::Data, seq, plain)
        .expect("封帧失败");
    let opened: OpenedFrame = receiver.open_frame(&wire).expect("解帧失败");
    assert_eq!(opened.frame_type, FrameType::Data);
    assert_eq!(opened.seq, seq);
    opened.plaintext
}

/// 构造协商发起方。
pub fn initiator_negotiator(caps: u64, modes: ModeSet) -> Negotiator {
    Negotiator::new(
        NegotiatorRole::Initiator,
        CapabilitySet::from_bits(caps),
        modes,
        Generation::INITIAL,
    )
}

/// 构造协商响应方。
pub fn responder_negotiator(caps: u64, modes: ModeSet) -> Negotiator {
    Negotiator::new(
        NegotiatorRole::Responder,
        CapabilitySet::from_bits(caps),
        modes,
        Generation::INITIAL,
    )
}

/// 字节的十六进制表示（用于断言错误输出不含密钥）。
pub fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut out, "{byte:02x}").expect("写字符串失败");
    }
    out
}
