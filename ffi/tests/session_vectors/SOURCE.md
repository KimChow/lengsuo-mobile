# 跨端 L2 会话向量：来源与本卡报告

## 1. 向量来源三元组（必须与 `ffi/Cargo.toml` 的 pin 一致）

- 仓库：`KimChow/lengsuo-network`（GitHub，公开）
- 路径：`crates/lengsuo-link-core/tests/vectors/session_data.rs`
- revision：`9b4e7aa86476acc4a697d2f1fbf8c090987f6922`
- 抽取日期：2026-10-22

⚠️ **不变式**：`session_vectors.json` 来自上述 revision；该 revision 必须与
`ffi/Cargo.toml` 里 `lengsuo-link-core` 的 `rev` **逐字符相同**。
本卡**没有**改动 `ffi/Cargo.toml` / `ffi/Cargo.lock`（共享区），
两者当前 pin 一致（grep 结果见 §7）。

## 2. 抽取方式

用 `git show 9b4e7aa86476acc4a697d2f1fbf8c090987f6922:crates/lengsuo-link-core/tests/vectors/session_data.rs`
取出该 rev 的原文（**只读，没有 checkout、没有改源仓**），再用脚本定位
`pub const SESSION_VECTORS_JSON: &str = r##"…"##;` 的 raw string 边界，把
JSON 文本**精确抽出**：**只做首尾空白去除，内部一个字节都未改**，写成本目录
`session_vectors.json`。

⛔ 没有用源仓的 `generator::regenerate_session_vectors_json`（`#[ignore]`）
重新生成，也没有手抄——这是一份**搬运**，搬运的必须是源仓仓内那一份。
（重新生成是**源仓**的职责：源仓的 `rebuild.rs` 要直接碰
`lengsuo-wireguard` 的传输层，而移动端门面**刻意不依赖**它。）

抽取结果 **9333 字节**（原始 raw string 9334 字节，差 1 字节 = 末尾换行）。
校验：解析后重新渲染与文件**逐字节相同**（由测试
`session_vectors_json_is_canonical` 守住）。

**逐字节一致证据**（用脚本从源仓重新抽一次，与磁盘文件对拍）：

```console
$ python3 -I /tmp/extract_check.py     # git show <rev>:…/session_data.rs → 抽 raw string → 与磁盘文件比
extracted bytes : 9333
disk bytes      : 9333
extracted sha256: 6d763d97f5c9e6c3ff8d5fcd874bcb2b973dc0dbbd673f690945e9e0f7ca64a6
disk sha256     : 6d763d97f5c9e6c3ff8d5fcd874bcb2b973dc0dbbd673f690945e9e0f7ca64a6
BYTE-IDENTICAL  : True
```

另：磁盘文件与编译期 `include_str!` 内嵌文本的一致性由测试
`session_vector_file_is_present_and_matches_the_embedded_text` 守住。

## 3. 本卡新增文件

| 文件 | 内容 |
|---|---|
| `ffi/tests/session_vectors/session_vectors.json` | 从源仓精确抽取的向量原文（9333 字节）。 |
| `ffi/tests/session_vectors/mod.rs` | 模块入口 + 对外再导出。 |
| `ffi/tests/session_vectors/model.rs` | 向量数据模型 + 解析（`links[]` / `rounds[]` / `frames[]`）。 |
| `ffi/tests/session_vectors/util.rs` | 十六进制、区间、帧类型名等机械换算工具。 |
| `ffi/tests/session_vectors/runner.rs` | 用**移动端门面层**跑握手 + 封帧，产出「应得的字节」。 |
| `ffi/tests/session_vectors/SOURCE.md` | 本文件。 |
| `ffi/tests/cross_end_session_vectors.rs` | L2 会话对拍测试入口。 |

## 4. 关键取舍与理由

### 4.1 JSON 读取器：**复用** `tests/vectors/json.rs`，不写第二份

先读了 `ffi/tests/vectors/json.rs`（`mb-ffi-vectors` 交付）的公开面：
`parse` / `render` / `Json` 变体（`get` / `field` / `as_str` / `as_u64` /
`as_arr`）。

