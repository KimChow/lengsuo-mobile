# 跨端帧向量：来源与本卡报告

## 1. 向量来源三元组（必须与 `ffi/Cargo.toml` 的 pin 一致）

- 仓库：`KimChow/lengsuo-network`（公开）
- 路径：`crates/lengsuo-link-core/tests/frame_vectors.rs`
- revision：`9b4e7aa86476acc4a697d2f1fbf8c090987f6922`
- 抽取日期：2026-10-21

⚠️ **不变式**：`frame_vectors.json` 来自上述 revision；该 revision 必须与
`ffi/Cargo.toml` 里 `lengsuo-link-core` 的 `rev` **逐字符相同**。
本卡没有改动 `ffi/Cargo.toml` / `ffi/Cargo.lock`（共享区），两者当前 pin 一致。

## 2. 抽取方式

用 `git show 9b4e7aa86476acc4a697d2f1fbf8c090987f6922:crates/lengsuo-link-core/tests/frame_vectors.rs`
取出该 rev 的原文（**只读，没有 checkout、没有改源仓**），再用脚本定位
`const VECTORS_JSON: &str = r##"…"##;` 的 raw string 边界，把 JSON 文本
**精确抽出**：只做首尾空白去除，**内部一个字节都未改**，写成本目录
`frame_vectors.json`。

⛔ 没有用源仓的 `generator.rs` 重新生成，也没有手抄——这是一份**搬运**，
搬运的必须是源仓仓内那一份。抽取结果 1918 字节，解析后重新渲染与文件
逐字节相同（由测试 `vectors_json_is_canonical` 守住）。

## 3. 本卡新增文件

- `ffi/tests/vectors/frame_vectors.json` —— 从源仓精确抽取的向量原文。
- `ffi/tests/vectors/json.rs` —— 极小 JSON 解析器（对象/数组/字符串/数字/
  布尔/null，**对象保序**、**数字保留字面文本**，不引 serde）。
- `ffi/tests/vectors/mod.rs` —— 模块入口 + 内嵌原文 + 十六进制工具 + 向量模型。
- `ffi/tests/vectors/SOURCE.md` —— 本文件。
- `ffi/tests/cross_end_vectors.rs` —— 对拍测试。

## 4. 关键取舍与理由

1. **不新增依赖**：`ffi/Cargo.toml` 是共享区，不引 `serde`/`serde_json`；
   JSON 读取用极小解析器，范式同源仓 `tests/vectors/json.rs`。
2. **对象保序 + 数字保字面**：这是「解析后重新渲染 == 原文」规范化校验的前提。
3. **`include_str!` 内嵌原文 + 运行期读盘双重校验**：内嵌避免测试依赖工作目录，
   读盘用例再证「文件非空、与内嵌逐字节相同」，满足门禁的「向量文件非空」断言。
4. **帧头长 / 传输层开销从 authoritative 字节推导**：不在测试里写死 20 / 32 /
   1400 / 1368，上限一致性按「从 `MAX_FRAME_LEN` 推」的规则做。

## 5. 对拍结果（真实数字）

- case 总数：**5**；全部通过：**5**；失败：**0**。
- 逐条结论（每条的帧头字段、态类型、序号、载荷切分、`encode` 逐字节还原、上限
  全部通过）：

| case | frame_type | 结论 |
|---|---|---|
| `data-ascii` | data | ✅ 帧类型/序号/载荷/整帧字节一致 |
| `data-binary` | data | ✅ 帧类型/序号/载荷/整帧字节一致 |
| `control-utf8` | control | ✅ 帧类型/序号/载荷/整帧字节一致 |
| `keepalive-empty` | keepalive | ✅ 帧类型/序号/载荷/整帧字节一致 |
| `data-seq-max` | data | ✅ 帧类型/序号/载荷/整帧字节一致 |

- 规范化校验：`vectors_json_is_canonical` ✅（解析后重新渲染与原文逐字节相同）。

## 6. 上限一致性结论

- `MAX_FRAME_LEN` 直接由门面 `frame` 模块复用 link-core 的定义（源仓
  `frame/mod.rs`：`MAX_FRAME_LEN = MAX_TRANSPORT_PACKET_LEN`，即 1420）。
