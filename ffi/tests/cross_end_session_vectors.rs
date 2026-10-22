//! **跨端 L2 会话向量对拍**：把 `lengsuo-link-core` 的 L2 会话向量搬进
//! 移动端仓，让门面层的握手字节、世代号、密文帧逐字节对拍权威字节。
//!
//! 与 `cross_end_vectors.rs`（`mb-ffi-vectors` 交付）的区别：那份对拍的是
//! **L3 帧格式**这一层（无状态编解码）；本文件对拍的是 **L2 会话**这一层：
//!
//! 1. **握手报文**：发起方 `create_initiation` / 响应方 `create_response`
//!    铸出的字节，**墙钟区段按向量声明清零**后与 `canonical_hex` 逐字节相同；
//! 2. **世代号**：握手成功后双方 `generation_value()` 与向量一致，且
//!    **重连轮 = 上一代 + 1**；
//! 3. **密文帧整帧字节**：双方各自 `seal_frame` 封出的整帧与 `frame_hex`
//!    逐字节相同（**方向**按 `direction` 字段选发送方）；
//! 4. **解密回读**：对端 `open_frame` 解出的明文 / 帧类型 / 序号与向量一致。
//!
//! # 判据的权威出处
//!
//! 源仓 `crates/lengsuo-link-core/tests/vectors/consistency.rs` 的
//! `every_session_vector_is_reproducible` / `generation_is_reproducible_and_advances_by_one`
//! / `every_handshake_message_canonical_form_is_reproducible` /
//! `every_expected_frame_decodes_back_to_its_plaintext` —— 本文件的断言与它们
//! **逐条等价**（不是自创判据）。
//!
//! # 墙钟区段
//!
//! 握手报文里有一段**依赖墙钟**的字节（TAI64N 时间戳密文 + 覆盖它的 MAC1），
//! 跨进程必然不同 ⇒ 向量把那段**声明**出来（`volatile_ranges`），并给出
//! **清零后**的 `canonical_hex`。本文件**从向量读**清零范围，
//! **不写死** `88-132` / `44-76`（向量才是权威）。
//!
//! # 固定测试密钥
//!
//! 向量里的密钥全是**测试专用假密钥**（`0x11` / `0x22` / `0x33` / `0x44` 模式字节），
//! 仅用于对拍，**不得**用于真实会话。

mod session_vectors;

use session_vectors::{
    byte_diff, case_name, layout_byte_ranges, merge_ranges, parse_links, rebuild_link, to_hex,
    zero_ranges, Direction, DIR_I2R, DIR_R2I, FORMAT, SESSION_VECTORS_JSON, SPEC, VERSION,
};

/// 向量文件必须存在、非空，且与编译期内嵌文本逐字节相同。
#[test]
fn session_vector_file_is_present_and_matches_the_embedded_text() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/session_vectors/session_vectors.json"
    );
    let on_disk = std::fs::read_to_string(path).expect("向量文件必须存在且可读");
    assert!(!on_disk.is_empty(), "向量文件不能为空");
    assert_eq!(
        on_disk, SESSION_VECTORS_JSON,
        "内嵌文本必须与磁盘文件逐字节相同"
    );
}

/// 向量集非空，且「解析后重新渲染」与原文逐字节相同（规范化校验）。
#[test]
fn session_vectors_json_is_canonical() {
    let value = session_vectors::json::parse(SESSION_VECTORS_JSON);
    assert_eq!(
        session_vectors::json::render(&value),
        SESSION_VECTORS_JSON,
        "解析后重新渲染必须与向量原文逐字节相同"
    );
}

/// 向量子集**非空**：`links[]` 与 `rounds[]` 数量 > 0（零用例不算通过）。
#[test]
fn session_vector_set_is_non_empty() {
    let links = parse_links(SESSION_VECTORS_JSON);
    assert!(!links.is_empty(), "会话向量 links 数量必须 > 0");
    let total_rounds: usize = links.iter().map(|link| link.rounds.len()).sum();
    assert!(total_rounds > 0, "会话向量 rounds 数量必须 > 0");
    for link in &links {
        assert!(!link.rounds.is_empty(), "链路 {} 至少要有一轮", link.name);
        for round in &link.rounds {
            assert!(
                !round.frames.is_empty(),
                "链路 {} 第 {} 轮至少要有一帧",
                link.name,
                round.round
            );
        }
    }
}

/// 向量文件的头部字段与本卡常量一致（移动端按它们识别版本）。
#[test]
fn session_vector_header_matches_constants() {
    let value = session_vectors::json::parse(SESSION_VECTORS_JSON);
    assert_eq!(value.field("format").as_str(), FORMAT);
    assert_eq!(value.field("version").as_u64(), VERSION);
    assert_eq!(value.field("spec").as_str(), SPEC);
}

