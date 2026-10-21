//! FFI spike：**绑定一致性**断言 —— 选型的核心判据。
//!
//! # 这个文件测什么
//!
//! 同一组输入，分别经过两种绑定的**入口**（uniffi 导出 / 手写 C ABI），
//! **行为必须一致**：
//!
//! - 成功路径：结构体字段、会话标识、指纹、世代号、能力掩码**逐字段相同**；
//! - 错误路径：两种绑定的**错误码/错误种类**映射到**同一个语义错误**；
//! - 错误信息**不泄露密钥字节**（沿用 `lengsuo-wireguard` 的 `TransportKey` 纪律）。
//!
//! # 怎么做到「同一份实现，两种绑定」
//!
//! 两种绑定的**薄包装**都只调用 `ffi/spike/core/core.rs` 的同一批函数
//! ⇒ 本文件把两个绑定层与核心**一起**用 `#[path]` 引入，
//! 对**两个不同的绑定入口**做对照
//! （而不是对同一条路径做两次调用 —— 那样测不出差异）。
//!
//! ⚠️ `ffi/Cargo.toml` 与 `ffi/Cargo.lock` 属**共享区**（本卡不许动），
//! 所以这里**不引入** `uniffi` 之类的新依赖：
//! uniffi 的**语义层**（`api.rs`，纯 Rust）被对照测试，
//! uniffi 的**宏层**（`lib.rs`）由 `ffi/spike/` 自己的 workspace 测。
//!
//! ⚠️ `ffi/spike/` 是隔离区 ⇒ 这些测试**不进最终产物**。

#[path = "../spike/cabi/src/lib.rs"]
mod cabi;

#[path = "../spike/uniffi/src/api.rs"]
mod uniffi_api;

#[path = "../spike/vectors/golden.rs"]
mod golden;

use std::ffi::{CStr, CString};

use golden::{CAPABILITY_ALL, CAPABILITY_NAMES, CASES, ERROR_CASES};

/// 共享核心。
///
/// ⚠️ 直接复用 `cabi` 里**已经引入**的那一份（`cabi::core`），
///    不要再 `#[path]` 引一次 —— 同一文件被当作两个模块加载
///    会触发 `clippy::duplicate_mod`，而且两份副本会各跑一遍单测。
use cabi::core;

// ---------------------------------------------------------------- C ABI 适配器

/// 手写 C ABI 的调用结果（**完整走一遍 C 的调用形状**）。
struct CAbiCall {
    rc: i32,
    epoch: u64,
    capabilities: u32,
    fingerprint: u64,
    session_id: String,
    peer_label: String,
    error_code: i32,
    error_message: String,
}

/// 用手写 C ABI 打开会话。
fn call_cabi_open_session(key: &[u8], peer_label: &str) -> CAbiCall {
    let label = CString::new(peer_label).expect("标签不含 NUL");
    let mut info = cabi::SpikeSessionInfo {
        epoch: 0,
        capabilities: 0,
        fingerprint: 0,
        session_id: std::ptr::null_mut(),
        peer_label: std::ptr::null_mut(),
    };
    let mut error = cabi::SpikeError {
        code: cabi::SpikeErrorCode::Unknown,
        message: std::ptr::null_mut(),
    };

    // SAFETY: 全部指针有效；key 有 key.len() 个可读字节；label 是 NUL 结尾字符串。
    let rc = unsafe {
        cabi::spike_open_session(
            key.as_ptr(),
            key.len(),
            label.as_ptr(),
            &mut info,
            &mut error,
        )
    };

    // SAFETY: error.message 由本库分配（失败路径），可读一次。
    let error_message = if error.message.is_null() {
        String::new()
    } else {
        // SAFETY: 非空且是有效的 NUL 结尾字符串。
        let text = unsafe { CStr::from_ptr(error.message) }
            .to_string_lossy()
            .into_owned();
        // SAFETY: 由本库分配，释放一次。
        unsafe { cabi::spike_free_string(error.message) };
        text
    };

    let mut out = CAbiCall {
        rc,
        epoch: info.epoch,
        capabilities: info.capabilities,
        fingerprint: info.fingerprint,
        session_id: String::new(),
        peer_label: String::new(),
        error_code: error.code as i32,
        error_message,
    };

    if rc == 0 {
        // SAFETY: 成功路径下两个指针都由本库分配。
        out.session_id = if info.session_id.is_null() {
            String::new()
        } else {
            // SAFETY: 有效的 NUL 结尾字符串。
            unsafe { CStr::from_ptr(info.session_id) }
                .to_string_lossy()
                .into_owned()
        };
        out.peer_label = if info.peer_label.is_null() {
            String::new()
        } else {
            // SAFETY: 有效的 NUL 结尾字符串。
            unsafe { CStr::from_ptr(info.peer_label) }
                .to_string_lossy()
                .into_owned()
        };
        // SAFETY: 释放本库分配的两个字符串（结构体本身是栈上内存，不释放）。
        unsafe { cabi::spike_free_session_info(&mut info) };
    }

    out
}

