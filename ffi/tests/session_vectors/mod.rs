//! 跨端 **L2 会话向量**的读取端（模块入口）。
//!
//! 这个目录下的文件不会被 Cargo 当作独立测试目标；
//! `cross_end_session_vectors.rs` 用 `mod session_vectors;` 引入。
//! 数据文件 `session_vectors.json` 由 Cargo 在编译期经 `include_str!` 内嵌，
//! 运行时不再依赖工作目录。
//!
//! # 分工
//!
//! - [`json`]：复用 `tests/vectors/json.rs`（`mb-ffi-vectors` 交付的极小
//!   JSON DOM 解析器 + 规范化渲染器）——**不重复写第二份**；
//! - [`model`]：向量数据模型 + 解析（`links[]` / `rounds[]` / `frames[]`）；
//! - [`util`]：十六进制、区间、帧类型名等机械换算工具；
//! - [`runner`]：用**移动端门面层**跑握手 + 封帧，产出「应得的字节」。
//!
//! # 来源与不变式
//!
//! 见同目录 [`SOURCE.md`]：向量来自公开仓 `KimChow/lengsuo-network` 的
//! `crates/lengsuo-link-core/tests/vectors/session_data.rs`，revision 与
//! `ffi/Cargo.toml` 里 pin 的 rev **逐字符相同**。

#![allow(dead_code)] // 每个测试二进制只用到其中一部分工具。

#[path = "../vectors/json.rs"]
pub mod json;

pub mod model;
pub mod runner;
pub mod util;

pub use model::{
    parse_links, Direction, DIR_I2R, DIR_R2I, FORMAT, SESSION_VECTORS_JSON, SPEC, VERSION,
};
pub use runner::{byte_diff, case_name, rebuild_link};
pub use util::{layout_byte_ranges, merge_ranges, to_hex, zero_ranges};
