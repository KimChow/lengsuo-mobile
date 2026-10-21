#![allow(dead_code)]
// 说明：本文件是**被复用的共享源文件**（被 cabi / uniffi / ffi/tests 以 `#[path]` 引入），
// 各处用到的 API 子集不同 ⇒ 允许 dead_code 警告。

//! FFI spike 的**共享样例核心**（纯 Rust，无 IO、无 FFI、无平台依赖）。
//!
//! # 为什么要有这一层
//!
//! 绑定方案选型（`uniffi` vs 手写 C ABI）的核心判据之一是
//! **「同一组输入，两种绑定方式行为必须一致」**。若两种方案各自写一份业务逻辑，
//! 这个判据就无从验证 —— 差异可能来自业务代码而不是绑定层。
//!
//! ⇒ 本 crate 是**唯一的样例实现**；`spike/cabi` 与 `spike/uniffi` 都只做
//! 「薄包装」：把同一批函数以各自的 ABI 形状暴露出去，**不重写语义**。
//!
//! # 纪律
//!
//! - ⛔ 本 crate **不进最终产物**（`spike/` 是隔离区，选型后由后续卡删掉一份）。
//! - ⛔ 不含 tokio / 任何平台依赖 ⇒ 能在 iOS/Android 目标上编。
//! - ⛔ 密钥材料**字节私有**，`Debug` 只出长度（沿用 `lengsuo-wireguard`
//!   的 `TransportKey` 纪律）。

use std::fmt;

use thiserror::Error;

/// 期望的密钥长度（字节）。
pub const KEY_LEN: usize = 32;

/// 样例密钥：字节私有，`Debug` 只输出长度。
///
/// 纪律同 `lengsuo-wireguard::transport::TransportKey`：
/// **密钥材料不得进入日志、错误信息或 `Debug` 输出**。
#[derive(Clone, PartialEq, Eq)]
pub struct SampleKey {
    bytes: Vec<u8>,
}

impl SampleKey {
    /// 由调用方提供密钥字节；长度必须是 [`KEY_LEN`]。
    pub fn new(bytes: &[u8]) -> Result<Self, SampleError> {
        if bytes.len() != KEY_LEN {
            return Err(SampleError::KeyLength {
                actual: bytes.len(),
                expected: KEY_LEN,
            });
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// 密钥字节长度（**不是**密钥内容）。
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// 是否为空（恒为 `false`，因为长度受检）。
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// 派生一个**非密钥**的公开指纹：FNV-1a 64 位。
    ///
    /// ⛔ 这**不是**密码学哈希，**不得**用于任何安全判定；
    /// 它只是给移动端一个可以安全显示/打日志的「会话指纹」。
    pub fn public_fingerprint(&self) -> u64 {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in &self.bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash
    }
}

impl fmt::Debug for SampleKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SampleKey")
            .field("len", &self.bytes.len())
            // 故意不输出 bytes：密钥材料不得进入日志。
            .finish_non_exhaustive()
    }
}

/// 样例错误类型。
///
/// ⛔ 每个变体都**不得**携带密钥字节或任何可逆的密钥派生材料。
/// `KeyLength` 只带**长度**，`InvalidCapability` 只带**能力位掩码**。
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SampleError {
    /// 密钥长度非法。只报长度，不报内容。
    #[error("密钥长度非法：实际 {actual} 字节，期望 {expected} 字节")]
    KeyLength { actual: usize, expected: usize },

    /// 会话尚未握手。
    #[error("会话尚未握手")]
    NotHandshaked,

    /// 能力位掩码含有未知位。
    #[error("能力位掩码非法：0x{mask:08x} 含有未知位")]
    InvalidCapability { mask: u32 },

    /// 对端标签为空或过长。
    #[error("对端标签非法：长度 {actual} 字节，允许 1..={expected}")]
    InvalidPeerLabel { actual: usize, expected: usize },
}

/// 样例能力位（与 `lengsuo-mobile-ffi` 的 `capability` 模块形状一致）。
pub mod capability {
    /// 支持密文帧的编解码。
    pub const FRAME_CODEC: u32 = 1 << 0;
    /// 支持通道协商。
    pub const CHANNEL_NEGOTIATION: u32 = 1 << 1;
    /// 支持连接世代号。
    pub const CONNECTION_GENERATION: u32 = 1 << 2;
    /// 支持按游标回溯事件。
    pub const EVENT_REPLAY: u32 = 1 << 3;

    /// 全部**已定义**的能力位。
    pub const ALL_DEFINED: u32 =
        FRAME_CODEC | CHANNEL_NEGOTIATION | CONNECTION_GENERATION | EVENT_REPLAY;
}

/// 会话信息：**返回结构体**的样例载荷。
///
/// 字段刻意全部是「移动端能安全显示/传递」的量：
/// **不含**密钥字节、不含派生材料。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInfo {
    /// 会话标识（由密钥指纹与标签派生，**非密钥**）。
    pub session_id: String,
    /// 连接世代号。
    pub epoch: u64,
    /// 协商出的能力位掩码。
    pub capabilities: u32,
    /// 对端标签。
    pub peer_label: String,
    /// 密钥指纹（公开，可安全显示）。
    pub fingerprint: u64,
}