/// 用手写 C ABI 取能力名列表。
fn call_cabi_capability_names(mask: u32) -> Result<Vec<String>, (i32, String)> {
    let count = cabi::spike_capability_name_count(mask);
    let mut names: Vec<*mut std::ffi::c_char> = vec![std::ptr::null_mut(); count.max(1)];
    let mut actual = 0usize;
    let mut error = cabi::SpikeError {
        code: cabi::SpikeErrorCode::Unknown,
        message: std::ptr::null_mut(),
    };
    // SAFETY: names 有 count.max(1) 个可写槽位；out_count/out_error 有效。
    let rc =
        unsafe { cabi::spike_capability_names(mask, names.as_mut_ptr(), &mut actual, &mut error) };
    if rc != 0 {
        // SAFETY: 错误消息由本库分配。
        let message = if error.message.is_null() {
            String::new()
        } else {
            // SAFETY: 有效的 NUL 结尾字符串。
            let text = unsafe { CStr::from_ptr(error.message) }
                .to_string_lossy()
                .into_owned();
            // SAFETY: 释放一次。
            unsafe { cabi::spike_free_string(error.message) };
            text
        };
        return Err((error.code as i32, message));
    }
    let result: Vec<String> = names
        .iter()
        .take(actual)
        .map(|ptr| {
            if ptr.is_null() {
                String::new()
            } else {
                // SAFETY: 有效的 NUL 结尾字符串。
                unsafe { CStr::from_ptr(*ptr) }
                    .to_string_lossy()
                    .into_owned()
            }
        })
        .collect();
    // SAFETY: names 有 actual 个由本库分配的字符串。
    unsafe { cabi::spike_free_string_array(names.as_mut_ptr(), actual) };
    Ok(result)
}

// ---------------------------------------------------------------- 断言

#[test]
fn both_bindings_return_identical_session_info_for_golden_cases() {
    for case in CASES.iter() {
        let cabi_out = call_cabi_open_session(&case.key, case.peer_label);
        let uniffi_out = uniffi_api::open_session(case.key.to_vec(), case.peer_label.to_owned())
            .expect("uniffi 侧应当成功");

        assert_eq!(cabi_out.rc, 0, "C ABI 侧应当成功：{}", case.name);
        assert_eq!(
            cabi_out.epoch, uniffi_out.epoch,
            "世代号不一致：{}",
            case.name
        );
        assert_eq!(
            cabi_out.capabilities, uniffi_out.capabilities,
            "能力掩码不一致：{}",
            case.name
        );
        assert_eq!(
            cabi_out.fingerprint, uniffi_out.fingerprint,
            "指纹不一致：{}",
            case.name
        );
        assert_eq!(
            cabi_out.session_id, uniffi_out.session_id,
            "会话标识不一致：{}",
            case.name
        );
        assert_eq!(
            cabi_out.peer_label, uniffi_out.peer_label,
            "对端标签不一致：{}",
            case.name
        );

        // 还要与**向量**一致（防止两边一起跑偏）。
        assert_eq!(
            cabi_out.epoch, case.epoch,
            "世代号与向量不一致：{}",
            case.name
        );
        assert_eq!(
            cabi_out.capabilities, case.capabilities,
            "能力掩码与向量不一致：{}",
            case.name
        );
        assert_eq!(
            cabi_out.fingerprint, case.fingerprint,
            "指纹与向量不一致：{}",
            case.name
        );
        assert_eq!(
            cabi_out.session_id, case.session_id,
            "会话标识与向量不一致：{}",
            case.name
        );
    }
}

