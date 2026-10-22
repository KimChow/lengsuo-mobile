//! 跨端 L2 会话向量的**执行器**：用门面层跑握手 + 封帧，产出「应得的字节」。
//!
//! # 与源仓重算引擎的关系
//!
//! 源仓 `crates/lengsuo-link-core/tests/vectors/rebuild.rs` 是**重算引擎**
//! （只读参考，不照抄）：它在**传输层**（`lengsuo-wireguard`）重算握手，
//! 在**会话层**（`lengsuo-link-core`）重算世代号与封出的帧。
//!
//! 本模块走**移动端门面层**（`lengsuo_mobile_ffi::core::Session`）做同样的事：
//! 门面把 link-core 包成**绑定无关的稳定 API**，正是移动端将来真正调用的
//! 那一层 ⇒ 对拍的对象必须是它，而不是源仓的内部模块。
//!
//! # 时钟语义
//!
//! 墙钟区段（`volatile_ranges`）由向量声明，测试只比对**清零后**的字节。
//! 但会话层还有两处**依赖时间**的检查，重连轮必须跨过：
//!
//! - **握手洪水窗口**（20ms，两次「消费 initiation」之间的最小间隔）——
//!   看的是调用方传入的 `Instant`（单调钟）；
//! - **TAI64N 时间戳**（约 16.78ms 刻度；重连的 initiation 必须**严格晚于**
//!   上一轮，否则会被**重放检查**拒绝）——看的是**墙钟**。
//!
//! ⇒ 重连轮必须等墙钟真的走过一格再发第二轮，否则握手会被拒。
//! 本模块用 [`advance_round`] 处理（与源仓 `advance_round` 同思路：两个时钟都要跨过）。
//!
//! # 为什么传输密钥 / 密文帧是**跨进程可复现**的
//!
//! 握手的 **transcript（hash 链）**里含墙钟时间戳 ⇒ 报文里的墙钟区段
//! 逐进程不同；但**传输密钥**只从 **chain key** 派生（chain key 由临时密钥、
//! 静态密钥与 DH 共享秘密混合而来，**不含**时间戳），**密文帧**的 nonce
//! 又只由**计数器**派生 ⇒ 两者**不含**任何随机数 / 墙钟 ⇒ 逐字节可复现。

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use lengsuo_mobile_ffi::core::{FrameType, Session, SessionConfig, SessionRole};

use super::model::{Direction, LinkVector, RoundVector};
use super::util::to_hex;

/// 重连轮的最小间隔（单调钟）。≥ 握手洪水窗口（20ms），且 > 墙钟刻度（~16.78ms）。
const ROUND_GAP: Duration = Duration::from_millis(50);

/// 一帧的对拍产物：封出的字节 + 对端解回来的明文。
#[derive(Debug, Clone)]
pub struct FrameOutcome {
    /// 发送方封出的**整帧**字节。
    pub wire: Vec<u8>,
    /// 接收方 `open_frame` 解出的明文。
    pub plaintext: Vec<u8>,
    /// 接收方解出的帧类型。
    pub frame_type: FrameType,
    /// 接收方解出的帧序号。
    pub seq: u64,
}

/// 一轮重算的产物（门面层跑出来的字节）。
#[derive(Debug, Clone)]
pub struct RebuiltRound {
    /// initiation 报文**原始**字节（含墙钟区段）。
    pub initiation: Vec<u8>,
    /// response 报文**原始**字节（含墙钟区段）。
    pub response: Vec<u8>,
    /// 逐帧对拍结果（与向量的 `frames[]` 顺序一致）。
    pub frames: Vec<FrameOutcome>,
    /// 发起方侧的世代号。
    pub initiator_generation: u64,
    /// 响应方侧的世代号。
    pub responder_generation: u64,
}

