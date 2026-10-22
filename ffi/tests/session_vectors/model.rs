//! 跨端 **L2 会话向量**的数据模型与解析。
//!
//! 数据来自公开仓 `KimChow/lengsuo-network` 的
//! `crates/lengsuo-link-core/tests/vectors/session_data.rs`，revision 与
//! `ffi/Cargo.toml` 里 `lengsuo-link-core` 的 pin **逐字符相同**
//! （来源三元组与不变式见同目录 [`SOURCE.md`]）。
//!
//! # 为什么 JSON 读取器是「复用」而不是「再写一份」
//!
//! 本仓已有 `tests/vectors/json.rs`（`mb-ffi-vectors` 交付）：一个**与向量
//! 形状无关**的极小 JSON DOM 解析器 + 规范化渲染器（对象**保序**、数字
//! **保留字面文本**）。L2 会话向量的形状（`links[]` / `rounds[]` /
//! `frames[]`）完全落在它支持的子集内 ⇒ **直接复用**，
//! 本目录不再写第二份解析器，也**不引入** `serde` / `serde_json`
//! （`ffi/Cargo.toml` 属共享区，本卡不许加依赖）。
//!
//! # 墙钟区段（volatile ranges）
//!
//! 握手报文里有一段**依赖墙钟**的字节（TAI64N 时间戳密文 + 覆盖它的
//! MAC1），跨进程不可能逐字节相同 ⇒ 向量把那段**声明**出来
//! （`volatile_ranges`），并给出「清零后」的 `canonical_hex`。
//!
//! ⚠️ 清零范围**必须从向量读**（`volatile_ranges` 字段），
//! **不得**在测试里写死 `88-132` / `44-76`（向量才是权威）。
//!
//! # 密钥纪律
//!
//! 向量里的密钥全是**测试专用假密钥**（`0x11` / `0x22` / `0x33` / `0x44`
//! 模式字节），**不得**用于任何真实会话。

#![allow(dead_code)] // 每个测试二进制只用到其中一部分工具。

use lengsuo_mobile_ffi::core::FrameType;

use super::json::{self, Json};
use super::util::{frame_type_from_name, from_hex, key32, parse_ranges};

/// 向量文件的**格式标识**（移动端按它识别这是哪一版向量）。
pub const FORMAT: &str = "lengsuo-link-session-vectors";
/// 向量文件的**版本号**。
pub const VERSION: u64 = 1;
/// 向量文件指向的规范文档。
pub const SPEC: &str = "docs/link-vectors-spec.md";

/// 内嵌的权威向量原文（与磁盘上的 `session_vectors.json` 逐字节相同）。
pub const SESSION_VECTORS_JSON: &str = include_str!("session_vectors.json");

/// 帧方向：发起方 → 响应方。
pub const DIR_I2R: &str = "initiator_to_responder";
/// 帧方向：响应方 → 发起方。
pub const DIR_R2I: &str = "responder_to_initiator";

/// 帧方向（类型化，避免方向字符串写错）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// 发起方 → 响应方。
    InitiatorToResponder,
    /// 响应方 → 发起方。
    ResponderToInitiator,
}

impl Direction {
    /// 由向量里的名字还原方向。
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            DIR_I2R => Self::InitiatorToResponder,
            DIR_R2I => Self::ResponderToInitiator,
            other => panic!("未知帧方向 {other}"),
        }
    }

    /// 稳定名字（用于失败信息）。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InitiatorToResponder => DIR_I2R,
            Self::ResponderToInitiator => DIR_R2I,
        }
    }
}

/// 一条链路的**密钥材料**（测试专用假密钥）。
#[derive(Debug, Clone)]
pub struct LinkMaterial {
    /// 发起方静态私钥。
    pub initiator_static_private: [u8; 32],
    /// 发起方静态公钥。
    pub initiator_static_public: [u8; 32],
    /// 响应方静态私钥。
    pub responder_static_private: [u8; 32],
    /// 响应方静态公钥。
    pub responder_static_public: [u8; 32],
}

/// 一份握手报文的向量（**墙钟区段已清零**的 canonical 形式）。
#[derive(Debug, Clone)]
pub struct MessageVector {
    /// 依赖墙钟的字节区间（半开区间，可多段）。
    pub volatile_ranges: Vec<(usize, usize)>,
    /// 墙钟区段的布局说明（逐段说明那几段各自是什么）。
    pub volatile_layout: String,
    /// canonical 字节（墙钟区段清零）。
    pub canonical: Vec<u8>,
}

/// 传输密钥向量（双方各两把）。
#[derive(Debug, Clone)]
pub struct TransportKeys {
    /// 发起方发送密钥。
    pub initiator_send: [u8; 32],
    /// 发起方接收密钥。
    pub initiator_receive: [u8; 32],
    /// 响应方发送密钥。
    pub responder_send: [u8; 32],
    /// 响应方接收密钥。
    pub responder_receive: [u8; 32],
}

