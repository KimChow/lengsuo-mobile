//! FFI spike：**手写 C ABI** 方案（**对照**）—— `extern "C"` + 手写内存/错误/回调管理。
//!
//! # 这个样例要暴露什么
//!
//! 手写 C ABI 的真实成本不在「写一个 `extern "C" fn`」，而在**边界上的四件事**：
//!
//! 1. **内存所有权**：谁分配、谁释放、释放几次 —— 全部手写；
//! 2. **结构体布局**：`#[repr(C)]` 的字段顺序/对齐要**逐字段**镜像到 C 头文件；
//! 3. **错误传递**：Rust `Result` 要**手工**降级成「返回码 + 出参结构体」；
//! 4. **回调**：Rust 调 Swift/Kotlin 要手写 `extern "C"` 函数指针，
//!    且 **C 函数指针不能捕获上下文** ⇒ 移动端要自己想办法带状态。
//!
//! # 与 uniffi 的对照
//!
//! uniffi 那边这四件事**全是自动的**（生成绑定 + 生成头文件 + 自动的错误映射 +
//! 回调接口）。代价是**多一个 codegen 工具链**和**多一批依赖**。
//! ⇒ 逐项数字见 `docs/ffi-spike.md`。
//!
//! # 纪律
//!
//! - 本 crate **需要** `unsafe`（手写 FFI 的本质）⇒ 用
//!   `#![deny(unsafe_op_in_unsafe_fn)]` 代替仓库默认的 `forbid(unsafe_code)`；
//!   这是一个**有意的例外**，已在 `docs/ffi-decision.md` 里说明。
//! - `panic = "abort"` 下，Rust 侧 panic 会直接 abort 进程（不 unwind 跨 FFI），
//!   所以本 crate **在 FFI 边界不 panic**（错误一律走返回码）。
//!
//! ⚠️ 本文件属于**隔离区**（`ffi/spike/`）⇒ **不进最终产物**。

#![deny(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_char, c_int, CStr, CString};

#[path = "../../core/core.rs"]
pub mod core;

use self::core as core_api;
use self::core::{SampleError, SessionInfo};

// ---------------------------------------------------------------- 错误码

/// 错误码：与 `cabi/include/lengsuo_spike_cabi.h` 里的 `SpikeErrorCode` **逐值对应**。
///
/// ⚠️ 手写 C ABI 的成本之一：这个映射是**手写**的，改了 Rust 侧忘了改 C 侧
///    ⇒ 移动端会解出错误的错误码（**静默**）。
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpikeErrorCode {
    /// 未知错误。
    Unknown = 0,
    /// 密钥长度非法（**只报长度**）。
    KeyLength = 1,
    /// 会话尚未握手。
    NotHandshaked = 2,
    /// 能力位掩码非法。
    InvalidCapability = 3,
    /// 对端标签非法。
    InvalidPeerLabel = 4,
}

impl From<&SampleError> for SpikeErrorCode {
    fn from(error: &SampleError) -> Self {
        match error {
            SampleError::KeyLength { .. } => Self::KeyLength,
            SampleError::NotHandshaked => Self::NotHandshaked,
            SampleError::InvalidCapability { .. } => Self::InvalidCapability,
            SampleError::InvalidPeerLabel { .. } => Self::InvalidPeerLabel,
        }
    }
}

/// 错误载荷：错误码 + 可为 NULL 的消息。
///
/// ⚠️ 布局必须与 C 头文件里的 `SpikeError` **逐字段对齐**。
#[repr(C)]
pub struct SpikeError {
    /// 错误码。
    pub code: SpikeErrorCode,
    /// 库分配的消息（NUL 结尾）；失败路径上由本库分配，用 `spike_free_string` 释放。
    pub message: *mut c_char,
}

// ---------------------------------------------------------------- 结构体

/// 会话信息：与 C 头文件里的 `SpikeSessionInfo` **逐字段对齐**。
///
/// ⚠️ 这里的字段是**裸指针**（不是 `String`）——因为要跨 FFI 边界。
///    `String` 的所有权不能直接交给 Swift/Kotlin。
#[repr(C)]
pub struct SpikeSessionInfo {
    /// 连接世代号。
    pub epoch: u64,
    /// 协商出的能力位掩码。
    pub capabilities: u32,
    /// 密钥指纹（公开）。
    pub fingerprint: u64,
    /// 会话标识（**非密钥**）；库分配，用 `spike_free_session_info` 释放。
    pub session_id: *mut c_char,
    /// 对端标签（**非密钥**）；库分配，用 `spike_free_session_info` 释放。
    pub peer_label: *mut c_char,
}

