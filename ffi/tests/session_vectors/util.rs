//! 跨端 L2 会话向量的**小工具**：十六进制、区间、帧类型名。
//!
//! 这些工具与「向量形状」无关，纯属机械换算 ⇒ 单独成文件，
//! 让 [`super`]（模型 + 解析）与 [`super::runner`]（执行器）都好读。

#![allow(dead_code)] // 每个测试二进制只用到其中一部分工具。

use lengsuo_mobile_ffi::core::FrameType;

/// 十六进制解码（拒绝奇数长度与非法字符）。
pub fn from_hex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0, "十六进制长度必须是偶数：{text}");
    let (pairs, _remainder) = text.as_bytes().as_chunks::<2>();
    let mut out = Vec::with_capacity(pairs.len());
    for pair in pairs {
        let high = (pair[0] as char).to_digit(16).expect("非法十六进制字符");
        let low = (pair[1] as char).to_digit(16).expect("非法十六进制字符");
        out.push(((high << 4) | low) as u8);
    }
    out
}

/// 十六进制小写编码。
pub fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut out, "{byte:02x}").expect("写字符串失败");
    }
    out
}

/// 把 32 字节十六进制解码成定长数组。
pub fn key32(text: &str) -> [u8; 32] {
    let bytes = from_hex(text);
    assert_eq!(bytes.len(), 32, "密钥字段必须是 32 字节");
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    out
}

/// 由名字还原帧类型（与 `FrameType::to_u8` 一一对应）。
pub fn frame_type_from_name(name: &str) -> FrameType {
    match name {
        "data" => FrameType::Data,
        "control" => FrameType::Control,
        "keepalive" => FrameType::Keepalive,
        other => panic!("未知帧类型 {other}"),
    }
}

/// 把 `start-end` 形式的区间表解析成 `(usize, usize)` 列表（半开区间）。
pub fn parse_ranges(spec: &str) -> Vec<(usize, usize)> {
    spec.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            let (start, end) = part.split_once('-').expect("区间格式必须是 start-end");
            let start: usize = start.trim().parse().expect("区间起点");
            let end: usize = end.trim().parse().expect("区间终点");
            assert!(start <= end, "区间起点不得大于终点：{part}");
            (start, end)
        })
        .collect()
}

/// 把区间表渲染回 `"start-end,start-end"` 形式（用于失败信息）。
pub fn render_ranges(ranges: &[(usize, usize)]) -> String {
    ranges
        .iter()
        .map(|(start, end)| format!("{start}-{end}"))
        .collect::<Vec<_>>()
        .join(",")
}

/// 把 `ranges` 覆盖到的字节**清零**（其余字节原样保留）。
///
/// 用于「canonical 形式」：握手报文里依赖墙钟的区段被清零后，
/// 剩下的字节才是**跨进程逐字节可复现**的那部分。
pub fn zero_ranges(bytes: &[u8], ranges: &[(usize, usize)]) -> Vec<u8> {
    let mut out = bytes.to_vec();
    for (start, end) in ranges {
        assert!(*end <= out.len(), "区间越界：{start}-{end}");
        for byte in &mut out[*start..*end] {
            *byte = 0;
        }
    }
    out
}

/// 从布局说明里抽出 `bytes[A..B]` 形式的字节区间。
///
/// 向量的 `volatile_layout` 逐段说明「墙钟区段里各段是什么」；抽出区间后
/// 可与 `volatile_ranges` 交叉校验（防「说明说的位置」与「实际清零的位置」
/// 不一致）。
pub fn layout_byte_ranges(layout: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut rest = layout;
    while let Some(at) = rest.find("bytes[") {
        rest = &rest[at + "bytes[".len()..];
        let end = rest.find(']').expect("布局说明里的区间未闭合");
        let body = &rest[..end];
        rest = &rest[end + 1..];
        let (start, stop) = body.split_once("..").expect("区间格式必须是 A..B");
        out.push((
            start.trim().parse().expect("区间起点"),
            stop.trim().parse().expect("区间终点"),
        ));
    }
    out
}

/// 合并区间列表（相邻或重叠的区间合成一个，按起点排序）。
pub fn merge_ranges(ranges: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let mut sorted: Vec<(usize, usize)> = ranges.to_vec();
    sorted.sort_unstable();
    let mut out: Vec<(usize, usize)> = Vec::new();
    for (start, end) in sorted {
        match out.last_mut() {
            Some(last) if start <= last.1 => last.1 = last.1.max(end),
            _ => out.push((start, end)),
        }
    }
    out
}