**结论：能复用。** 该读取器是**与向量形状无关**的极小 JSON DOM 解析器
（对象**保序**、数字**保留字面文本**），L2 会话向量的形状
（`links[]` / `rounds[]` / `frames[]` / `volatile_ranges` / `canonical_hex` …）
完全落在它支持的子集内 ⇒ 本卡用
`#[path = "../vectors/json.rs"] pub mod json;` 引入，
**不修改** `ffi/tests/vectors/json.rs` 本身（它不在本卡 touches 内），
也**不引入** `serde` / `serde_json`（`ffi/Cargo.toml` 属共享区，不许加依赖）。

⚠️ 一处**必须注意的差异**：源仓的 `render` **以 `\n` 结尾**，本仓的
`render` **不以 `\n` 结尾**（`mb-ffi-vectors` 交付时按「抽取时已去除首尾空白」
的口径改的）。本卡抽出的 `session_vectors.json` 同样**不含末尾换行**
⇒ 与本仓 `render` 的口径一致，规范化校验成立。

### 4.2 墙钟区段：**从向量读**，不写死

握手报文里有一段**依赖墙钟**的字节（TAI64N 时间戳密文 + 覆盖它的 MAC1），
跨进程必然不同 ⇒ 向量把那段**声明**出来（`volatile_ranges`），
并给出**清零后**的 `canonical_hex`。

- ⛔ 本卡**不**把 `88-132` / `44-76` 写死成常量（向量才是权威）；
- 清零实现（[`util::zero_ranges`]）与源仓 `vectors/mod.rs::zero_ranges`
  **语义一致**：把区间覆盖到的字节**替换为 0**，其余字节原样保留；
- 另加一条**自洽性**用例（`volatile_ranges_match_the_declared_layout`）：
  把 `volatile_layout` 里 `bytes[A..B]` 形式的说明解析出来，与
  `volatile_ranges` 合并后**必须覆盖同一段字节** ⇒ 防「说明说的位置」
  与「实际清零的位置」不一致。

### 4.3 对拍对象：**移动端门面层**，不是源仓内部模块

源仓的重算引擎 `rebuild.rs` 在**传输层**（`lengsuo-wireguard`）重算握手、
在**会话层**（`lengsuo-link-core`）重算世代号与帧。

本卡走**移动端门面层**（`lengsuo_mobile_ffi::core::Session`）做同样的事：
门面把 link-core 包成**绑定无关的稳定 API**，正是移动端将来真正调用的那一层
⇒ 对拍的对象必须是它。门面**刻意不依赖 `lengsuo-wireguard`**，
所以门面能做的「握手 + 世代号 + 密文帧」恰好就是本卡要验的三件事。

### 4.4 重连轮必须等**墙钟**走过一格

会话层有两处**依赖时间**的检查，重连轮必须跨过：

- **握手洪水窗口**（20ms，两次「消费 initiation」之间的最小间隔）——
  看的是调用方传入的 `Instant`（单调钟）；
- **TAI64N 时间戳**（~16.78ms 刻度；重连的 initiation 必须**严格晚于**
  上一轮，否则被**重放检查**拒绝）—— 看的是**墙钟**。

⇒ [`runner::advance_round`] 等到「单调钟 ≥ 50ms」**且**「墙钟刻度严格晚于
上一轮」。墙钟刻度用与源仓 `tai64n_now()` 同公式的探针计算
（`BE64(TAI64 秒) || BE32(纳秒 &! 0x00FF_FFFF)`）——它只是**测试夹具**，
不参与任何协议逻辑。

### 4.5 帧的**方向**与**发送方**必须对上

向量每帧带 `direction`（`initiator_to_responder` / `responder_to_initiator`）。
本卡按方向选发送方，**⛔ 不许**把帧全从一个方向发；
并另加一条用例（`frames_are_sealed_in_both_directions`）保证两个方向都出现过。

## 5. 对拍判据（与源仓 `consistency.rs` 逐条等价）

