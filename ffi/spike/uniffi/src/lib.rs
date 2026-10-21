//! FFI spike：**uniffi** 方案（**候选**）的**宏层**。
//!
//! # 这个样例要暴露什么
//!
//! uniffi 的真实成本不在「声明一个函数」这一行，而在**工具链**与**边界语义**：
//!
//! 1. **代码生成**：需要 `uniffi-bindgen` CLI（额外工具），
//!    且生成物要**随版本提交或随构建产出** ⇒ 移动端构建链多一步。
//! 2. **错误传递**：`Result<T, E>` 自动映射成 Swift `throws` / Kotlin **异常**，
//!    `E` 必须是 uniffi 能表达的类型（枚举/字符串）。
//! 3. **回调**：uniffi 有**回调接口**（CallbackInterface）机制；
//!    Rust 调 Swift/Kotlin 的回调是「foreign trait object」⇒ 有线程限制。
//! 4. **panic**：uniffi 在 FFI 边界**捕获 panic**（需要 `panic = "unwind"`）。
//!
//! # 分层
//!
//! - [`api`]：**纯 Rust** 的语义层（无宏）⇒ 可被 `ffi/tests/` 直接对照测试。
//! - 本文件：uniffi 的**宏层**（`#[derive]` / `#[uniffi::export]`），
//!   只负责「把 `api` 的东西暴露成 uniffi 类型」。
//!
//! # ⛔ 本 crate 不进最终产物
//!
//! `ffi/spike/` 是隔离区；选型定稿后由后续卡删掉落选的一份。
//!
//! # ⛔ 依赖
//!
//! 本 crate 引入 `uniffi`（0.32.2）——这是**新增依赖**，
//! 按卡面要求写进报告等主理人拍板，**不**擅自加进 `ffi/` 主体 crate。

use std::sync::atomic::{AtomicU32, Ordering};

use uniffi::{Enum, Record};

/// 样例核心（**共享源码**，非独立 crate）。见 `cabi` 的同名说明。
#[path = "../../core/core.rs"]
pub mod core;

pub mod api;

use api::{SessionInfo, SpikeErrorKind};

/// 计数器：用于演示「Rust 侧状态」与线程行为。
static CALL_COUNTER: AtomicU32 = AtomicU32::new(0);

/// 跨语言错误类型：uniffi 的**富错误**（Rich Error）。
///
/// ⚠️ uniffi 的错误**必须**是枚举（`Enum` 派生）或字符串，
/// 且变体里只能带 uniffi 能表达的字段。
/// 本例只带**长度/掩码**，**不带密钥字节** ⇒ 跨语言后也不泄露。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Enum)]
pub enum SpikeError {
    /// 密钥长度非法（**只报长度**）。
    #[error("密钥长度非法：实际 {actual} 字节，期望 {expected} 字节")]
    KeyLength { actual: u32, expected: u32 },

    /// 会话尚未握手。
    #[error("会话尚未握手")]
    NotHandshaked,

    /// 能力位掩码非法。
    #[error("能力位掩码非法：0x{mask:08x} 含有未知位")]
    InvalidCapability { mask: u32 },

    /// 对端标签非法。
    #[error("对端标签非法：长度 {actual} 字节，允许 1..={expected}")]
    InvalidPeerLabel { actual: u32, expected: u32 },
}

impl From<SpikeErrorKind> for SpikeError {
    fn from(err: SpikeErrorKind) -> Self {
        match err {
            SpikeErrorKind::KeyLength { actual, expected } => Self::KeyLength { actual, expected },
            SpikeErrorKind::NotHandshaked => Self::NotHandshaked,
            SpikeErrorKind::InvalidCapability { mask } => Self::InvalidCapability { mask },
            SpikeErrorKind::InvalidPeerLabel { actual, expected } => {
                Self::InvalidPeerLabel { actual, expected }
            }
        }
    }
}

