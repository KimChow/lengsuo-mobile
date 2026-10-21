//! 极小 JSON 工具：解析 + **规范化渲染**。
//!
//! ⛔ **不引入 `serde` / `serde_json`**（`ffi/Cargo.toml` 属共享区，本卡不许加依赖）。
//! 跨端向量本质就是一份纯 JSON 文本，移动端任何 JSON 库都能读；
//! 本模块只是移动端仓里「解析 + 自检格式漂移」用的读取器。
//! 范式来自源仓 `lengsuo-link-core/tests/vectors/json.rs`（同样不引 serde）。
//!
//! # 支持的子集
//!
//! 对象 / 数组 / 字符串 / 数字 / `true` / `false` / `null`。够向量文件用。
//!
//! # 为什么对象**保序**
//!
//! 对象用 `Vec<(String, Json)>` 保存，**保持键的书写顺序**，渲染时原样输出
//! ⇒ 「解析后重新渲染」能**逐字节**回到原文（跨端向量规范化校验的基础）。
//!
//! # 规范化渲染规则
//!
//! - 对象：`{` 换行，逐字段 `  "key": 值`，`,` 换行分隔，末尾 `}`；
//! - 数组：非空时 `[` 换行，逐元素单独一行，末尾 `]`；空数组写 `[]`；
//! - 字符串：`"` 包裹，必要时转义 `\" \\ \n \r \t \b \f` 与 `< 0x20` 的控制字符；
//! - 数字：保留**字面文本**（不重新格式化）；
//! - 整个文档**不以 `\n` 结尾**（本仓的 `frame_vectors.json` 抽取时已去除首尾空白）。
#![allow(dead_code)] // 每个测试二进制只用到其中一部分取值器。

/// 极小 JSON 值。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Json {
    /// `null`。
    Null,
    /// `true` / `false`。
    Bool(bool),
    /// 数字（保留**字面文本**，保证渲染回原样）。
    Num(String),
    /// 字符串。
    Str(String),
    /// 数组。
    Arr(Vec<Json>),
    /// 对象（**保持键的书写顺序**）。
    Obj(Vec<(String, Json)>),
}

impl Json {
    /// 取对象的某个字段（不存在返回 `None`）。
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Self::Obj(entries) => entries.iter().find(|(name, _)| name == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// 取对象的某个字段并断言存在。
    pub fn field(&self, key: &str) -> &Json {
        self.get(key)
            .unwrap_or_else(|| panic!("JSON 缺少字段 {key}"))
    }

    /// 取字符串值。
    pub fn as_str(&self) -> &str {
        match self {
            Self::Str(value) => value,
            other => panic!("期望字符串，实际 {other:?}"),
        }
    }

    /// 取无符号整数值。
    pub fn as_u64(&self) -> u64 {
        match self {
            Self::Num(text) => text.parse().expect("数字"),
            other => panic!("期望数字，实际 {other:?}"),
        }
    }

    /// 取数组切片。
    pub fn as_arr(&self) -> &[Json] {
        match self {
            Self::Arr(items) => items,
            other => panic!("期望数组，实际 {other:?}"),
        }
    }
}

/// 解析一段 JSON 文本（格式非法即 panic，测试里让失败直接暴露）。
pub fn parse(text: &str) -> Json {
    let chars: Vec<char> = text.chars().collect();
    let mut parser = Parser {
        chars: &chars,
        at: 0,
    };
    parser.skip_ws();
    let value = parser.value();
    parser.skip_ws();
    assert_eq!(parser.at, parser.chars.len(), "JSON 结尾有多余内容");
    value
}

/// 极小 JSON 解析器（递归下降，只覆盖上面列出的子集）。
struct Parser<'a> {
    chars: &'a [char],
    at: usize,
}