/// 在**移动端门面层**重算一条链路的全部轮次。
///
/// 判据与源仓 `rebuild_link` 逐条等价：同样的握手顺序、同样的
/// 「按 `frames[]` 的 `direction` 选发送方」规则、同样的世代号读数。
pub fn rebuild_link(link: &LinkVector) -> Vec<RebuiltRound> {
    let mut initiator = Session::new(SessionConfig::new(
        SessionRole::Initiator,
        link.material.initiator_static_private,
        link.material.responder_static_public,
    ))
    .expect("发起方会话可构造");
    let mut responder = Session::new(SessionConfig::new(
        SessionRole::Responder,
        link.material.responder_static_private,
        link.material.initiator_static_public,
    ))
    .expect("响应方会话可构造");

    let mut now = Instant::now();
    // 墙钟刻度基准：上一轮 initiation **铸出之后**立刻取（时间戳是铸出瞬间取的，
    // 这样「下一轮必须严格晚于它」才成立）。
    let mut tick = wall_clock_tick();
    let mut rebuilt = Vec::new();
    for (index, round) in link.rounds.iter().enumerate() {
        if index > 0 {
            now = advance_round(now, tick);
        }
        let initiation = initiator
            .create_initiation(round.initiator_seed, round.initiator_index, now)
            .expect("initiation 可生成");
        // 铸出后重新取刻度，作为下一轮的基准。
        tick = wall_clock_tick();

        responder
            .consume_initiation(&initiation, now)
            .expect("initiation 可剥壳");
        let response = responder
            .create_response(round.responder_seed, round.responder_index, now)
            .expect("response 可生成");
        initiator
            .consume_response(&response, now)
            .expect("response 可消费");

        // 按向量声明的方向选发送方（⛔ 不许把帧全从一个方向发），
        // 并**对端解回**明文（解密回读）。
        let mut frames = Vec::new();
        for frame in &round.frames {
            let (wire, opened) = match frame.direction {
                Direction::InitiatorToResponder => {
                    let wire = initiator
                        .seal_frame(frame.frame_type, frame.seq, &frame.plaintext)
                        .expect("发起方封帧成功");
                    let opened = responder.open_frame(&wire).expect("响应方解帧成功");
                    (wire, opened)
                }
                Direction::ResponderToInitiator => {
                    let wire = responder
                        .seal_frame(frame.frame_type, frame.seq, &frame.plaintext)
                        .expect("响应方封帧成功");
                    let opened = initiator.open_frame(&wire).expect("发起方解帧成功");
                    (wire, opened)
                }
            };
            frames.push(FrameOutcome {
                wire,
                plaintext: opened.plaintext,
                frame_type: opened.frame_type,
                seq: opened.seq,
            });
        }
        rebuilt.push(RebuiltRound {
            initiation,
            response,
            frames,
            initiator_generation: initiator.generation_value(),
            responder_generation: responder.generation_value(),
        });
    }
    rebuilt
}

/// 当前的**墙钟刻度**（12 字节，等价于源仓 `tai64n_now()` 的线格式）。
///
/// ⚠️ 这不是协议逻辑，只是**测试夹具**用来判断「墙钟是否走过一格」的探针
/// （公式与源仓 `lengsuo-wireguard::handshake::tai64n_now` 一致：
/// 8 字节 BE(TAI64 秒) || 4 字节 BE(纳秒 &! 0x00FF_FFFF) ⇒ 刻度 ~16.78ms）。
fn wall_clock_tick() -> [u8; 12] {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let seconds = 0x4000_0000_0000_000a_u64 + now.as_secs();
    let whitened = now.subsec_nanos() & !0x00ff_ffff;
    let mut out = [0u8; 12];
    out[..8].copy_from_slice(&seconds.to_be_bytes());
    out[8..].copy_from_slice(&whitened.to_be_bytes());
    out
}

/// 推进到「下一轮」可用的时刻（返回新的 `Instant`）。
///
/// ⚠️ 两个**独立**的时钟源都必须跨过：
/// - **单调钟**（会话层洪水窗口 20ms）：否则重连会被洪水检查拒绝；
/// - **墙钟**（TAI64N，刻度 ~16.78ms）：否则重连的 initiation 会被
///   **重放检查**拒绝（时间戳不晚于上一轮）。
pub fn advance_round(previous: Instant, previous_tick: [u8; 12]) -> Instant {
    loop {
        std::thread::sleep(Duration::from_millis(2));
        let now = Instant::now();
        let monotonic = now
            .checked_duration_since(previous)
            .is_some_and(|gap| gap >= ROUND_GAP);
        // 墙钟刻度必须**严格晚于**上一轮（逐字节字典序，等价于源仓的
        // `tai64n_after`）。
        let wall = wall_clock_tick() > previous_tick;
        if monotonic && wall {
            return now;
        }
    }
}

/// 失败信息里的「case 名」：链路名 + 轮次（+ 帧名）。
///
/// 判据要求**每条断言的失败信息必须打印 case 名** ⇒ 统一走这里，
/// 保证「链路名 + 轮次 + 帧名」三者齐全。
pub fn case_name(link: &LinkVector, round: &RoundVector, frame: Option<&str>) -> String {
    let mut name = format!("链路 {} 第 {} 轮", link.name, round.round);
    if let Some(frame) = frame {
        name.push_str(&format!(" 帧 {frame}"));
    }
    name
}

/// 字节差异摘要（期望 / 实际），供失败信息使用。
///
/// 只打印**第一处不同**的偏移与该偏移附近的字节，避免整帧十六进制刷屏。
pub fn byte_diff(expected: &[u8], actual: &[u8]) -> String {
    if expected == actual {
        return "字节相同".to_string();
    }
    if expected.len() != actual.len() {
        return format!(
            "长度不同：期望 {} 字节，实际 {} 字节",
            expected.len(),
            actual.len()
        );
    }
    let at = expected
        .iter()
        .zip(actual)
        .position(|(left, right)| left != right)
        .unwrap_or(0);
    let window = 8usize;
    let start = at.saturating_sub(window);
    let end = (at + window).min(expected.len());
    format!(
        "首个差异在偏移 {at}：期望 {}，实际 {}（期望全长 {} 字节，实际全长 {} 字节）",
        to_hex(&expected[start..end]),
        to_hex(&actual[start..end]),
        expected.len(),
        actual.len()
    )
}
