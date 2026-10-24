# mb-ios-conv-list 报告 —— iOS 会话列表：本地/云端双轨分组（SwiftUI）

> 卡：`mb-ios-conv-list`　lane：`ios`　worktree：`~/lengsuo-mobile-dev/worktrees/mb-ios-conv-list`
> 结论：**已完成并真跑通过**（`swift build` exit **0** / `swift test` exit **0**，
> `Executed 56 tests, with 0 failures`，其中本卡新增 `ConvListTests` **4 用例**）
> 本卡 **3 commit**，只碰 `ios/Sources/LengsuoMobile/Conversation/`（新建目录）
> + `ios/Tests/LengsuoMobileTests/ConvListTests.swift` + 本报告。
> 存在 **2 项偏差声明**（§五：`sample` 可见性、devguide 文件不在本 worktree），需主理人知悉。

---

## 〇、交付物

| 文件 | 行数 | 说明 |
| --- | --- | --- |
| `ios/Sources/LengsuoMobile/Conversation/ConversationModel.swift` | 41 | `ConvTrack` + `Conversation` + `groupConversations`（新建） |
| `ios/Sources/LengsuoMobile/Conversation/ConversationListView.swift` | 41 | 双 Section 视图 + 写死样例（新建） |
| `ios/Tests/LengsuoMobileTests/ConvListTests.swift` | 27 | 4 用例（新建） |
| `outputs/mb-ios-conv-list-report.md` | — | 本报告 |

单文件均 ≤400 行（最大 41）。零新依赖；未碰共享区（`Cargo.toml` / `Cargo.lock` /
`rust-toolchain.toml` / `scripts` / `.github`）/ `android/` / `ffi/` / `Package.swift` /
`Shell/` / `Home/` / `Me/` / `Discover/` / `Chat/` / `Scheduled/` / `Automation/` /
`Drawer/` / `Theme/`。

commit：

```
54cb2bd feat(ios/conversation): Conversation 双轨模型 + 分组纯函数
3e16941 feat(ios/conversation): ConversationListView 双 Section + ConvListTests 4 用例
docs(ios/conversation): mb-ios-conv-list 卡报告（本 commit）
```

---

## 一、三前缀逐字对照表

来源 = 卡面 §1.1「双轨缓存」**原文串**（用户派卡正文逐字摘录自 `android-devguide/04-对话运行时.md`
§2.2）。原文串逐字：

> 「双轨缓存：`/chats/local_*` 与 `/chats/cloud_*` 前缀（串证据），本地草稿会话与云端会话分开；
> 后台 Agent 会话独立前缀 `/chats/local_bga_`。」

| # | 轨道 | 04§2.2 原文前缀（权威） | iOS `ConvTrack.prefix` 字面量 | 逐字一致 |
| --- | --- | --- | --- | --- |
| 1 | `local` | `/chats/local_` | `"/chats/local_"` | ✅ |
| 2 | `cloud` | `/chats/cloud_` | `"/chats/cloud_"` | ✅ |
| 3 | `bga` | `/chats/local_bga_` | `"/chats/local_bga_"` | ✅ |

> 上述 3 条同时被 `ConvListTests.testPrefixesVerbatim` 以字面量断言锁死（改一个字即测试红）。
> 前缀**不含** 04§2.2 原文里的通配符 `*`（原文 `/chats/local_*` 的 `*` 是"该前缀开头"的通配写法）；
> iOS 侧落为前缀常量 `/chats/local_`，`Conversation.id.hasPrefix(...)` 即等价语义。

## 二、分组规则声明

**1）轨道推断顺序（关键：`local_bga_` 是 `local_` 的子前缀）**：`/chats/local_bga_` 以
`/chats/local_` 开头，若先判 `local` 则 bga 会话会被误判为本地普通会话。故
`Conversation.track` 的判定顺序为 **bga → cloud → 兜底 local**（`ConversationModel.swift:29-33`），
由 `ConvListTests.testTrackInference` 覆盖（含 `local_bga_9` 判为 `.bga`）。

**2）分组归属**：`groupConversations` 纯函数 ⇒
- **本地组** = `track == .local || track == .bga`（**bga 归入本地 Section**）；
- **云端组** = `track == .cloud`。

由 `ConvListTests.testGrouping`（样例 3 条 ⇒ 本地 2 / 云端 1）与
`ConvListTests.testBgaGoesToLocalSection`（bga 单独输入 ⇒ 落在 `.local`）覆盖。

**3）页面形态**：`ConversationListView` = 两个 `Section`（「本地」/「云端」）；bga 会话在本地
Section 内带「后台」标记（`Text("后台").font(.caption2)` + `.quaternary` 胶囊底），逐字照抄卡面 §2.2。

**4）样例数据（写死）**：

| id | title | track |
| --- | --- | --- |
| `/chats/local_draft_1` | 本地草稿：周报 | local |
| `/chats/local_bga_9` | 后台任务：数据整理 | bga（本地 Section + 后台标记） |
| `/chats/cloud_42` | 云端：旅行计划 | cloud |

> 卡面前置提及可复用 `DrawerCopy.sampleConversations`（`["hello", "周报整理", "旅行计划"]`）；
> 但卡面 §2.2 **逐字代码**明确给出本卡自己的 id/标题样例矩阵（含 `local_`/`local_bga_`/`cloud_`
> 三种前缀），为验证分组规则必需。本卡**按 §2.2 逐字照抄**使用该矩阵，**未**改动 `Drawer/`（禁区）。

## 三、范围裁剪 + P2 声明