- 由 authoritative 字节推导出帧头长 = **20**、传输层开销 = **32**（跨 5 条向量一致）。
- 测试断言：`MAX_FRAME_PAYLOAD_LEN == MAX_FRAME_LEN − 20 == 1400`、
  `MAX_PLAIN_LEN == MAX_FRAME_PAYLOAD_LEN − 32 == 1368`，与源仓同一 rev 的
  推导公式一致；且每条向量的整帧 / 明文都不超上限。

## 7. 对拍边界与本卡未做（诚实声明）

- **本卡只做 Rust 侧跨端对拍**：**无 iOS 构建、无 Android 构建、无真机验证**
  （47 无 Xcode/macOS、无 Android SDK/NDK）。未声称已验证 iOS / Android。
- **明文↔密文的 AEAD 往返不在本卡范围**（见下一条语义冲突）；本卡对拍的是
  门面层确实暴露的无状态帧编解码那一部分。
- ⛔ 源仓 `frame_vectors.rs` **不含错误向量**（错误向量在源仓
  `tests/vectors/error_vectors.rs`，属另一份更全的向量集）⇒ 本卡**跳过**
  `CoreError::stable_name()` 错误路径对拍，**没有**自造错误向量。
- 未完成：绑定层落地（等 3 项拍板：绑定依赖 / `panic` 设置 / codegen 步骤）、
  iOS / Android 壳、Swift / Kotlin 侧对拍。

## 8. 语义冲突：逐字节对拍与门面 API 的边界（写进报告，不自行选路）

卡片 §2.3 预期用 `ffi/src/core` 的 API **从明文复现 `frame_hex`**、并用
`decode_exact` **解回明文**。核实源码后确认这两点在门面层**做不到**，原因：

1. `frame::encode` 只做「帧头 + 载荷」拼接，**不做 AEAD 封印**；要由明文得到
   密文载荷，需要 `lengsuo_wireguard::transport::Sender`。
2. `Session::seal_frame` / `open_frame` 能做 AEAD，但其传输上下文**只能由握手
   派生**（密钥由 Noise IK 派生，非向量里的 0x5A 测试密钥）；且握手 transcript
   含墙钟时间戳，跨进程不可复现。门面没有 API 能拿「测试密钥 + 接收者标识 +
   起始计数器」直接构造 `Sender` / `Receiver`。
3. 门面**刻意不依赖、也不透出 `lengsuo-wireguard`**（依赖纪律），移动端测试
   无法直接触碰 `Sender` / `Receiver` 的类型。

⇒ 因此：

- 本卡对拍**能**覆盖的部分（frame 格式边界：`decode_exact` 解析 authoritative
  字节、`encode` 逐字节还原、帧头字段、上限推导）已全部实现并通过；
- **未**覆盖「明文 → 密文 payload」与「密文 payload → 明文」：这需要要么让
  门面暴露一个**测试专用**的传输上下文构造入口，要么把 `lengsuo-wireguard`
  以 dev-dependency 引入移动端测试（都会动共享区 / 门面，本卡没有擅自改）。

⚠️ 如何处理，请主理人定：本卡**没有**为「让向量变绿」去改门面层，也没有
新增依赖；gate 已在不改动共享区的前提下做到绿灯。

## 9. 门禁结果

- `ffi/`（生产包）：`cargo fmt --all -- --check` ✅、
  `cargo clippy --workspace --all-targets --locked -- -D warnings` ✅、
  `cargo test --workspace --locked` ✅（8 个测试目标全部 `0 failed`）、
  `cargo build --workspace --locked` ✅。
- `ffi/spike/`（隔离 workspace）：fmt ✅、clippy ✅、`cargo test --workspace --locked` ✅
  （12 + 12 `0 failed`）。
- 脚本门禁：`python3 scripts/check_line_limit.py` ✅ 硬上限失败 0，
  软目标超 400 的 2 个文件均为**既有** `scripts/*.py`（非本卡文件）；
  `python3 scripts/check_no_source_leak.py` ✅（未发现参考来源泄漏）。