#[test]
fn both_bindings_report_the_same_error_kind_for_error_cases() {
    for case in ERROR_CASES.iter() {
        // ⚠️ 每个向量都必须在**两侧走同一个入口** ⇒ 错误码才可比。
        //    `unknown-capability-bit` 走的是「能力名」接口，不是「开会话」接口。
        let (cabi_code, uniffi_err) = if case.name == "unknown-capability-bit" {
            let cabi_err = call_cabi_capability_names(1 << 20).expect_err("C ABI 侧应当失败");
            let uniffi_err = uniffi_api::capability_names(1 << 20).expect_err("uniffi 侧应当失败");
            (cabi_err.0, uniffi_err)
        } else {
            let cabi_out = call_cabi_open_session(case.key_bytes(), case.peer_label);
            assert_ne!(cabi_out.rc, 0, "C ABI 侧应当失败：{}", case.name);
            assert_eq!(
                cabi_out.error_code, case.error_code,
                "C ABI 错误码与向量不一致：{}",
                case.name
            );
            let uniffi_err =
                uniffi_api::open_session(case.key_bytes().to_vec(), case.peer_label.to_owned())
                    .expect_err("uniffi 侧应当失败");
            (cabi_out.error_code, uniffi_err)
        };
        let uniffi_code = match uniffi_err {
            uniffi_api::SpikeErrorKind::KeyLength { .. } => 1,
            uniffi_api::SpikeErrorKind::NotHandshaked => 2,
            uniffi_api::SpikeErrorKind::InvalidCapability { .. } => 3,
            uniffi_api::SpikeErrorKind::InvalidPeerLabel { .. } => 4,
        };
        assert_eq!(
            uniffi_code, cabi_code,
            "两种绑定的错误码不一致：{}",
            case.name
        );
        assert_eq!(
            uniffi_code, case.error_code,
            "错误码与向量不一致：{}",
            case.name
        );
    }
}

#[test]
fn both_bindings_list_the_same_capability_names() {
    let cabi_names = call_cabi_capability_names(CAPABILITY_ALL).expect("C ABI 侧应当成功");
    let uniffi_names = uniffi_api::capability_names(CAPABILITY_ALL).expect("uniffi 侧应当成功");

    assert_eq!(cabi_names, uniffi_names, "两侧能力名列表不一致");
    assert_eq!(cabi_names, CAPABILITY_NAMES, "能力名与向量不一致");
}

#[test]
fn both_bindings_reject_unknown_capability_bits_with_same_code() {
    let cabi_err = call_cabi_capability_names(1 << 20).expect_err("C ABI 侧应当失败");
    let uniffi_err = uniffi_api::capability_names(1 << 20).expect_err("uniffi 侧应当失败");
    assert_eq!(cabi_err.0, 3, "C ABI 侧错误码应为 InvalidCapability(3)");
    assert!(matches!(
        uniffi_err,
        uniffi_api::SpikeErrorKind::InvalidCapability { mask } if mask == 1 << 20
    ));
}

