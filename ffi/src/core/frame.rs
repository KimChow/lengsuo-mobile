//! 帧编解码门面：**无状态**的线上帧格式。
//!
//! 这一层对应 `lengsuo-link-core::frame` 的无状态部分：帧头 + 载荷的切分与
//! 重组，**不持有任何会话状态**。真正需要密钥的封帧 / 解帧在
//! [`crate::core::session`]（`Session::seal_frame` / `Session::open_frame`）。
//!
//! # 为什么保留这一层
//!
//! 1. 中继 / 打洞等 L1 组件只需要**切帧**，不需要会话密钥；
//! 2. 跨端向量可以对**帧格式**做逐字节对拍（确定性，无随机数）；
//! 3. 绑定层可以把它原样暴露成「编码 / 解码」两个纯函数。
//!
//! # 类型来源
//!
//! [`FrameType`] 与长度上限**直接复用 `lengsuo-link-core` 的定义**——
//! ⛔ 门面不另编帧类型值，避免新旧端对同一字节有不同理解。

use lengsuo_link_core::frame as lc_frame;

use crate::core::error::CoreError;

pub use lengsuo_link_core::frame::{
    FrameType, MAX_FRAME_LEN, MAX_FRAME_PAYLOAD_LEN, MAX_PLAIN_LEN,
};

/// 一帧的线内容：帧头字段 + 载荷（密文包或明文，取决于调用方）。
///
/// `Debug` **只输出类型、序号与载荷长度**，绝不输出载荷字节。
#[derive(Clone, PartialEq, Eq)]
pub struct WireFrame {
    /// 帧类型。
    pub frame_type: FrameType,
    /// 帧序号（由调用方提供）。
    pub seq: u64,
    /// 载荷字节。
    pub payload: Vec<u8>,
}

impl WireFrame {
    /// 构造一帧。
    #[must_use]
    pub fn new(frame_type: FrameType, seq: u64, payload: Vec<u8>) -> Self {
        Self {
            frame_type,
            seq,
            payload,
        }
    }

    /// 编码成线上字节（定长帧头 + 载荷）。
    ///
    /// # Errors
    /// 载荷超过 [`MAX_FRAME_PAYLOAD_LEN`] ⇒ [`CoreError::PayloadTooLong`]，
    /// **不截断**。
    pub fn encode(&self) -> Result<Vec<u8>, CoreError> {
        let frame = lc_frame::Frame::new(self.frame_type, self.seq, self.payload.clone());
        frame.encode().map_err(CoreError::from)
    }

    /// 由 link-core 的帧类型转换。
    fn from_inner(frame: lc_frame::Frame) -> Self {
        Self {
            frame_type: frame.frame_type,
            seq: frame.seq,
            payload: frame.payload,
        }
    }
}

impl core::fmt::Debug for WireFrame {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("WireFrame")
            .field("frame_type", &self.frame_type)
            .field("seq", &self.seq)
            .field("payload_len", &self.payload.len())
            // 故意不输出 payload。
            .finish_non_exhaustive()
    }
}

/// 编码一帧。
///
/// 同一 `(frame_type, seq, payload)` 输入 ⇒ **逐字节相同**输出（无随机数）。
///
/// # Errors
/// 载荷超过 [`MAX_FRAME_PAYLOAD_LEN`] ⇒ [`CoreError::PayloadTooLong`]。
pub fn encode(frame_type: FrameType, seq: u64, payload: &[u8]) -> Result<Vec<u8>, CoreError> {
    WireFrame::new(frame_type, seq, payload.to_vec()).encode()
}

/// 切分出一个帧：返回帧内容与**已消费的字节数**（支持粘包）。
///
/// # Errors
/// 缓冲不足、帧头非法 ⇒ [`CoreError::InvalidFrame`] 等（不返回部分数据）。
pub fn decode(buf: &[u8]) -> Result<(WireFrame, usize), CoreError> {
    let (frame, consumed) = lc_frame::decode(buf).map_err(CoreError::from)?;
    Ok((WireFrame::from_inner(frame), consumed))
}

/// 切分出**恰好一个**帧：缓冲多一个字节都算错。
///
/// # Errors
/// 缓冲不足 / 多帧 / 帧头非法 ⇒ 对应的 [`CoreError`]。
pub fn decode_exact(buf: &[u8]) -> Result<WireFrame, CoreError> {
    lc_frame::decode_exact(buf)
        .map(WireFrame::from_inner)
        .map_err(CoreError::from)
}

/// 切分粘在一起的若干帧。
///
/// # Errors
/// 任一帧不完整 / 帧头非法 / 边界错位 ⇒ 对应的 [`CoreError`]；
/// 要么给出全部帧，要么报错，**不给半截结果**。
pub fn split_frames(buf: &[u8]) -> Result<Vec<WireFrame>, CoreError> {
    lc_frame::split_frames(buf)
        .map(|frames| frames.into_iter().map(WireFrame::from_inner).collect())
        .map_err(CoreError::from)
}