impl Parser<'_> {
    /// 跳过空白。
    fn skip_ws(&mut self) {
        while self.at < self.chars.len() && self.chars[self.at].is_whitespace() {
            self.at += 1;
        }
    }

    /// 看下一个字符（不消费）。
    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    /// 期待并消费下一个字符。
    fn expect(&mut self, ch: char) {
        assert_eq!(self.peek(), Some(ch), "JSON 语法错误：期待 {ch}");
        self.at += 1;
    }

    /// 消费一个字符（结尾即 panic）。
    fn take(&mut self) -> char {
        let ch = self.peek().expect("JSON 语法错误：内容意外结束");
        self.at += 1;
        ch
    }

    /// 解析一个值。
    fn value(&mut self) -> Json {
        self.skip_ws();
        match self.peek() {
            Some('{') => self.object(),
            Some('[') => self.array(),
            Some('"') => Json::Str(self.string()),
            Some('t') => self.literal("true", Json::Bool(true)),
            Some('f') => self.literal("false", Json::Bool(false)),
            Some('n') => self.literal("null", Json::Null),
            Some(_) => self.number(),
            None => panic!("JSON 语法错误：内容意外结束"),
        }
    }

    /// 消费一个字面量。
    fn literal(&mut self, text: &str, value: Json) -> Json {
        for ch in text.chars() {
            self.expect(ch);
        }
        value
    }

    /// 解析数字（保留字面文本，不做重新格式化）。
    fn number(&mut self) -> Json {
        let start = self.at;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || matches!(ch, '-' | '+' | '.' | 'e' | 'E') {
                self.at += 1;
            } else {
                break;
            }
        }
        assert!(self.at > start, "JSON 语法错误：空数字");
        Json::Num(self.chars[start..self.at].iter().collect())
    }

    /// 解析字符串。
    fn string(&mut self) -> String {
        self.expect('"');
        let mut out = String::new();
        loop {
            let ch = self.take();
            match ch {
                '"' => break,
                '\\' => {
                    let escaped = self.take();
                    out.push(match escaped {
                        '"' => '"',
                        '\\' => '\\',
                        '/' => '/',
                        'b' => '\u{0008}',
                        'f' => '\u{000c}',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        'u' => {
                            let mut code = String::new();
                            for _ in 0..4 {
                                code.push(self.take());
                            }
                            let value =
                                u32::from_str_radix(&code, 16).expect("\\u 转义必须是四位十六进制");
                            char::from_u32(value).expect("\\u 转义必须是合法码点")
                        }
                        other => panic!("JSON 语法错误：未知转义 \\{other}"),
                    });
                }
                other => out.push(other),
            }
        }
        out
    }

    /// 解析数组。
    fn array(&mut self) -> Json {
        self.expect('[');
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(']') {
            self.at += 1;
            return Json::Arr(items);
        }
        loop {
            items.push(self.value());
            self.skip_ws();
            match self.peek() {
                Some(',') => self.at += 1,
                Some(']') => {
                    self.at += 1;
                    break;
                }
                other => panic!("JSON 语法错误：数组里出现 {other:?}"),
            }
        }
        Json::Arr(items)
    }

    /// 解析对象（保持键顺序，拒绝重复键）。
    fn object(&mut self) -> Json {
        self.expect('{');
        let mut entries: Vec<(String, Json)> = Vec::new();
        self.skip_ws();
        if self.peek() == Some('}') {
            self.at += 1;
            return Json::Obj(entries);
        }
        loop {
            self.skip_ws();
            let key = self.string();
            assert!(
                !entries.iter().any(|(name, _)| *name == key),
                "JSON 语法错误：重复键 {key}"
            );
            self.skip_ws();
            self.expect(':');
            let value = self.value();
            entries.push((key, value));
            self.skip_ws();
            match self.peek() {
                Some(',') => self.at += 1,
                Some('}') => {
                    self.at += 1;
                    break;
                }
                other => panic!("JSON 语法错误：对象里出现 {other:?}"),
            }
        }
        Json::Obj(entries)
    }
}

/// 把 JSON 值渲染成**规范化**文本（2 空格缩进，**不含结尾换行**）。
pub fn render(value: &Json) -> String {
    let mut out = String::new();
    render_into(value, 0, &mut out);
    out
}

/// 渲染辅助。
fn render_into(value: &Json, indent: usize, out: &mut String) {
    let pad = "  ".repeat(indent);
    let inner_pad = "  ".repeat(indent + 1);
    match value {
        Json::Null => out.push_str("null"),
        Json::Bool(true) => out.push_str("true"),
        Json::Bool(false) => out.push_str("false"),
        Json::Num(text) => out.push_str(text),
        Json::Str(text) => out.push_str(&render_string(text)),
        Json::Arr(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push_str("[\n");
            for (index, item) in items.iter().enumerate() {
                out.push_str(&inner_pad);
                render_into(item, indent + 1, out);
                if index + 1 < items.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&pad);
            out.push(']');
        }
        Json::Obj(entries) => {
            if entries.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push_str("{\n");
            for (index, (key, item)) in entries.iter().enumerate() {
                out.push_str(&inner_pad);
                out.push_str(&render_string(key));
                out.push_str(": ");
                render_into(item, indent + 1, out);
                if index + 1 < entries.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&pad);
            out.push('}');
        }
    }
}

/// 渲染 JSON 字符串（含必要的转义）。
fn render_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