#[test]
fn error_messages_never_leak_key_bytes() {
    for case in ERROR_CASES.iter() {
        let cabi_out = call_cabi_open_session(case.key_bytes(), case.peer_label);
        // ⛔ 错误消息里**不得**出现密钥的可打印字节。
        for byte in case.key_bytes() {
            if byte.is_ascii_graphic() {
                assert!(
                    !cabi_out.error_message.contains(*byte as char),
                    "C ABI 错误消息泄露了密钥字节：{}",
                    case.name
                );
            }
        }
        // uniffi 侧同理。
        let uniffi_err = if case.name == "unknown-capability-bit" {
            uniffi_api::capability_names(1 << 20).expect_err("uniffi 侧应当失败")
        } else {
            uniffi_api::open_session(case.key_bytes().to_vec(), case.peer_label.to_owned())
                .expect_err("uniffi 侧应当失败")
        };
        let text = uniffi_err.to_string();
        for byte in case.key_bytes() {
            if byte.is_ascii_graphic() {
                assert!(
                    !text.contains(*byte as char),
                    "uniffi 错误消息泄露了密钥字节：{}",
                    case.name
                );
            }
        }
    }
}

#[test]
fn core_error_debug_never_prints_key_bytes() {
    let key = core::SampleKey::new(&CASES[0].key).expect("应当成功");
    let debug = format!("{key:?}");
    assert!(debug.contains("len"), "Debug 必须给出长度");
    assert!(debug.contains("32"), "Debug 必须给出长度值");
    assert!(!debug.contains('['), "Debug 不得打印字节数组");
}

#[test]
fn both_bindings_emit_the_same_event_sequence() {
    // 手写 C ABI 的回调：收集 (seq, kind, payload)。
    use std::sync::Mutex;
    static C_EVENTS: Mutex<Vec<(u64, String, String)>> = Mutex::new(Vec::new());

    extern "C" fn record(
        seq: u64,
        kind: *const std::ffi::c_char,
        payload: *const std::ffi::c_char,
    ) {
        // SAFETY: kind/payload 由 Rust 侧保证是有效的 NUL 结尾字符串。
        let kind_text = unsafe { CStr::from_ptr(kind) }
            .to_string_lossy()
            .into_owned();
        let payload_text = unsafe { CStr::from_ptr(payload) }
            .to_string_lossy()
            .into_owned();
        if let Ok(mut guard) = C_EVENTS.lock() {
            guard.push((seq, kind_text, payload_text));
        }
    }

    // SAFETY: record 是有效的函数指针。
    let delivered = unsafe { cabi::spike_emit_events(golden::EVENT_COUNT, record) };
    assert_eq!(delivered as u32, golden::EVENT_COUNT, "C ABI 回调次数不对");

    let cabi_events = C_EVENTS.lock().expect("锁可用").clone();

    // uniffi 侧的回调：用一个测试用的 sink。
    struct TestSink {
        events: std::sync::Mutex<Vec<(u64, String, String)>>,
    }
    impl uniffi_api::EventSink for TestSink {
        fn on_event(&self, event: uniffi_api::Event) {
            if let Ok(mut guard) = self.events.lock() {
                guard.push((event.seq, event.kind, event.payload));
            }
        }
    }
    let sink = TestSink {
        events: std::sync::Mutex::new(Vec::new()),
    };
    let uniffi_delivered = uniffi_api::emit_events(golden::EVENT_COUNT, &sink);
    let uniffi_events = sink.events.lock().expect("锁可用").clone();

    assert_eq!(uniffi_delivered, golden::EVENT_COUNT, "uniffi 回调次数不对");
    assert_eq!(cabi_events, uniffi_events, "两种绑定的回调序列不一致");

    // 还要与向量一致。
    for (i, (seq, _, payload)) in cabi_events.iter().enumerate() {
        assert_eq!(*seq, golden::EVENT_SEQS[i], "事件序号与向量不一致");
        assert_eq!(payload, golden::EVENT_PAYLOADS[i], "事件载荷与向量不一致");
    }
}
