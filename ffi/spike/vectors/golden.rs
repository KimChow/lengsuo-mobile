#![allow(dead_code)]
// 说明：本文件是**被复用的共享源文件**（被 ffi/tests 以 `#[path]` 引入），
// 各处用到的 API 子集不同 ⇒ 允许 dead_code 警告。

//! FFI spike 的**跨端测试向量**（单一事实来源）。
//!
//! # 为什么是 Rust 里的常量，而不是直接写 JSON
//!
//! 若向量只存在于 JSON，Rust 侧要解析 JSON ⇒ 要加 `serde_json` 依赖
//! ⇒ 要改 `ffi/Cargo.toml` 与 `ffi/Cargo.lock`（**共享区**，本卡不许动）。
//!
//! ⇒ 折中：**向量在这里（Rust 常量）**，
//!   `golden.json` 由 [`render_golden_json`] **渲染**出来，
//!   `ffi/tests/spike_golden.rs` 断言两者**逐字节一致**。
//!   ⇒ JSON 不会与 Rust 漂移，且移动端（Swift/Kotlin）能直接读 JSON。

/// 一个正常会话的向量。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GoldenCase {
    /// 向量名。
    pub name: &'static str,
    /// 32 字节密钥（**测试数据**，非真实密钥）。
    pub key: [u8; 32],
    /// 对端标签。
    pub peer_label: &'static str,
    /// 期望的连接世代号。
    pub epoch: u64,
    /// 期望的能力位掩码。
    pub capabilities: u32,
    /// 期望的密钥指纹。
    pub fingerprint: u64,
    /// 期望的会话标识。
    pub session_id: &'static str,
}

/// 一个错误分支的向量。
///
/// ⛔ 密钥用**定长数组 + 显式长度**（而不是 `Vec`）——
///    `Vec` 不能出现在 `const` 里。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GoldenErrorCase {
    /// 向量名。
    pub name: &'static str,
    /// 触发错误的密钥字节。
    pub key: [u8; 32],
    /// 触发错误的密钥**实际长度**（可能小于 32）。
    pub key_len: usize,
    /// 触发错误的对端标签。
    pub peer_label: &'static str,
    /// 期望的错误码（与 C ABI 的 `SpikeErrorCode` 一致）。
    pub error_code: i32,
    /// 期望的错误种类名（跨语言的语义名）。
    pub error_kind: &'static str,
}

impl GoldenErrorCase {
    /// 触发错误的密钥（**截断**到实际长度）。
    #[must_use]
    pub fn key_bytes(&self) -> &[u8] {
        &self.key[..self.key_len]
    }
}

/// 正常会话向量。
pub const CASES: [GoldenCase; 3] = [
    GoldenCase {
        name: "golden-alpha",
        key: [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
            0x1c, 0x1d, 0x1e, 0x1f,
        ],
        peer_label: "peer-a",
        epoch: 1,
        capabilities: 15,
        fingerprint: 16_630_484_232_385_628_869,
        session_id: "s-e6cb594c1a148ac5",
    },
    GoldenCase {
        name: "golden-beta",
        key: [
            0x03, 0x0a, 0x11, 0x18, 0x1f, 0x26, 0x2d, 0x34, 0x3b, 0x42, 0x49, 0x50, 0x57, 0x5e,
            0x65, 0x6c, 0x73, 0x7a, 0x81, 0x88, 0x8f, 0x96, 0x9d, 0xa4, 0xab, 0xb2, 0xb9, 0xc0,
            0xc7, 0xce, 0xd5, 0xdc,
        ],
        peer_label: "peer-b",
        epoch: 1,
        capabilities: 15,
        fingerprint: 11_720_265_927_062_773_285,
        session_id: "s-a2a6bfc397b0de25",
    },
    GoldenCase {
        name: "golden-gamma",
        key: [0xaa; 32],
        peer_label: "对端-γ",
        epoch: 1,
        capabilities: 15,
        fingerprint: 13_376_576_374_488_402_469,
        session_id: "s-b9a3255728438a25",
    },
];

/// 错误分支向量。
pub const ERROR_CASES: [GoldenErrorCase; 3] = [
    GoldenErrorCase {
        name: "short-key",
        key: [
            0x01, 0x02, 0x03, 0x04, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ],
        key_len: 4,
        peer_label: "peer-a",
        error_code: 1,
        error_kind: "KeyLength",
    },
    GoldenErrorCase {
        name: "empty-label",
        key: CASES[0].key,
        key_len: 32,
        peer_label: "",
        error_code: 4,
        error_kind: "InvalidPeerLabel",
    },
    GoldenErrorCase {
        name: "unknown-capability-bit",
        key: CASES[0].key,
        key_len: 32,
        peer_label: "peer-a",
        error_code: 3,
        error_kind: "InvalidCapability",
    },
];