/// 把 Rust 侧的 [`SessionInfo`] 填进 C 形状的 [`SpikeSessionInfo`]。
fn fill_session_info(target: &mut SpikeSessionInfo, source: &SessionInfo) {
    target.epoch = source.epoch;
    target.capabilities = source.capabilities;
    target.fingerprint = source.fingerprint;
    target.session_id = to_raw_string(&source.session_id);
    target.peer_label = to_raw_string(&source.peer_label);
}

/// 把 `&str` 复制成库分配的 NUL 结尾字符串（所有权转移给调用方）。
fn to_raw_string(value: &str) -> *mut c_char {
    // ⚠️ 失败（内含 NUL）时退回空串 —— 绝**不** panic 跨 FFI 边界。
    let owned = CString::new(value).unwrap_or_else(|_| CString::new("").expect("空串不含 NUL"));
    owned.into_raw()
}

/// 填充错误载荷（错误码 + 消息）。
///
/// ⛔ 消息里**只带长度/掩码**，绝不带密钥字节（沿用 `TransportKey` 纪律）。
fn fill_error(target: &mut SpikeError, error: &SampleError) {
    target.code = SpikeErrorCode::from(error);
    let text = match error {
        SampleError::KeyLength { actual, expected } => {
            format!("密钥长度非法：实际 {actual}，期望 {expected}")
        }
        SampleError::NotHandshaked => "会话尚未握手".to_owned(),
        SampleError::InvalidCapability { mask } => {
            format!("能力位掩码非法：0x{mask:08x}")
        }
        SampleError::InvalidPeerLabel { actual, expected } => {
            format!("对端标签非法：实际 {actual}，期望 {expected}")
        }
    };
    target.message = to_raw_string(&text);
}

// ---------------------------------------------------------------- 生命周期

/// 释放一个由库分配的字符串。
///
/// # Safety
///
/// `ptr` 必须来自本库的 `to_raw_string`（即 `spike_open_session` 等函数返回的
/// 字符串），且**未被释放过**。
#[no_mangle]
pub unsafe extern "C" fn spike_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    // SAFETY: 调用方保证 `ptr` 来自本库的 `CString::into_raw`。
    drop(unsafe { CString::from_raw(ptr) });
}

/// 释放 [`SpikeSessionInfo`] 内部的两个字符串（**不**释放结构体本身）。
///
/// ⚠️ 结构体是**调用方的内存**（栈上/堆上由调用方持有）⇒ 这里只释放内部字符串。
///
/// # Safety
///
/// `ptr` 必须指向有效的 [`SpikeSessionInfo`]。
#[no_mangle]
pub unsafe extern "C" fn spike_free_session_info(ptr: *mut SpikeSessionInfo) {
    if ptr.is_null() {
        return;
    }
    // SAFETY: 调用方保证 `ptr` 有效。
    let info = unsafe { &mut *ptr };
    if !info.session_id.is_null() {
        // SAFETY: 字符串由本库分配，且只释放一次。
        unsafe { spike_free_string(info.session_id) };
        info.session_id = std::ptr::null_mut();
    }
    if !info.peer_label.is_null() {
        // SAFETY: 字符串由本库分配，且只释放一次。
        unsafe { spike_free_string(info.peer_label) };
        info.peer_label = std::ptr::null_mut();
    }
}

/// 释放能力名数组。
///
/// # Safety
///
/// `names` 必须是本库 `spike_capability_names` 填好的数组，
/// `count` 必须是**实际**填入的个数。
#[no_mangle]
pub unsafe extern "C" fn spike_free_string_array(names: *mut *mut c_char, count: usize) {
    if names.is_null() {
        return;
    }
    for index in 0..count {
        // SAFETY: 调用方保证数组至少有 `count` 个可读槽位。
        let slot = unsafe { &mut *names.add(index) };
        if !slot.is_null() {
            // SAFETY: 字符串由本库分配，且只释放一次。
            unsafe { spike_free_string(*slot) };
            *slot = std::ptr::null_mut();
        }
    }
}

// ---------------------------------------------------------------- 业务入口

