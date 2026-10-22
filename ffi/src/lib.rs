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
//! `lengsuo-link-core` 的 `frame` / `session` / `negotiate` 三个模块已落地
//! （依赖固定在真实实现 rev）。本 crate 在其上提供**绑定无关**的门面层
//! [`core`]：纯 Rust、无 IO、无平台依赖、不含任何绑定机制。
//!
//! 绑定层（现成绑定框架 / 手写 ABI）**尚未落地**：等主理人对「新增绑定依赖」
//! 「`panic` 设置」「codegen 步骤」三项拍板后，再在 [`core`] 之上加薄壳。
//! ⇒ [`supported_capabilities`] 仍返回 `0`（本端尚未暴露任何**门面板**能力）。
//!
//! # 能力位：**门面板**与**协议能力集**是两套东西
//!
//! [`capability`] 是**门面板**（本端绑定层实现了哪些功能），
//! 协议协商用的是 link-core 的 `Capability`（线上可选特性）。
//! 两者**位号互不相交**，跨命名空间只能走
//! [`core::negotiate::capability_set_from_facade`] 的**显式对照表**
//! ⇒ 位序语义漂移在**编译期 / 测试期**就被挡住（见 `tests/capability_align.rs`）。
//!
//! ## 绑定方案（**待定**）
//!
//! 两种候选方案（现成绑定框架 vs 手写 ABI）：**尚未决定**，
//! 建议在 M1 阶段做小规模 spike 后再定（见本地决策记录）。
//! 本文件现在用**手写 ABI 的形状**占位，是为了让 ABI 边界先定下来，
//! 便于两种方案复用同一份边界设计。
//!
//! ## 纪律
//!
//! - ⛔ 本 crate **不引入 tokio 或任何平台依赖** ⇒ 静态库能在 iOS/Android 上编。
//! - ⛔ 本 crate **不做 IO**（socket/文件/线程）：移动端侧由平台层负责。
//! - ⛔ 协议语义**不在这里重新实现** —— 那是 `lengsuo-link-core` 的事。

pub mod core;

/// 本绑定层与其承载的协议核心的版本。
///
/// 移动端在握手时用它做**能力协商**：版本不匹配 ⇒ 明确拒绝，
/// 不尝试「兼容处理」。
pub const BINDING_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 协议核心 crate 的版本（由 Cargo 解析得出，编译期固化）。
pub const PROTOCOL_CORE_VERSION: &str = "0.1.0";

/// 移动端**门面能力位**（绑定层的能力声明面板）。
///
/// 用**位**而不是字符串列表，便于移动端（Swift/Kotlin）零成本比对。
///
/// # ⛔ 门面板 ≠ link-core 协议能力集
///
/// 本模块的位与 `lengsuo-link-core` 的**协议能力位**
/// （`Capability`：`Keepalive` / `Renegotiate` / `CiphertextOnlyRelay` /
/// `StreamBothDirections`）是**两个互不相交的命名空间**：
///
/// | 命名空间 | 回答的问题 | 谁定义位号 |
/// |---|---|---|
/// | 本模块（门面板） | 本端**绑定层实现了哪些功能** | 本仓 |
/// | 协议能力集 | 线上协商的**可选特性** | link-core（已发布，不可改） |
///
/// ⚠️ **同名位、不同义**：门面板的「密文帧编解码」与协议位 0 的
/// 「keepalive 保活帧」**毫无关系**。⇒ 门面板位**不得**当作协议位使用：
/// 一旦被直接塞进 `CapabilitySet::from_bits`，对端会按**协议语义**解读它，
/// 协商**不报错**而双方理解的能力集**完全错位**（静默漂移）。
///
/// # 跨命名空间只能走显式对照表
///
/// 门面板 → 协议能力集的唯一通道是
/// [`crate::core::negotiate::capability_set_from_facade`]：它按**逐项显式的对照表**
/// 换算，**未登记位 fail-closed 报错**（⛔ 不静默丢弃、不静默当已知位）。
///
/// 对照表当前**四项全为「协议侧无对应」**（门面板的四项在协议侧都没有对应能力，
/// 例如「事件回放」在协议侧根本不存在）⇒ **门面板不参与协议协商**。
///
/// # 位号基址：刻意避开协议位
///
/// 门面板从 **bit 16** 起编号，避开 link-core **已发布**的 `0..=3`
/// （语义完全不同，复用即漂移），也避开其**近期保留**的 `4..=15`。
/// ⇒ 即便有人误把门面板掩码当协议掩码塞进 `CapabilitySet::from_bits`，
/// link-core 看到的也只是**未知位**（按协议纪律忽略并记录），
/// 而**不会**误解成 `Keepalive` 等既有能力。
pub mod capability {
    /// 支持密文帧的编解码（`lengsuo-link-core` 的 `frame` 模块）。
    pub const FRAME_CODEC: u32 = 1 << 16;
    /// 支持通道协商（`p2p` / `relay`）。
    pub const CHANNEL_NEGOTIATION: u32 = 1 << 17;
    /// 支持连接世代号（用于中继↔P2P 切换的去重与补齐）。
    pub const CONNECTION_GENERATION: u32 = 1 << 18;
    /// 支持按游标回溯事件（`afterSeq` 语义）。
    pub const EVENT_REPLAY: u32 = 1 << 19;

    /// 全部**已定义**的门面能力位。
    pub const ALL_DEFINED: u32 =
        FRAME_CODEC | CHANNEL_NEGOTIATION | CONNECTION_GENERATION | EVENT_REPLAY;
}

/// 当前实现具备的能力位集合。
///
/// ⛔ **只列真正已实现的**。协议实现尚未合入时，本值应只反映骨架能力，
/// 不得「预先声明」尚未落地的能力位 —— 否则移动端会以为对端支持而走上错误路径。
///
/// # 命名空间
///
/// 本值是**门面板**（[`capability`]）的位，⛔ **不是**协议协商用的能力集：
/// 协议能力集是 link-core 的 `Capability`（见 [`core::negotiate`]），
/// 两者位号互不相交，**不得**互相塞。
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