/// 墙钟区段的**声明**与**布局说明**必须自洽。
///
/// `volatile_ranges` 说「哪几个字节是墙钟依赖」，`volatile_layout` 逐段说
/// 「那几段各自是什么」⇒ 两者合并后必须**恰好覆盖**同一段字节
/// （否则移动端按文档清零会清错位置）。
#[test]
fn volatile_ranges_match_the_declared_layout() {
    for link in parse_links(SESSION_VECTORS_JSON) {
        for round in &link.rounds {
            for (label, message) in [
                ("initiation", &round.initiation),
                ("response", &round.response),
            ] {
                let declared = merge_ranges(&message.volatile_ranges);
                let described = merge_ranges(&layout_byte_ranges(&message.volatile_layout));
                assert_eq!(
                    declared, described,
                    "链路 {} 第 {} 轮的 {label} volatile_ranges 与 volatile_layout 不自洽",
                    link.name, round.round
                );
                assert!(
                    !declared.is_empty(),
                    "链路 {} 第 {} 轮的 {label} 必须声明墙钟区段",
                    link.name,
                    round.round
                );
            }
        }
    }
}

/// **握手报文 canonical 字节对拍**（本卡核心判据之一）。
///
/// 发起方 `create_initiation` ⇒ 墙钟区段清零后与 `canonical_hex` 逐字节相同；
/// 响应方 `create_response` ⇒ 同理。
#[test]
fn handshake_messages_match_the_canonical_bytes() {
    for link in parse_links(SESSION_VECTORS_JSON) {
        let rebuilt = rebuild_link(&link);
        assert_eq!(
            rebuilt.len(),
            link.rounds.len(),
            "链路 {} 的重算轮数不符",
            link.name
        );
        for (round, outcome) in link.rounds.iter().zip(&rebuilt) {
            let case = case_name(&link, round, None);

            let canonical_init =
                zero_ranges(&outcome.initiation, &round.initiation.volatile_ranges);
            assert_eq!(
                to_hex(&canonical_init),
                to_hex(&round.initiation.canonical),
                "{case}：initiation canonical 字节不符（{}）",
                byte_diff(&round.initiation.canonical, &canonical_init)
            );

            let canonical_resp = zero_ranges(&outcome.response, &round.response.volatile_ranges);
            assert_eq!(
                to_hex(&canonical_resp),
                to_hex(&round.response.canonical),
                "{case}：response canonical 字节不符（{}）",
                byte_diff(&round.response.canonical, &canonical_resp)
            );
        }
    }
}

/// **世代号对拍**：握手成功后双方世代号与向量一致；**重连轮 = 上一代 + 1**。
#[test]
fn generation_matches_and_advances_by_one_on_reconnect() {
    for link in parse_links(SESSION_VECTORS_JSON) {
        let rebuilt = rebuild_link(&link);
        let mut previous: Option<u64> = None;
        for (round, outcome) in link.rounds.iter().zip(&rebuilt) {
            let case = case_name(&link, round, None);
            assert_eq!(
                outcome.initiator_generation, round.generation,
                "{case}：发起方世代号不符"
            );
            assert_eq!(
                outcome.responder_generation, round.generation,
                "{case}：响应方世代号不符"
            );
            // 源仓 `generation_is_reproducible_and_advances_by_one` 的判据：
            // 首轮 = 初始值 0；之后每轮 = 上一代 + 1。
            match previous {
                None => assert_eq!(round.generation, 0, "{case}：首轮世代号必须是初始值 0"),
                Some(prev) => assert_eq!(
                    round.generation,
                    prev + 1,
                    "{case}：重连后世代号必须 = 上一代 + 1"
                ),
            }
            previous = Some(round.generation);
        }
    }
}

/// **密文帧整帧字节对拍**：双方各自 `seal_frame` 封出的整帧与 `frame_hex`
/// 逐字节相同（方向按 `direction` 选发送方）。
#[test]
fn sealed_frames_match_the_expected_frame_bytes() {
    for link in parse_links(SESSION_VECTORS_JSON) {
        let rebuilt = rebuild_link(&link);
        for (round, outcome) in link.rounds.iter().zip(&rebuilt) {
            assert_eq!(
                outcome.frames.len(),
                round.frames.len(),
                "链路 {} 第 {} 轮的帧数不符",
                link.name,
                round.round
            );
            for (frame, actual) in round.frames.iter().zip(&outcome.frames) {
                let case = case_name(&link, round, Some(&frame.name));
                assert_eq!(
                    to_hex(&actual.wire),
                    to_hex(&frame.frame),
                    "{case}：密文帧整帧字节不符（{}）",
                    byte_diff(&frame.frame, &actual.wire)
                );
            }
        }
    }
}