/// 跨语言结构体：**返回结构体**的样例。
///
/// ⚠️ 与手写 C ABI 的对照点：
/// 这里用 `Record`（uniffi 记录类型）⇒ 生成的 Swift/Kotlin 是**具名字段**，
/// **加字段是兼容的**（不像 `#[repr(C)]` 那样要求两侧逐字对齐）。
#[derive(Debug, Clone, PartialEq, Eq, Record)]
pub struct SpikeSessionInfo {
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

impl From<SessionInfo> for SpikeSessionInfo {
    fn from(info: SessionInfo) -> Self {
        Self {
            epoch: info.epoch,
            capabilities: info.capabilities,
            fingerprint: info.fingerprint,
            session_id: info.session_id,
            peer_label: info.peer_label,
        }
    }
}

/// 事件回调载荷：**回调**的样例。
#[derive(Debug, Clone, PartialEq, Eq, Record)]
pub struct SpikeEvent {
    /// 事件序号（单调递增）。
    pub seq: u64,
    /// 事件种类。
    pub kind: String,
    /// 事件载荷（**明文**，样例数据）。
    pub payload: String,
}

/// **回调接口**（uniffi 的 foreign trait 机制）：移动端实现它，Rust 调它。
///
/// ⚠️ 关键限制（实测记录在 `docs/ffi-spike.md` 测项 9）：
/// - 回调对象在 Swift/Kotlin 侧是**一个类/对象**，Rust 侧持有它的**代理**；
/// - 代理要求 `Send + Sync` ⇒ 移动端实现方**必须**自己处理线程安全；
/// - 代理的**生命周期**由移动端持有；Rust 侧 drop 代理后回调不再到达。
#[uniffi::export(callback_interface)]
pub trait SpikeEventSink: Send + Sync {
    /// 收到一个事件。
    fn on_event(&self, event: SpikeEvent);
}

/// 打开一个样例会话（**返回结构体**的样例）。
///
/// 失败时返回 [`SpikeError`] ⇒ Swift 侧是 `throws`，Kotlin 侧是**异常**。
#[uniffi::export]
pub fn open_session(key: Vec<u8>, peer_label: String) -> Result<SpikeSessionInfo, SpikeError> {
    let info = api::open_session(key, peer_label)?;
    Ok(info.into())
}

/// 把能力位掩码翻译成能力名列表（**返回 `Result`** 的样例）。
#[uniffi::export]
pub fn capability_names(mask: u32) -> Result<Vec<String>, SpikeError> {
    let names = api::capability_names(mask)?;
    Ok(names)
}

/// 触发一串样例事件，通过**回调接口**回传给移动端。
///
/// 返回值：成功回调的次数。
#[uniffi::export]
pub fn emit_events(count: u32, sink: Box<dyn SpikeEventSink>) -> u32 {
    let events = core::sample_events(count);
    let mut delivered = 0;
    for event in events {
        sink.on_event(SpikeEvent {
            seq: event.seq,
            kind: event.kind,
            payload: event.payload,
        });
        delivered += 1;
    }
    delivered
}

/// 回调次数计数：用来验证「回调确实到达了移动端」。
#[uniffi::export]
pub fn callback_count() -> u32 {
    CALL_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// 版本号：让移动端能确认加载到的是哪一版绑定。
#[uniffi::export]
pub fn binding_version() -> String {
    api::binding_version()
}

// 生成 uniffi 的 scaffolding 符号（`ffi_*` 导出）。
//
// ⛔ 这一行是 uniffi 方案的**硬性构建链要求**：
// 移动端侧必须链接这份 scaffolding 产物。
uniffi::setup_scaffolding!("spike");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_session_succeeds() {
        let key: Vec<u8> = (0u8..32).collect();
        let info = open_session(key, "peer-a".to_owned()).expect("应当成功");
        assert_eq!(info.peer_label, "peer-a");
        assert_eq!(info.epoch, 1);
    }

    #[test]
    fn open_session_maps_error() {
        let err = open_session(vec![0u8; 8], "peer-a".to_owned()).expect_err("应当失败");
        assert_eq!(
            err,
            SpikeError::KeyLength {
                actual: 8,
                expected: 32
            }
        );
        // 错误文本不得包含密钥字节
        assert!(!err.to_string().contains('\u{1}'));
    }

    #[test]
    fn capability_names_rejects_unknown_bits() {
        let err = capability_names(1 << 20).expect_err("应当失败");
        assert_eq!(err, SpikeError::InvalidCapability { mask: 1 << 20 });
    }

    #[test]
    fn capability_names_lists_all() {
        let names = capability_names(core::capability::ALL_DEFINED).expect("应当成功");
        assert_eq!(names.len(), 4);
        assert_eq!(names[0], "FRAME_CODEC");
    }

    #[test]
    fn version_is_not_empty() {
        assert!(!binding_version().is_empty());
    }
}
