//! FFI spike：**手写 C ABI** 方案的单测。
//!
//! ⚠️ 拆成独立文件是为了让 `cabi/src/lib.rs` 保持在 400 行软目标内。
//!    测的是**边界形状**：错误码映射、消息不泄露密钥字节、两段式内存协议。

use super::*;
use crate::core as core_api;

/// 打开会话（成功路径）。
#[test]
fn open_session_returns_expected_fields() {
    let key = [0x11u8; 32];
    let label = CString::new("peer-a").expect("标签不含 NUL");
    let mut info = SpikeSessionInfo {
        epoch: 0,
        capabilities: 0,
        fingerprint: 0,
        session_id: std::ptr::null_mut(),
        peer_label: std::ptr::null_mut(),
    };
    let mut error = SpikeError {
        code: SpikeErrorCode::Unknown,
        message: std::ptr::null_mut(),
    };
    // SAFETY: 全部指针有效；key 有 32 个可读字节；label 是 NUL 结尾字符串。
    let rc = unsafe {
        spike_open_session(
            key.as_ptr(),
            key.len(),
            label.as_ptr(),
            &mut info,
            &mut error,
        )
    };
    assert_eq!(rc, 0, "应当成功");
    assert_eq!(info.epoch, 1);
    assert_eq!(info.capabilities, core_api::capability::ALL_DEFINED);
    // SAFETY: 成功路径下两个指针都由本库分配。
    let session_id = unsafe { CStr::from_ptr(info.session_id) }
        .to_string_lossy()
        .into_owned();
    assert!(
        session_id.starts_with("s-"),
        "会话标识格式不对：{session_id}"
    );
    // SAFETY: 释放本库分配的两个字符串。
    unsafe { spike_free_session_info(&mut info) };
}

/// 错误路径：短密钥 ⇒ 错误码 1，且消息**不含密钥字节**。
#[test]
fn open_session_reports_key_length_error() {
    let key = [0x01u8, 0x02, 0x03, 0x04];
    let label = CString::new("peer-a").expect("标签不含 NUL");
    let mut info = SpikeSessionInfo {
        epoch: 0,
        capabilities: 0,
        fingerprint: 0,
        session_id: std::ptr::null_mut(),
        peer_label: std::ptr::null_mut(),
    };
    let mut error = SpikeError {
        code: SpikeErrorCode::Unknown,
        message: std::ptr::null_mut(),
    };
    // SAFETY: 全部指针有效；key 有 4 个可读字节；label 是 NUL 结尾字符串。
    let rc = unsafe {
        spike_open_session(
            key.as_ptr(),
            key.len(),
            label.as_ptr(),
            &mut info,
            &mut error,
        )
    };
    assert_ne!(rc, 0, "短密钥应当失败");
    assert_eq!(error.code, SpikeErrorCode::KeyLength);
    // SAFETY: 错误消息由本库分配。
    let message = unsafe { CStr::from_ptr(error.message) }
        .to_string_lossy()
        .into_owned();
    assert!(
        !message.contains('\u{1}') && !message.contains('\u{2}'),
        "错误消息泄露了密钥字节：{message}"
    );
    // SAFETY: 释放本库分配的消息。
    unsafe { spike_free_string(error.message) };
}

/// 错误码映射覆盖全部变体。
#[test]
fn error_code_mapping_covers_all_variants() {
    assert_eq!(
        SpikeErrorCode::from(&SampleError::KeyLength {
            actual: 4,
            expected: 32
        }),
        SpikeErrorCode::KeyLength
    );
    assert_eq!(
        SpikeErrorCode::from(&SampleError::NotHandshaked),
        SpikeErrorCode::NotHandshaked
    );
    assert_eq!(
        SpikeErrorCode::from(&SampleError::InvalidCapability { mask: 1 }),
        SpikeErrorCode::InvalidCapability
    );
    assert_eq!(
        SpikeErrorCode::from(&SampleError::InvalidPeerLabel {
            actual: 0,
            expected: 64
        }),
        SpikeErrorCode::InvalidPeerLabel
    );
}

/// 两段式：先问数量、再取名字。
#[test]
fn capability_names_two_phase_protocol() {
    let count = spike_capability_name_count(0b1111);
    assert_eq!(count, 4);
    let mut names: Vec<*mut c_char> = vec![std::ptr::null_mut(); count];
    let mut actual = 0usize;
    let mut error = SpikeError {
        code: SpikeErrorCode::Unknown,
        message: std::ptr::null_mut(),
    };
    // SAFETY: `names` 有 count 个可写槽位；out_count/out_error 有效。
    let rc = unsafe { spike_capability_names(0b1111, names.as_mut_ptr(), &mut actual, &mut error) };
    assert_eq!(rc, 0);
    assert_eq!(actual, count);
    let collected: Vec<String> = names
        .iter()
        .take(actual)
        .map(|ptr| {
            // SAFETY: 有效的 NUL 结尾字符串。
            unsafe { CStr::from_ptr(*ptr) }
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(collected[0], "FRAME_CODEC");
    assert_eq!(collected[3], "EVENT_REPLAY");
    // SAFETY: names 有 actual 个由本库分配的字符串。
    unsafe { spike_free_string_array(names.as_mut_ptr(), actual) };
}

/// 未知能力位 ⇒ 错误码 3。
#[test]
fn capability_names_rejects_unknown_bits() {
    let mut names: Vec<*mut c_char> = vec![std::ptr::null_mut(); 1];
    let mut actual = 0usize;
    let mut error = SpikeError {
        code: SpikeErrorCode::Unknown,
        message: std::ptr::null_mut(),
    };
    // SAFETY: names 有 1 个可写槽位；out_count/out_error 有效。
    let rc =
        unsafe { spike_capability_names(1 << 20, names.as_mut_ptr(), &mut actual, &mut error) };
    assert_ne!(rc, 0, "未知能力位应当失败");
    assert_eq!(error.code, SpikeErrorCode::InvalidCapability);
    // SAFETY: 释放本库分配的消息。
    unsafe { spike_free_string(error.message) };
}