/// **解密回读**：对端 `open_frame` 解出的明文 / 帧类型 / 序号与向量一致。
#[test]
fn opened_frames_decode_back_to_the_expected_plaintext() {
    for link in parse_links(SESSION_VECTORS_JSON) {
        let rebuilt = rebuild_link(&link);
        for (round, outcome) in link.rounds.iter().zip(&rebuilt) {
            for (frame, actual) in round.frames.iter().zip(&outcome.frames) {
                let case = case_name(&link, round, Some(&frame.name));
                assert_eq!(
                    actual.plaintext, frame.plaintext,
                    "{case}：解密回读的明文不符"
                );
                assert_eq!(actual.frame_type, frame.frame_type, "{case}：帧类型不符");
                assert_eq!(actual.seq, frame.seq, "{case}：帧序号不符");
            }
        }
    }
}

/// 帧的**方向**确实被区分（两个方向都出现过，且不混淆）。
///
/// 若实现把帧全从一个方向发，本用例会失败（对端解不开 / 计数器错位）。
#[test]
fn frames_are_sealed_in_both_directions() {
    let mut saw_i2r = false;
    let mut saw_r2i = false;
    for link in parse_links(SESSION_VECTORS_JSON) {
        for round in &link.rounds {
            for frame in &round.frames {
                match frame.direction {
                    Direction::InitiatorToResponder => saw_i2r = true,
                    Direction::ResponderToInitiator => saw_r2i = true,
                }
            }
        }
    }
    assert!(saw_i2r, "向量必须含发起方 → 响应方方向的帧");
    assert!(saw_r2i, "向量必须含响应方 → 发起方方向的帧");
}

/// 帧的**计数器**与**实际发送顺序**一致（每个方向各自从 0 起）。
///
/// 这是向量自带的判据：计数器的顺序错 ⇒ nonce 错 ⇒ 密文字节必然不符。
#[test]
fn frame_counters_match_per_direction_send_order() {
    for link in parse_links(SESSION_VECTORS_JSON) {
        for round in &link.rounds {
            for direction in [
                Direction::InitiatorToResponder,
                Direction::ResponderToInitiator,
            ] {
                for (index, frame) in round
                    .frames
                    .iter()
                    .filter(|frame| frame.direction == direction)
                    .enumerate()
                {
                    assert_eq!(
                        frame.counter, index as u64,
                        "链路 {} 第 {} 轮的帧 {}：计数器必须与该方向的发送顺序一致",
                        link.name, round.round, frame.name
                    );
                }
            }
        }
    }
}

/// 传输密钥的**方向性**正确：发起方 send == 响应方 receive，反之亦然。
///
/// 向量里的传输密钥是跨端约定的**独立证据**（不由本卡重算）⇒ 这一条
/// 校验的是「向量本身自洽」，并间接保证密文帧对拍的密钥来源正确。
#[test]
fn transport_keys_are_directional() {
    for link in parse_links(SESSION_VECTORS_JSON) {
        for round in &link.rounds {
            let keys = &round.transport_keys;
            let case = case_name(&link, round, None);
            assert_eq!(
                keys.initiator_send, keys.responder_receive,
                "{case}：发起方发送密钥必须等于响应方接收密钥"
            );
            assert_eq!(
                keys.initiator_receive, keys.responder_send,
                "{case}：发起方接收密钥必须等于响应方发送密钥"
            );
        }
    }
}

/// 跨向量不矛盾：不同链路**不共用**密钥材料（不共用密钥）。
#[test]
fn links_do_not_share_key_material() {
    let links = parse_links(SESSION_VECTORS_JSON);
    for (index, link) in links.iter().enumerate() {
        for other in &links[index + 1..] {
            assert_ne!(
                link.material.initiator_static_private, other.material.initiator_static_private,
                "链路 {} 与 {} 共用了发起方静态私钥",
                link.name, other.name
            );
            assert_ne!(
                link.material.responder_static_private, other.material.responder_static_private,
                "链路 {} 与 {} 共用了响应方静态私钥",
                link.name, other.name
            );
        }
    }
}

/// 帧方向常量与源仓一致（防止本卡把方向字符串写反）。
#[test]
fn direction_constants_match_the_source() {
    assert_eq!(DIR_I2R, "initiator_to_responder");
    assert_eq!(DIR_R2I, "responder_to_initiator");
    assert!(matches!(
        Direction::from_name(DIR_I2R),
        Direction::InitiatorToResponder
    ));
    assert!(matches!(
        Direction::from_name(DIR_R2I),
        Direction::ResponderToInitiator
    ));
}
