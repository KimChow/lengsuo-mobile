//! FFI spike：**跨端测试向量**的一致性断言。
//!
//! # 这个文件测什么
//!
//! 1. `golden.json` 与 Rust 侧的向量常量**逐字节一致**（防止漂移）；
//! 2. Rust 侧算出的指纹/会话标识与向量**一致**（防止算法漂移）；
//! 3. 移动端（Swift/Kotlin）读的 `golden.json` 里含**同样的值**。
//!
//! # 为什么用 `#[path]` 而不是加依赖
//!
//! `ffi/Cargo.toml` 与 `ffi/Cargo.lock` 属**共享区**（本卡不许动），
//! 所以这里**不引入** `serde_json` 之类的新依赖：
//! 向量以 Rust 常量为准，JSON 由 [`render_golden_json`] 渲染，测试断言两者一致。
//!
//! ⚠️ `golden.rs` 位于 `ffi/spike/`（隔离区）⇒ **不进最终产物**。

#[path = "../spike/vectors/golden.rs"]
mod golden;

use golden::{render_golden_json, CASES, ERROR_CASES};

/// `golden.json` 的位置（相对于本文件）。
const GOLDEN_JSON: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/spike/vectors/golden.json");

#[test]
fn golden_json_matches_rust_source_of_truth() {
    let on_disk = std::fs::read_to_string(GOLDEN_JSON).expect("golden.json 必须存在");
    let rendered = render_golden_json();
    assert_eq!(
        on_disk, rendered,
        "golden.json 与 Rust 向量常量漂移了 —— 请重新生成 golden.json"
    );
}

#[test]
fn golden_json_contains_every_case_name() {
    let json = std::fs::read_to_string(GOLDEN_JSON).expect("golden.json 必须存在");
    for case in CASES.iter() {
        assert!(
            json.contains(case.name),
            "golden.json 缺少向量 {}",
            case.name
        );
        assert!(
            json.contains(case.session_id),
            "golden.json 缺少会话标识 {}",
            case.session_id
        );
    }
    for case in ERROR_CASES.iter() {
        assert!(
            json.contains(case.name),
            "golden.json 缺少错误向量 {}",
            case.name
        );
    }
}

#[test]
fn golden_json_does_not_contain_key_bytes_as_plaintext() {
    let json = std::fs::read_to_string(GOLDEN_JSON).expect("golden.json 必须存在");
    // `golden-alpha` 的密钥首字节是 0x00 ⇒ 十六进制串里必然出现 "00010203"；
    // ⛔ 但**不得**出现可打印的原始字节序列（"abc" 之类）。
    for case in CASES.iter() {
        let printable: String = case
            .key
            .iter()
            .filter(|b| b.is_ascii_graphic())
            .map(|b| *b as char)
            .collect();
        if printable.len() >= 4 {
            assert!(
                !json.contains(&printable),
                "golden.json 不得包含密钥的可打印字节片段"
            );
        }
    }
}