/// 一条密文帧向量（**整帧期望字节**）。
#[derive(Debug, Clone)]
pub struct FrameVector {
    /// 帧名（链路内唯一）。
    pub name: String,
    /// 帧方向。
    pub direction: Direction,
    /// 帧类型。
    pub frame_type: FrameType,
    /// 帧序号。
    pub seq: u64,
    /// 传输层计数器（决定 nonce；**每个方向各自**从 0 开始）。
    pub counter: u64,
    /// 明文。
    pub plaintext: Vec<u8>,
    /// 期望的整帧字节。
    pub frame: Vec<u8>,
}

/// 一轮握手的向量。
#[derive(Debug, Clone)]
pub struct RoundVector {
    /// 轮次（从 1 开始）。
    pub round: u64,
    /// 发起方本轮 index。
    pub initiator_index: u32,
    /// 响应方本轮 index。
    pub responder_index: u32,
    /// 发起方本轮临时私钥种子。
    pub initiator_seed: [u8; 32],
    /// 响应方本轮临时私钥种子。
    pub responder_seed: [u8; 32],
    /// 握手成功后的世代号（双方一致）。
    pub generation: u64,
    /// initiation 报文向量。
    pub initiation: MessageVector,
    /// response 报文向量。
    pub response: MessageVector,
    /// 传输密钥向量。
    pub transport_keys: TransportKeys,
    /// 帧向量（按发送顺序）。
    pub frames: Vec<FrameVector>,
}

/// 一条链路的向量。
#[derive(Debug, Clone)]
pub struct LinkVector {
    /// 链路名。
    pub name: String,
    /// 密钥材料。
    pub material: LinkMaterial,
    /// 各轮握手。
    pub rounds: Vec<RoundVector>,
}

/// 解析整份会话向量 JSON。
pub fn parse_links(text: &str) -> Vec<LinkVector> {
    let root = json::parse(text);
    assert_eq!(root.field("format").as_str(), FORMAT, "向量格式标识不符");
    assert_eq!(root.field("version").as_u64(), VERSION, "向量版本号不符");
    assert_eq!(root.field("spec").as_str(), SPEC, "向量规范文档路径不符");
    root.field("links")
        .as_arr()
        .iter()
        .map(parse_link)
        .collect()
}

/// 解析一条链路的向量。
fn parse_link(item: &Json) -> LinkVector {
    let material = LinkMaterial {
        initiator_static_private: key32(item.field("initiator_static_private_hex").as_str()),
        initiator_static_public: key32(item.field("initiator_static_public_hex").as_str()),
        responder_static_private: key32(item.field("responder_static_private_hex").as_str()),
        responder_static_public: key32(item.field("responder_static_public_hex").as_str()),
    };
    let rounds = item
        .field("rounds")
        .as_arr()
        .iter()
        .map(parse_round)
        .collect();
    LinkVector {
        name: item.field("name").as_str().to_string(),
        material,
        rounds,
    }
}

/// 解析一轮握手的向量。
fn parse_round(item: &Json) -> RoundVector {
    let keys = item.field("transport_keys");
    RoundVector {
        round: item.field("round").as_u64(),
        initiator_index: item.field("initiator_index").as_u64() as u32,
        responder_index: item.field("responder_index").as_u64() as u32,
        initiator_seed: key32(item.field("initiator_ephemeral_seed_hex").as_str()),
        responder_seed: key32(item.field("responder_ephemeral_seed_hex").as_str()),
        generation: item.field("generation").as_u64(),
        initiation: parse_message(item.field("initiation")),
        response: parse_message(item.field("response")),
        transport_keys: TransportKeys {
            initiator_send: key32(keys.field("initiator_send_hex").as_str()),
            initiator_receive: key32(keys.field("initiator_receive_hex").as_str()),
            responder_send: key32(keys.field("responder_send_hex").as_str()),
            responder_receive: key32(keys.field("responder_receive_hex").as_str()),
        },
        frames: item
            .field("frames")
            .as_arr()
            .iter()
            .map(parse_frame)
            .collect(),
    }
}

/// 解析一份握手报文的向量。
fn parse_message(item: &Json) -> MessageVector {
    MessageVector {
        volatile_ranges: parse_ranges(item.field("volatile_ranges").as_str()),
        volatile_layout: item.field("volatile_layout").as_str().to_string(),
        canonical: from_hex(item.field("canonical_hex").as_str()),
    }
}

/// 解析一条帧向量。
fn parse_frame(item: &Json) -> FrameVector {
    FrameVector {
        name: item.field("name").as_str().to_string(),
        direction: Direction::from_name(item.field("direction").as_str()),
        frame_type: frame_type_from_name(item.field("frame_type").as_str()),
        seq: item.field("seq").as_u64(),
        counter: item.field("counter").as_u64(),
        plaintext: from_hex(item.field("plaintext_hex").as_str()),
        frame: from_hex(item.field("frame_hex").as_str()),
    }
}