| 本卡用例 | 源仓对应用例 | 判据 |
|---|---|---|
| `handshake_messages_match_the_canonical_bytes` | `every_handshake_message_canonical_form_is_reproducible` | 墙钟区段清零后与 `canonical_hex` 逐字节相同 |
| `generation_matches_and_advances_by_one_on_reconnect` | `generation_is_reproducible_and_advances_by_one` | 世代号 == 向量；首轮 = 0；重连轮 = 上一代 + 1 |
| `sealed_frames_match_the_expected_frame_bytes` | `every_session_vector_is_reproducible`（帧部分） | 封出的整帧与 `frame_hex` 逐字节相同 |
| `opened_frames_decode_back_to_the_expected_plaintext` | `every_expected_frame_decodes_back_to_its_plaintext` | 解出的明文 / 帧类型 / 序号与向量一致 |
| `session_vectors_json_is_canonical` | `session_vectors_json_is_canonical` | 解析后重新渲染 == 原文 |
| `frame_counters_match_per_direction_send_order` | `frame_counters_match_per_direction_send_order` | 计数器 == 该方向的发送顺序 |
| `transport_keys_are_directional` | `every_transport_key_is_reproducible_and_directional`（方向性部分） | 发起方 send == 响应方 receive，反之亦然 |
| `links_do_not_share_key_material` | `links_do_not_share_key_material` | 不同链路不共用密钥材料 |
| `volatile_ranges_match_the_declared_layout` | `volatile_layout_matches_the_code_constants`（加强版） | 声明的区段与布局说明自洽 |

⚠️ 源仓的 `every_transport_key_is_reproducible_and_directional` 里
「传输密钥可复现」那一半需要**传输层**的密钥导出
（`lengsuo-wireguard` 的 `TransportKeys`），移动端门面**刻意不透出**它
⇒ 本卡只校验**方向性**（向量自洽性），并靠「密文帧整帧逐字节相同」
**间接**证明「双方用的确实是同一套派生密钥」（密钥不同 ⇒ 密文必不同）。

## 6. 对拍结果（真实数字）

向量规模：**2 条链路 × 共 3 轮 × 共 7 帧**（非空子集，⛔ 零用例的 exit 0 不算通过）。

| 链路 | 轮次 | 握手 canonical（initiation/response） | 世代号 | 密文帧整帧 | 解密回读 |
|---|---|---|---|---|---|
| link-a | 1 | ✓ / ✓ | ✓ | ✓（4/4 帧） | ✓（4/4 帧） |
| link-a | 2 | ✓ / ✓ | ✓ | ✓（2/2 帧） | ✓（2/2 帧） |
| link-b | 1 | ✓ / ✓ | ✓ | ✓（1/1 帧） | ✓（1/1 帧） |

- **握手 canonical 对拍**：3 轮 × 2 条报文（initiation 148B / response 92B）= **6/6 通过**；
- **世代号对拍**：3/3 通过（link-a 轮 1 = 0，轮 2 = 1 = 上一代 + 1；link-b 轮 1 = 0）；
- **密文帧整帧对拍**：7/7 通过；
- **解密回读对拍**：7/7 通过；
- **失败总数：0**。

逐帧明细：

| 链路 | 轮 | 帧名 | 方向 | 类型 | seq | counter | 明文 | 整帧 |
|---|---|---|---|---|---|---|---|---|
| link-a | 1 | `i2r-data-ascii` | 发起→响应 | data | 1 | 0 | 8B | 60B |
| link-a | 1 | `r2i-data-reply` | 响应→发起 | data | 1 | 0 | 4B | 56B |
| link-a | 1 | `i2r-control` | 发起→响应 | control | 2 | 1 | 3B | 55B |
| link-a | 1 | `r2i-keepalive` | 响应→发起 | keepalive | 2 | 1 | 0B | 52B |
| link-a | 2 | `i2r-data-reconnect` | 发起→响应 | data | 3 | 0 | 14B | 66B |
| link-a | 2 | `r2i-data-reconnect-reply` | 响应→发起 | data | 3 | 0 | 2B | 54B |
| link-b | 1 | `i2r-data-binary` | 发起→响应 | data | 1 | 0 | 6B | 58B |

⚠️ **断言是严的**（做过两次**负向**验证，验完已把向量文件恢复原样、
sha256 复核为 `6d763d97…64a6`）：

- 把 `i2r-data-ascii` 的 `frame_hex` 翻一个 nibble ⇒
  `sealed_frames_match_the_expected_frame_bytes` **FAILED**，
  失败信息带 case 名 + 期望/实际字节
  （`链路 link-a 第 1 轮 帧 i2r-data-ascii：密文帧整帧字节不符（首个差异在偏移 0：期望 0c53010100000000，实际 4c53010100000000 …）`）；
- 把 link-a 轮 1 的 `initiation.canonical_hex` 翻一个 nibble ⇒
  `handshake_messages_match_the_canonical_bytes` **FAILED**，
  失败信息同样带 case 名 + 期望/实际字节
  （`链路 link-a 第 1 轮：initiation canonical 字节不符（首个差异在偏移 0：期望 1100000011111111，实际 0100000011111111 …）`）。