/// 能力位的期望名字（顺序固定）。
pub const CAPABILITY_NAMES: [&str; 4] = [
    "FRAME_CODEC",
    "CHANNEL_NEGOTIATION",
    "CONNECTION_GENERATION",
    "EVENT_REPLAY",
];

/// 全部能力位掩码。
pub const CAPABILITY_ALL: u32 = 15;

/// 事件向量：条数。
pub const EVENT_COUNT: u32 = 3;

/// 事件向量：期望的序号。
pub const EVENT_SEQS: [u64; 3] = [1, 2, 3];

/// 事件向量：期望的载荷。
pub const EVENT_PAYLOADS: [&str; 3] = ["e0", "e1", "e2"];

/// 把字节数组渲染成小写十六进制。
fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// 把字符串渲染成 JSON 字符串字面量（含引号与转义）。
///
/// ⛔ 只做本文件用到的转义（`"` 与 `\`）；标签里若出现控制字符会被转成 `\uXXXX`。
fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// 渲染 `golden.json` 的内容。
///
/// ⚠️ 输出必须与 `ffi/spike/vectors/golden.json` **逐字节一致**
///    （由 `ffi/tests/spike_golden.rs` 断言）。
#[must_use]
pub fn render_golden_json() -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"_comment\": \"FFI spike 的跨端测试向量：Rust 侧算出的期望值，Swift/Kotlin 侧必须跑出同样的值。格式刻意用最朴素的 JSON（无依赖），便于移动端直接内嵌。\",\n");
    out.push_str("  \"key_len\": 32,\n");
    out.push_str(&format!("  \"capability_all\": {CAPABILITY_ALL},\n"));
    out.push_str("  \"capability_names\": [\n");
    for (i, name) in CAPABILITY_NAMES.iter().enumerate() {
        let comma = if i + 1 == CAPABILITY_NAMES.len() {
            ""
        } else {
            ","
        };
        out.push_str(&format!("    {}{comma}\n", json_string(name)));
    }
    out.push_str("  ],\n");

    out.push_str("  \"cases\": [\n");
    for (i, case) in CASES.iter().enumerate() {
        let comma = if i + 1 == CASES.len() { "" } else { "," };
        out.push_str("    {\n");
        out.push_str(&format!("      \"name\": {},\n", json_string(case.name)));
        out.push_str(&format!(
            "      \"key_hex\": {},\n",
            json_string(&hex(&case.key))
        ));
        out.push_str(&format!(
            "      \"peer_label\": {},\n",
            json_string(case.peer_label)
        ));
        out.push_str(&format!("      \"epoch\": {},\n", case.epoch));
        out.push_str(&format!("      \"capabilities\": {},\n", case.capabilities));
        out.push_str(&format!("      \"fingerprint\": {},\n", case.fingerprint));
        out.push_str(&format!(
            "      \"session_id\": {}\n",
            json_string(case.session_id)
        ));
        out.push_str(&format!("    }}{comma}\n"));
    }
    out.push_str("  ],\n");

    out.push_str("  \"error_cases\": [\n");
    for (i, case) in ERROR_CASES.iter().enumerate() {
        let comma = if i + 1 == ERROR_CASES.len() { "" } else { "," };
        out.push_str("    {\n");
        out.push_str(&format!("      \"name\": {},\n", json_string(case.name)));
        out.push_str(&format!(
            "      \"key_hex\": {},\n",
            json_string(&hex(case.key_bytes()))
        ));
        out.push_str(&format!(
            "      \"peer_label\": {},\n",
            json_string(case.peer_label)
        ));
        out.push_str(&format!("      \"error_code\": {},\n", case.error_code));
        out.push_str(&format!(
            "      \"error_kind\": {}\n",
            json_string(case.error_kind)
        ));
        out.push_str(&format!("    }}{comma}\n"));
    }
    out.push_str("  ],\n");

    out.push_str("  \"events\": {\n");
    out.push_str(&format!("    \"count\": {EVENT_COUNT},\n"));
    out.push_str("    \"seqs\": [1, 2, 3],\n");
    out.push_str("    \"kinds\": [\"tick\", \"tick\", \"tick\"],\n");
    out.push_str("    \"payloads\": [\"e0\", \"e1\", \"e2\"]\n");
    out.push_str("  }\n");
    out.push_str("}\n");
    out
}