/// 打开样例会话。
///
/// # Safety
///
/// - `key` 必须指向 `key_len` 个可读字节；
/// - `peer_label` 必须是 NUL 结尾的字符串；
/// - `out_info` / `out_error` 必须指向有效的可写结构体。
#[no_mangle]
pub unsafe extern "C" fn spike_open_session(
    key: *const u8,
    key_len: usize,
    peer_label: *const c_char,
    out_info: *mut SpikeSessionInfo,
    out_error: *mut SpikeError,
) -> c_int {
    // SAFETY: 调用方保证 `key` 指向 `key_len` 个可读字节。
    let key_bytes = if key.is_null() {
        &[][..]
    } else {
        // SAFETY: 调用方保证 `key` 指向 `key_len` 个可读字节。
        unsafe { std::slice::from_raw_parts(key, key_len) }
    };
    // SAFETY: 调用方保证 `peer_label` 是 NUL 结尾字符串（可为空）。
    let label = if peer_label.is_null() {
        String::new()
    } else {
        // SAFETY: 调用方保证 NUL 结尾。
        unsafe { CStr::from_ptr(peer_label) }
            .to_string_lossy()
            .into_owned()
    };

    match core_api::open_session(key_bytes, &label) {
        Ok(info) => {
            if !out_info.is_null() {
                // SAFETY: 调用方保证 `out_info` 指向有效的可写结构体。
                let target = unsafe { &mut *out_info };
                fill_session_info(target, &info);
            }
            0
        }
        Err(error) => {
            if !out_error.is_null() {
                // SAFETY: 调用方保证 `out_error` 指向有效的可写结构体。
                let target = unsafe { &mut *out_error };
                fill_error(target, &error);
            }
            -1
        }
    }
}

/// 能力名数量（两段式的第一段）。
#[no_mangle]
pub extern "C" fn spike_capability_name_count(mask: u32) -> usize {
    core_api::capability_names(mask).map_or(0, |names| names.len())
}

/// 能力名列表（两段式的第二段）。
///
/// # Safety
///
/// - `out_names` 必须是长度 ≥ `spike_capability_name_count(mask)` 的可写指针数组；
/// - `out_count` 必须指向有效的可写 `usize`。
#[no_mangle]
pub unsafe extern "C" fn spike_capability_names(
    mask: u32,
    out_names: *mut *mut c_char,
    out_count: *mut usize,
    out_error: *mut SpikeError,
) -> c_int {
    match core_api::capability_names(mask) {
        Ok(names) => {
            if !out_names.is_null() {
                for (index, name) in names.iter().enumerate() {
                    // SAFETY: 调用方保证数组容量足够（= 名字个数）。
                    let slot = unsafe { &mut *out_names.add(index) };
                    *slot = to_raw_string(name);
                }
            }
            if !out_count.is_null() {
                // SAFETY: 调用方保证 `out_count` 指向有效的可写 `usize`。
                unsafe { *out_count = names.len() };
            }
            0
        }
        Err(error) => {
            if !out_error.is_null() {
                // SAFETY: 调用方保证 `out_error` 指向有效的可写结构体。
                let target = unsafe { &mut *out_error };
                fill_error(target, &error);
            }
            -1
        }
    }
}

/// 事件回调：Rust 调移动端。
///
/// ⚠️ **C 函数指针不能捕获上下文** ⇒ 移动端要么用全局变量，
///    要么在 Rust 侧用 `Box` + 裸指针把状态带过去。
pub type SpikeEventCallback = extern "C" fn(seq: u64, kind: *const c_char, payload: *const c_char);

/// 触发样例事件（同步回调，在调用线程上）。
///
/// # Safety
///
/// `callback` 必须是有效的函数指针。
#[no_mangle]
pub unsafe extern "C" fn spike_emit_events(count: u32, callback: SpikeEventCallback) -> c_int {
    let events = core_api::sample_events(count);
    for event in &events {
        let kind = CString::new(event.kind.as_str())
            .unwrap_or_else(|_| CString::new("").expect("空串不含 NUL"));
        let payload = CString::new(event.payload.as_str())
            .unwrap_or_else(|_| CString::new("").expect("空串不含 NUL"));
        // 调用 `callback`：调用方保证它是有效的函数指针；
        // `kind`/`payload` 是有效的 NUL 结尾字符串，且在回调返回前一直有效（回调是同步的）。
        callback(event.seq, kind.as_ptr(), payload.as_ptr());
    }
    events.len() as c_int
}

/// 绑定版本号（库分配，用 `spike_free_string` 释放）。
#[no_mangle]
pub extern "C" fn spike_binding_version() -> *mut c_char {
    to_raw_string(env!("CARGO_PKG_VERSION"))
}

// ---------------------------------------------------------------- 单测

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

// ---------------------------------------------------------------- 单测
