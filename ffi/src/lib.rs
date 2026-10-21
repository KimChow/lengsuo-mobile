#![forbid(unsafe_code)]
//! `lengsuo-mobile-ffi` — 跨端链路协议栈的 Swift/Kotlin 绑定层。
//!
//! **这一层刻意很薄**：协议语义全部在 `lengsuo-link-core`（无 IO，纯状态机），
//! 本 crate 只负责把它暴露成移动端能安全调用的**稳定 ABI**。
//!
//! ## 为什么要有这一层
//!
//! 移动端是**原生 Swift / Kotlin**（N6 拍板），而协议栈是 **Rust**。
//! 若在 Swift/Kotlin 里重写一遍握手与 AEAD，就是**两套密码学实现** ——
//! 两倍缺陷面、两倍维护、长期行为漂移风险。⇒ **协议只有一份实现**，
//! 移动端经 FFI 调用。
//!
//! ## 当前状态
//!
//! **骨架**。协议帧编解码（`lengsuo-link-core` 的 `frame` 模块）尚未实现
//! （在 `lengsuo-network` 线的 `lk-frames` 卡进行中）。
//! ⇒ 本 crate 现在只提供**版本与能力查询**这类不依赖协议实现的稳定入口，
//! 协议相关入口**待协议实现合入后再加**。
//!
//! ## 绑定方案（**待定**）
//!
//! `uniffi` vs 手写 `extern "C"`：**尚未决定**，建议在 M1 阶段做小规模 spike 后再定
//! （见 `../docs/mobile-roadmap.md` 第三节）。
//! 本文件现在用**手写 C ABI 的形状**占位，是为了让 ABI 边界先定下来，
//! 便于两种方案复用同一份边界设计。
//!
//! ## 纪律
//!
//! - ⛔ 本 crate **不引入 tokio 或任何平台依赖** ⇒ 静态库能在 iOS/Android 上编。
//! - ⛔ 本 crate **不做 IO**（socket/文件/线程）：移动端侧由平台层负责。
//! - ⛔ 协议语义**不在这里重新实现** —— 那是 `lengsuo-link-core` 的事。

/// 本绑定层与其承载的协议核心的版本。
///
/// 移动端在握手时用它做**能力协商**：版本不匹配 ⇒ 明确拒绝，
/// 不尝试「兼容处理」。
pub const BINDING_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 协议核心 crate 的版本（由 Cargo 解析得出，编译期固化）。
pub const PROTOCOL_CORE_VERSION: &str = "0.1.0";

/// 移动端可用能力位。
///
/// 协议协商用：**位**而不是字符串列表，便于移动端（Swift/Kotlin）零成本比对。
/// 每一位的语义**必须**与 `lengsuo-network` 的协议规范一一对应；
/// 规范未冻结前**不得**新增位。
pub mod capability {
    /// 支持密文帧的编解码（`lengsuo-link-core` 的 `frame` 模块）。
    pub const FRAME_CODEC: u32 = 1 << 0;
    /// 支持通道协商（`p2p` / `relay`）。
    pub const CHANNEL_NEGOTIATION: u32 = 1 << 1;
    /// 支持连接世代号（用于中继↔P2P 切换的去重与补齐）。
    pub const CONNECTION_GENERATION: u32 = 1 << 2;
    /// 支持按游标回溯事件（`afterSeq` 语义）。
    pub const EVENT_REPLAY: u32 = 1 << 3;
}

/// 当前实现具备的能力位集合。
///
/// ⛔ **只列真正已实现的**。协议实现尚未合入时，本值应只反映骨架能力，
/// 不得「预先声明」尚未落地的能力位 —— 否则移动端会以为对端支持而走上错误路径。
#[must_use]
pub const fn supported_capabilities() -> u32 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_not_empty() {
        assert!(!BINDING_VERSION.is_empty());
        assert!(!PROTOCOL_CORE_VERSION.is_empty());
    }

    #[test]
    fn capability_bits_are_distinct() {
        let all = capability::FRAME_CODEC
            | capability::CHANNEL_NEGOTIATION
            | capability::CONNECTION_GENERATION
            | capability::EVENT_REPLAY;
        // 每个位必须是不同的单一位 ⇒ 全部相加等于按位或
        let sum = capability::FRAME_CODEC
            + capability::CHANNEL_NEGOTIATION
            + capability::CONNECTION_GENERATION
            + capability::EVENT_REPLAY;
        assert_eq!(all, sum, "能力位必须互不相同");
    }

    #[test]
    fn skeleton_declares_no_protocol_capability_yet() {
        // 协议实现未合入 ⇒ 不得声明任何协议能力位。
        assert_eq!(supported_capabilities(), 0);
    }
}