## 7. rev 不变式核对

```console
$ grep -n "lengsuo-link-core" ffi/Cargo.toml
7:description = "跨端链路协议栈的 Swift/Kotlin 绑定层：把 lengsuo-link-core 暴露为移动端可调用的稳定 ABI"
15:# ⚠️ 该仓的 crates/lengsuo-link-core 当前仍是占位实现（协议帧编解码在另卡进行中）。
16:lengsuo-link-core = { git = "https://github.com/KimChow/lengsuo-network.git", rev = "9b4e7aa86476acc4a697d2f1fbf8c090987f6922" }

$ grep -o 'rev = "[0-9a-f]*"' ffi/Cargo.toml
rev = "9b4e7aa86476acc4a697d2f1fbf8c090987f6922"
```

⇒ 与本文件 §1 记录的 revision `9b4e7aa86476acc4a697d2f1fbf8c090987f6922`
**逐字符相同**（本卡**未**改动 `ffi/Cargo.toml` / `ffi/Cargo.lock`）。

## 8. 诚实声明与本卡边界

- ⛔ 本卡**只做 Rust 侧对拍**：47 上**无 Xcode / macOS、无 Android SDK/NDK**
  ⇒ **无 iOS 构建、无 Android 构建、无真机验证**。
  **不得**据此声称已验证 iOS / Android。
- ⛔ 本卡**不做**：绑定层落地（uniffi / 手写 C ABI 需主理人拍板 3 项：
  新增绑定依赖 / `panic=abort`→`unwind` / codegen 步骤 + 生成物入库）、
  iOS 壳、Android 壳、Swift/Kotlin 侧对拍。
- ⛔ 本卡**没有**改动 `ffi/src/core/**`、`ffi/Cargo.toml`、`ffi/Cargo.lock`、
  `ffi/tests/vectors/**`、`ffi/tests/common/mod.rs`、`ffi/spike/**`
  —— 发现对拍不一致时**没有**去改门面层让它变绿。
- ⚠️ 移动端仓 `docs/` **只留本地不入库** ⇒ 本卡说明写在**本文件**（入库），
  **没有** `git add` 任何 `docs/` 路径。
- 向量里的密钥全是**测试专用假密钥**（`0x11` / `0x22` / `0x33` / `0x44` 模式字节），
  **不得**用于任何真实会话；本卡**没有**提交任何真实凭据 / 私钥 / 生产配置。

## 9. 跑门禁时的**环境**说明（不是代码问题，也没改仓库）

47 上跑 `ffi/spike` 那半边门禁时踩到一个**纯环境**障碍，本卡**没有**为此改动
任何仓库文件（`ffi/spike/**` 与共享区都不在本卡 touches 内）：

- 本机 **crates.io / rsproxy / tuna 全部网络不可达**（`curl` 全部超时）；
- `ffi/spike/Cargo.lock` 里 **24 个包**（`uniffi 0.32.2` 全家 + `askama` 系 +
  `toml` 系 + …）在本地 registry **只有解包后的源码**（`~/.cargo/registry/src/…`），
  既**没有** `.crate` 缓存文件，本地 index 缓存（rsproxy 的 `.cache/` 与
  tuna 的 git 快照）也**没有**这些版本的元数据 ⇒ 离线解析报
  `no matching package named 'uniffi' found`。

⇒ 处置：在**仓库外**（`/tmp/mb_ffi_session_vectors_env/`，不入库、也不在
worktree 里留下任何未跟踪文件）用本地已解包的源码做了一份
**vendor 目录源**（`cargo vendor` 的等价物，`.cargo-checksum.json` 按
`ffi/spike/Cargo.lock` 里记录的 checksum 写），再用**私有 `CARGO_HOME`**
（`config.toml` 里 `source.crates-io.replace-with`）把 crates-io 指向它
⇒ `ffi/spike` 的 fmt / clippy / test / build 全部 exit 0。

⚠️ 这个 vendor 目录是**环境脚手架**，与本卡的产物无关，也**没有**入库；
它**不改变** `ffi/spike/Cargo.lock` 里任何一个 checksum（锁文件里的
`uniffi 0.32.2` checksum `76407f5f396a2c949a069eff4a9fca9ce462410eb872bffef4c2b7b89804a77a`
与源仓一致，未被改动）。
