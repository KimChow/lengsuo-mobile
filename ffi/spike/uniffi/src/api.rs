#![allow(dead_code)]
// 说明：本文件是**被复用的共享源文件**（被 ffi/tests 以 `#[path]` 引入），
// 各处用到的 API 子集不同 ⇒ 允许 dead_code 警告。

//! FFI spike：**uniffi 方案**的**纯 Rust API 层**（不含 uniffi 宏）。
//!
//! # 为什么拆成两层
//!
//! uniffi 的 `#[derive(Enum)]` / `#[derive(Record)]` / `#[uniffi::export]`
//! 都是**过程宏**，只能在**编译期**展开 ⇒ 依赖 `uniffi` crate。
//! 而 `ffi/Cargo.toml` 与 `ffi/Cargo.lock` 属**共享区**（本卡不许动），
//! 所以 `ffi/tests/` **不能**把 `uniffi` 加进依赖树。
//!
//! ⇒ 拆法：
//! - **`api.rs`（本文件）**：uniffi 绑定层的**语义部分**（纯 Rust，无宏）；
//! - **`lib.rs`**：uniffi 的**宏层**（`#[derive]` / `#[uniffi::export]`），
//!   只做「把 `api.rs` 的东西暴露成 uniffi 类型」。
//!
//! ⇒ 于是 `ffi/tests/spike_consistency.rs` 可以用 `#[path]` 引入 `api.rs`，
//! **真的**把「uniffi 绑定」与「手写 C ABI 绑定」放在一起对照
//! —— ��是选型的核心判据。
//!
//! # ⛔ 本 crate 不进最终产物
//!
//! `ffi/spike/` 是隔离区；选型定稿后由后续卡删掉落选的一份。

use super::core as core_api;
use super::core::{SampleError, SessionInfo as CoreSessionInfo};

/// 错误种类（uniffi 绑定的**语义**部分）。
///
/// ⚠️ 只带**长度/掩码**，**不带密钥字节** ⇒ 跨语言后也不泄露。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SpikeErrorKind {
    /// 密钥长度非法（**只报长度**）。
    #[error("密钥长度非法：实际 {actual} 字节，期望 {expected} 字节")]
    KeyLength {
        /// 实际长度。
        actual: u32,
        /// 期望长度。
        expected: u32,
    },

    /// 会话尚未握手。
    #[error("会话尚未握手")]
    NotHandshaked,

    /// 能力位掩码非法。
    #[error("能力位掩码非法：0x{mask:08x} 含有未知位")]
    InvalidCapability {
        /// 掩码值。
        mask: u32,
    },

    /// 对端标签非法。
    #[error("对端标签非法：长度 {actual} 字节，允许 1..={expected}")]
    InvalidPeerLabel {
        /// 实际长度。
        actual: u32,
        /// 允许的最大长度。
        expected: u32,
    },
}

impl From<SampleError> for SpikeErrorKind {
    fn from(err: SampleError) -> Self {
        match err {
            SampleError::KeyLength { actual, expected } => Self::KeyLength {
                actual: actual as u32,
                expected: expected as u32,
            },
            SampleError::NotHandshaked => Self::NotHandshaked,
            SampleError::InvalidCapability { mask } => Self::InvalidCapability { mask },
            SampleError::InvalidPeerLabel { actual, expected } => Self::InvalidPeerLabel {
                actual: actual as u32,
                expected: expected as u32,
            },
        }
    }
}

/// 会话信息（uniffi 绑定的**语义**部分）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInfo {
    /// 连接世代号。
    pub epoch: u64,
    /// 协商出的能力位掩码。
    pub capabilities: u32,
    /// 密钥指纹（公开）。
    pub fingerprint: u64,
    /// 会话标识（**非密钥**）。
    pub session_id: String,
    /// 对端标签（**非密钥**）。
    pub peer_label: String,
}

impl From<CoreSessionInfo> for SessionInfo {
    fn from(info: CoreSessionInfo) -> Self {
        Self {
            epoch: info.epoch,
            capabilities: info.capabilities,
            fingerprint: info.fingerprint,
            session_id: info.session_id,
            peer_label: info.peer_label,
        }
    }
}

/// 事件载荷（uniffi 绑定的**语义**部分）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// 事件序号（单调递增）。
    pub seq: u64,
    /// 事件种类。
    pub kind: String,
    /// 事件载荷（**明文**，样例数据）。
    pub payload: String,
}

/// 打开一个样例会话（**返回结构体**的样例）。
pub fn open_session(key: Vec<u8>, peer_label: String) -> Result<SessionInfo, SpikeErrorKind> {
    let info = core_api::open_session(&key, &peer_label)?;
    Ok(info.into())
}

/// 把能力位掩码翻译成能力名列表（**返回 `Result`** 的样例）。
pub fn capability_names(mask: u32) -> Result<Vec<String>, SpikeErrorKind> {
    let names = core_api::capability_names(mask)?;
    Ok(names)
}

/// 事件源（uniffi 回调的**语义**部分）。
///
/// ⚠️ 这就是 uniffi 回调的**真实成本**所在：
/// 移动端实现的对象必须满足 `Send + Sync`，
/// 因为 uniffi 生成的代理允许 Rust 侧**从任意线程**回调。
pub trait EventSink: Send + Sync {
    /// 收到一个事件。
    fn on_event(&self, event: Event);
}

/// 触发一串样例事件，通过**回调**回传给移动端。
///
/// 返回值：成功回调的次数。
pub fn emit_events(count: u32, sink: &dyn EventSink) -> u32 {
    let events = core_api::sample_events(count);
    let mut delivered = 0;
    for event in events {
        sink.on_event(Event {
            seq: event.seq,
            kind: event.kind,
            payload: event.payload,
        });
        delivered += 1;
    }
    delivered
}

/// 版本号：让移动端能确认加载到的是哪一版绑定。
pub fn binding_version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}