**「Centrifugo 同步属 P2」**：04§2.2 / 规则 **R04-2b** 要求「多端同步走 Centrifugo 事件，不靠轮询」。
本卡**不做任何同步**（⛔ 无网络拉取、⛔ 无 Centrifugo 接入、⛔ 无轮询）——仅做**本地纯函数分组 +
静态展示**。Centrifugo 事件同步属 **P2**。

**「真实列表接口属 P2」**：真实会话列表接口 `/v2/user/cloudagent` 属 **P2**。本卡数据 =
**写死样例**（`ConversationListView.sample`），由 `public init(conversations:)` 暴露注入点，
后续卡接线零改动即可替换为真实数据源。

**未接线宿主**：本卡交付**独立** `ConversationListView`，**未修改**任何宿主页（`Shell/` 等为禁区）。
注入点：`public init(conversations: [Conversation] = .sample)`。

## 四、真跑验收输出

```console
$ cd ios && swift build
Building for debugging...
[0/1] Write swift-version-33747A42983211AE.txt
Build complete! (0.19s)
$ echo $?
0
```

```console
$ cd ios && swift test
Test Suite 'ConvListTests' started at 2026-10-24 23:38:06.738.
Test Case '-[LengsuoMobileTests.ConvListTests testBgaGoesToLocalSection]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ConvListTests testGrouping]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ConvListTests testPrefixesVerbatim]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ConvListTests testTrackInference]' passed (0.000 seconds).
Test Suite 'ConvListTests' passed at 2026-10-24 23:38:06.739.
	 Executed 4 tests, with 0 failures (0 unexpected) in 0.001 (0.001) seconds
...
Test Suite 'All tests' passed at 2026-10-24 23:38:06.772.
	 Executed 56 tests, with 0 failures (0 unexpected) in 0.031 (0.038) seconds
$ echo $?
0
```

- `swift build` exit **0**；`swift test` exit **0**；全包 `Executed 56 tests, with 0 failures`（含既有 52 + 本卡 4）。
- 本卡 **4 用例全绿**（非零用例，未出现「零用例 exit 0」式空跑）。
- 宿主 = macOS（`swift build` 默认，Swift 5.10 / Xcode 15.4）；**未做** iOS 模拟器/真机构建，
  **不声称真机**。

## 五、⚠️ 偏差声明（2 项，需主理人知悉）

**偏差 1｜`sample` 可见性：`static let` ⇒ `public static let`（必要，否则不可编译）。**

卡面 §2.2 逐字代码为：

```swift
public init(conversations: [Conversation] = ConversationListView.sample) {
static let sample: [Conversation] = [ ... ]
```

`public init` 的**默认实参**要求所引用符号可见性 ≥ `public`；真跑报错（实测）：

```
error: static property 'sample' is internal and cannot be referenced
       from a default argument value
note: static property 'sample' is not '@usableFromInline' or public
```

处置 = **最小修正**：`static let sample` ⇒ `public static let sample`（语义不变，
`@testable import` 与外部初始化均照常工作），并在源文件就地留注释说明。其余 §2.2/§2.1/§2.3
代码（含中文注释、`.quaternary` 胶囊、`.font(.caption2)`、`lineLimit(1)`、双 `Section`）
**逐字照抄未改**。

**偏差 2｜上位文档 `android-devguide/04-对话运行时.md` 不在本 worktree，前缀权威取自卡面 §1.1 原文串。**

实查：本 worktree / 主仓 / `~/lengsuo-mobile-dev` 下**均无** `android-devguide/` 目录，
`git log --all` 亦无该路径（该文档不在本仓版本库内）。因此本报告 §一 的「04§2.2 原文」
**无法实读该 md 文件**，权威串 = **用户派卡正文 §1.1 逐字摘录**（该摘录本身即标注「原文串逐字」）。
卡面 §1.1 与 §2.1 代码内的三前缀**互相一致**（`/chats/local_`、`/chats/cloud_`、`/chats/local_bga_`），
本卡据此对齐并由测试锁死。**若主理人可提供 devguide 文件，建议复核一次。**

## 六、参考二进制 / 互通说明（不适用）

卡面「参考二进制」「`#[ignore]` 用例必须用 `--ignored` 实跑并贴退出码」为 **Rust/FFI 线**要求。
本卡为**纯 SwiftUI 视图 + 纯函数模型 + 常量样例**：不涉及 Rust crate、不跨语言边界、**无互通面**
⇒ **无参考二进制可用、无 `#[ignore]` 用例**（故不存在「用自家 roundtrip 冒充互通」的情形，本卡无互通宣称）。

## 七、红线自查

| 项 | 结果 |
| --- | --- |
| 单文件 ≤400 行 | ✅ 最大 41 |
| ⛔ push / merge / 改主仓 | ✅ 未执行，仅本地 3 commit |
| 只碰 `Conversation/` + `ConvListTests.swift` + 本报告 | ✅ `git status` 干净；`git show --stat` 仅上述文件 |
| ⛔ 共享区（Cargo/scripts/.github）| ✅ 未碰；⛔ `Package.swift` 未碰（只读） |
| ⛔ 零新依赖 | ✅ `Package.swift` `dependencies: []` 未变 |
| ⛔ 前缀逐字 | ✅ §一 表 3 条，测试 `testPrefixesVerbatim` 锁死 |
| ⛔ 网络拉取 | ✅ 无网络代码；数据 = 写死样例 |
| 至少 2 commit | ✅ 3 commit（① 模型 ② 视图+测试 ③ 报告） |
| 真跑验收 | ✅ `swift build` exit 0 / `swift test` exit 0 / 56 tests 0 failures |
| ⛔ 不声称真机 | ✅ 仅 macOS 宿主 swift 构建，未做真机/模拟器 |