/// 打开一个样例会话。
///
/// **成功**路径：返回 [`SessionInfo`]。
/// **失败**路径：返回 [`SampleError`]（不泄露密钥字节）。
pub fn open_session(key: &[u8], peer_label: &str) -> Result<SessionInfo, SampleError> {
    let sample_key = SampleKey::new(key)?;
    if peer_label.is_empty() || peer_label.len() > 64 {
        return Err(SampleError::InvalidPeerLabel {
            actual: peer_label.len(),
            expected: 64,
        });
    }
    let capabilities = capability::ALL_DEFINED;
    Ok(SessionInfo {
        session_id: format!("s-{:016x}", sample_key.public_fingerprint()),
        epoch: 1,
        capabilities,
        peer_label: peer_label.to_owned(),
        fingerprint: sample_key.public_fingerprint(),
    })
}

/// 把能力位掩码翻译成能力名列表（按位序，**确定性**）。
///
/// 掩码含未知位 ⇒ [`SampleError::InvalidCapability`]。
pub fn capability_names(mask: u32) -> Result<Vec<String>, SampleError> {
    if mask & !capability::ALL_DEFINED != 0 {
        return Err(SampleError::InvalidCapability { mask });
    }
    let mut names = Vec::new();
    if mask & capability::FRAME_CODEC != 0 {
        names.push("FRAME_CODEC".to_owned());
    }
    if mask & capability::CHANNEL_NEGOTIATION != 0 {
        names.push("CHANNEL_NEGOTIATION".to_owned());
    }
    if mask & capability::CONNECTION_GENERATION != 0 {
        names.push("CONNECTION_GENERATION".to_owned());
    }
    if mask & capability::EVENT_REPLAY != 0 {
        names.push("EVENT_REPLAY".to_owned());
    }
    Ok(names)
}

/// 事件回调载荷：**回调**样例的形状。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// 事件序号（单调递增）。
    pub seq: u64,
    /// 事件种类。
    pub kind: String,
    /// 事件载荷（**明文**，样例数据）。
    pub payload: String,
}

/// 生成一串样例事件，供回调样例使用（**确定性**，不依赖随机源）。
#[must_use]
pub fn sample_events(count: u32) -> Vec<Event> {
    (0..count)
        .map(|i| Event {
            seq: u64::from(i) + 1,
            kind: "tick".to_owned(),
            payload: format!("e{i}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key32() -> Vec<u8> {
        (0u8..32).collect()
    }

    #[test]
    fn open_session_succeeds_with_valid_key() {
        let info = open_session(&key32(), "peer-a").expect("应当成功");
        assert_eq!(info.peer_label, "peer-a");
        assert_eq!(info.epoch, 1);
        assert_eq!(info.capabilities, capability::ALL_DEFINED);
        assert!(info.session_id.starts_with("s-"));
    }

    #[test]
    fn open_session_rejects_short_key_without_leaking_bytes() {
        let err = open_session(&[1u8; 8], "peer-a").expect_err("应当失败");
        assert_eq!(
            err,
            SampleError::KeyLength {
                actual: 8,
                expected: 32
            }
        );
        // 错误文本里不得出现密钥字节的可打印形式。
        let text = err.to_string();
        assert!(!text.contains('\u{1}'));
        assert!(text.contains("8"));
    }

    #[test]
    fn key_debug_shows_length_only() {
        let key = SampleKey::new(&key32()).expect("应当成功");
        let debug = format!("{key:?}");
        assert!(debug.contains("len"));
        assert!(debug.contains("32"));
        // 不得出现字节内容
        assert!(!debug.contains("0x1f"));
        assert!(!debug.contains('['));
    }

    #[test]
    fn capability_names_rejects_unknown_bits() {
        let err = capability_names(1 << 20).expect_err("应当失败");
        assert_eq!(err, SampleError::InvalidCapability { mask: 1 << 20 });
    }

    #[test]
    fn capability_names_is_deterministic() {
        let names = capability_names(capability::ALL_DEFINED).expect("应当成功");
        assert_eq!(
            names,
            vec![
                "FRAME_CODEC",
                "CHANNEL_NEGOTIATION",
                "CONNECTION_GENERATION",
                "EVENT_REPLAY"
            ]
        );
    }

    #[test]
    fn empty_peer_label_is_rejected() {
        let err = open_session(&key32(), "").expect_err("应当失败");
        assert_eq!(
            err,
            SampleError::InvalidPeerLabel {
                actual: 0,
                expected: 64
            }
        );
    }

    #[test]
    fn sample_events_are_deterministic() {
        let events = sample_events(3);
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].seq, 1);
        assert_eq!(events[2].payload, "e2");
    }
}
