# mb-ios-chat-page 报告 —— iOS 对话页静态版：两态 + 顶栏/提示条/消息区/工具条/输入区（SwiftUI）

> 卡：`mb-ios-chat-page`　lane：`ios`　worktree：`~/lengsuo-mobile-dev/worktrees/mb-ios-chat-page`
> 结论：**已完成并真跑通过**（`swift build` exit 0 / `swift test` exit 0，
> `Executed 59 tests, with 0 failures`，其中本卡新增 `ChatPageTests` 7 用例全绿）
> 本卡 **3 commit**，只碰 `ios/Sources/LengsuoMobile/Chat/`（新建目录）+
> `ios/Tests/LengsuoMobileTests/ChatPageTests.swift` + 本报告。
> 存在 **1 项必要偏差**（§五-1：`Color(.secondarySystemBackground)` 在 macOS 宿主不可编译），
> 另有 **2 项与 android 源实现的差异**（§五-2/3）已声明，需主理人知悉。

---

## 〇、交付物

| 文件 | 行数 | 说明 |
| --- | --- | --- |
| `ios/Sources/LengsuoMobile/Chat/ChatCopy.swift` | 37 | 17 条文案 + `ChatPhase` 两态映射（新建） |
| `ios/Sources/LengsuoMobile/Chat/ChatPageView.swift` | 111 | 对话页静态视图 + `Bubble`（新建） |
| `ios/Tests/LengsuoMobileTests/ChatPageTests.swift` | 66 | 7 用例（新建） |

单文件均 ≤400 行（最大 111）。零新依赖（`Package.swift` `dependencies: []` 未变，且未修改）；
未碰共享区（`Cargo.toml` / `Cargo.lock` / `rust-toolchain.toml` / `scripts` / `.github`）、
`android/` / `ffi/` / `Package.swift` / `Shell/` / `Home/` / `Me/` / `Discover/` / `Scheduled/` /
`Automation/` / `Drawer/` / `Theme/`。

commit：

```
b26ff22 feat(ios/chat): ChatCopy 17 条文案 + ChatPhase 两态映射（ChatPage.kt 同语义）
c9d02b4 feat(ios/chat): ChatPageView 对话页静态版 + ChatPageTests 7 用例
（③）docs(ios/chat): mb-ios-chat-page 卡报告 —— 即本报告所在 commit，hash 见 `git log -1`
```

---

## 一、17 条 `chat_*` 文案逐字对照表

来源 = 本仓 `android/app/src/main/res/values/strings.xml` L49–65（**本卡实读该文件取值，非转抄任务书**）。

| # | strings.xml 名 | android 字面量（权威） | iOS `ChatCopy` 字面量 | 逐字一致 |
| --- | --- | --- | --- | --- |
| 1 | `chat_ai_banner` | `内容由 AI 生成` | `aiBanner = "内容由 AI 生成"` | ✅ |
| 2 | `chat_status_done` | `已完成` | `statusDone = "已完成"` | ✅ |
| 3 | `chat_status_thinking` | `正在思考 ⋯` | `statusThinking = "正在思考 ⋯"` | ✅ |
| 4 | `chat_toolbar_copy` | `复制` | `toolbarCopy = "复制"` | ✅ |
| 5 | `chat_toolbar_like` | `赞` | `toolbarLike = "赞"` | ✅ |
| 6 | `chat_toolbar_dislike` | `踩` | `toolbarDislike = "踩"` | ✅ |
| 7 | `chat_toolbar_read_aloud` | `朗读` | `toolbarReadAloud = "朗读"` | ✅ |
| 8 | `chat_toolbar_share` | `分享` | `toolbarShare = "分享"` | ✅ |
| 9 | `chat_top_new_chat` | `新建对话` | `topNewChat = "新建对话"` | ✅ |
| 10 | `chat_top_more` | `更多操作` | `topMore = "更多操作"` | ✅ |
| 11 | `chat_subtitle_mode` | `☁ 工作模式` | `subtitleMode = "☁ 工作模式"` | ✅ |
| 12 | `chat_input_placeholder_idle` | `发消息或按住说话...` | `inputPlaceholderIdle = "发消息或按住说话..."` | ✅ |
| 13 | `chat_input_placeholder_sending` | `发消息...` | `inputPlaceholderSending = "发消息..."` | ✅ |
| 14 | `chat_model_tier_idle` | `auto` | `modelTierIdle = "auto"` | ✅ |
| 15 | `chat_model_tier_sending` | `⚡ 快速 ▾` | `modelTierSending = "⚡ 快速 ▾"` | ✅ |
| 16 | `chat_input_voice` | `语音` | `inputVoice = "语音"` | ✅ |
| 17 | `chat_input_plus` | `+` | `inputPlus = "+"` | ✅ |

**机器校验（非目测）**：用脚本把 `strings.xml` 的 17 条 `chat_*` 取值与 `ChatCopy.swift` 的 17 个
属性做**逐字符等值比对**，输出 `xml chat_* count=17  swift props=17` / `MISMATCHES: 0`。
另有 `ChatPageTests.testSeventeenCopiesMatchAndroidStringsVerbatim` 以 17 条字面量断言锁死
（改一个字即测试红）。

**省略号差异按卡面保留**：状态行 `正在思考 ⋯`（U+22EF 单字符），两条占位符 `发消息...`
（半角三点 U+002E×3）。二者**不混用**，由 `testPlaceholderMapping` 注释 + 断言双重锁定。

## 二、两态映射表（权威 = `android/.../ui/chat/ChatPage.kt`）

`ChatPage.kt` 三个纯函数（`chatStatusLabel` / `chatModelTierLabel` / `chatPlaceholder`）
返回 `R.string` 资源 id，`ChatPhase { SENDING, DONE }`。iOS 以同语义 `ChatPhase { case sending, done }`
+ `statusLabel` / `modelTierLabel` / `inputPlaceholder` 三个计算属性实现（映射值直接取字面量，
无资源间接层 ⇒ 与 android 的资源 id 断言等价）：

| 映射函数（ChatPage.kt） | `SENDING`（android 资源 → 字面量） | `DONE`（android 资源 → 字面量） | iOS 属性 | 用例 |
| --- | --- | --- | --- | --- |
| `chatStatusLabel` | `chat_status_thinking` → `正在思考 ⋯` | `chat_status_done` → `已完成` | `ChatPhase.statusLabel` | `testStatusMapping` ✅ |
| `chatModelTierLabel` | `chat_model_tier_sending` → `⚡ 快速 ▾` | `chat_model_tier_idle` → `auto` | `ChatPhase.modelTierLabel` | `testModelTierMapping` ✅ |
| `chatPlaceholder` | `chat_input_placeholder_sending` → `发消息...` | `chat_input_placeholder_idle` → `发消息或按住说话...` | `ChatPhase.inputPlaceholder` | `testPlaceholderMapping` ✅ |

`ChatPage.kt` 中另有 `TOOLBAR_ICON_DP = 24` / `TOOLBAR_GAP_DP = 12` 与
`CHAT_TOOLBAR_ITEMS` 顺序（复制/赞/踩/朗读/分享）。iOS 侧工具条 5 键**顺序逐字对齐**
`CHAT_TOOLBAR_ITEMS`（`toolbarCopy → toolbarLike → toolbarDislike → toolbarReadAloud → toolbarShare`），
以 `ForEach([...], id: \.0)` 硬编码同序数组表达；键间距按卡面取 `spacing: 20`（见 §六 说明）。

## 三、布局对齐（§1.3）

| 区块 | 卡面 §1.3 要求 | iOS 实现 |
| --- | --- | --- |
| 顶栏 | 新建对话 + ☁ 工作模式 + 更多操作 | `topBar`：`Label(topNewChat, "plus.bubble")` / `Text(subtitleMode)` 副标题 / `Image("ellipsis")`（`topMore` 语义位） |
| AI 提示条 | 内容由 AI 生成 | `Text(ChatCopy.aiBanner)` 通栏 `.background(.bar)` |
| 消息区 | 右气泡=用户 / 左无边框正文=AI | `Bubble(isUser: true)` 右对齐圆角底；AI 正文裸 `Text` 左对齐无气泡 |
| 工具条 | 5 键，仅 DONE 态显示 | `if phase == .done { toolbar }` — 复制/赞/踩/朗读/分享 |
| 输入区 | `+` / 占位符两态 / 语音 | `inputBar`：`Text(inputPlus)` / `Text(phase.inputPlaceholder)` / `Text(inputVoice)` |

## 四、声明（本卡边界）

1. **不发送**：`ChatPageView` 无任何网络/发送路径，`phase` 为**入参**（`public init(phase:...)`），
   视图内无 `@State`、无定时器、无 `onSend` 回调 —— 两态纯 UI 状态机，由调用方给定。
2. **不流式**：无 SSE/流式增量渲染；AI 正文为一次性静态字符串（默认值 = 卡面示例句）。
3. **不录音**：`语音` 仅为静态文案 `Text`，无 `AVAudioRecorder` / 麦克风 / 权限申请。
4. **不接抽屉**：本卡未渲染 `DrawerView`、未修改 `Home/TasksHomePage`、未加 `.sheet` 等宿主接线
   （宿主接线属后续卡，对齐 `mb-ios-drawer` 的「不接线宿主」惯例）。顶栏「更多操作/新建对话」
   为无操作 `Button {}` 占位。
5. **输入恒为静态**：输入区占位符非 `TextField`，为静态 `Text` —— 无键盘、无编辑、无提交。

## 五、偏差声明（1 项必要 + 2 项与 android 源实现的差异）

### 1）⚠️ 必要偏差：`Color(.secondarySystemBackground)` 在 macOS 宿主不可编译

卡面 §2.2 逐字给了 `.background(isUser ? Color.accentColor.opacity(0.18) : Color(.secondarySystemBackground), ...)`。
`Color(_ uiColor: UIColor)` 是 **iOS 专有** init；而 `Package.swift` 的
`platforms: [.iOS(.v17), .macOS(.v13)]` 使本仓验收命令 `swift build` 在 **macOS 宿主**构建，
该颜色在 AppKit 侧不存在 ⇒ 照抄会编译失败。

处置：按**本仓既有惯例**（与 `Theme/LengsuoTheme.swift` 及上一张 iOS 卡的
`#if canImport(UIKit)` / `#elseif canImport(AppKit)` 写法一致）改写为平台分支：

```swift
private var aiBubbleBackground: Color {
    #if canImport(UIKit)
    return Color(.secondarySystemBackground)   // 卡面原文，iOS 分支语义零偏差
    #else
    return Color(nsColor: .windowBackgroundColor)
    #endif
}
```

iOS 分支 = 卡面原文逐字；macOS 分支仅为让宿主构建通过（语义等价：均为「次级系统背景」）。
**其余视图代码逐字照抄未改**（含中文注释、`Bubble` 结构、`.quaternary` Capsule、
`Color.accentColor.opacity(0.18)`、`maxWidth: 280`、`cornerRadius: 14` 等）。

### 2）与 android 源实现的差异：状态行 / 工具条的显示条件

`ChatPage.kt` 中**执行状态行与工具条 5 键是无条件渲染**的（状态行在 DONE 态额外带一个右箭头
`KeyboardArrowRight`；工具条 5 键 `clickable(enabled = false)` 恒在）。而本卡 §1.3 明确写
「工具条 5 键（仅 DONE 态显示）」，§2.2 代码亦为 `if phase == .done { toolbar }` +
`if phase == .sending { Text(phase.statusLabel) }`。**本卡以卡面 §2.2 为准逐字实现**，
故并**未**渲染 DONE 态的右箭头图标。此为「卡面规格 vs android 源实现」的不一致，
**照卡面执行并在此声明**，待主理人裁定是否需后续卡对齐 android。

### 3）与 android 源实现的差异：顶栏标题硬编码

§2.2 代码把标题硬编码为 `Text("WorkBuddy")`；而 `ChatPage.kt` 的标题是**入参** `title: String`
（android 示例值 `CHAT_SAMPLE_TITLE = "hello"`，副标题才是 `chat_subtitle_mode`）。
`"WorkBuddy"` 不在 `strings.xml` 的 `chat_*` 17 条内。按要求逐字照抄**未改**，
仅在此声明：标题宜为入参（后续卡可加 `title` 注入点）。

## 六、真跑验收输出（108 宿主，实测）

```console
$ cd ios && swift build
Building for debugging...
[0/1] Write swift-version-33747A42983211AE.txt
Build complete! (0.32s)
$ echo $?   # ← 用 $? 显式捕获，非管道尾
0
```

（首次全量构建 `Build complete! (16.06s)`，`ChatCopy.swift` / `ChatPageView.swift` 均参与编译，无 warning。）

```console
$ cd ios && swift test
Test Suite 'ChatPageTests' started at 2026-10-24 23:37:27.421.
Test Case '-[LengsuoMobileTests.ChatPageTests testAllSeventeenCopies]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatPageTests testBannerVerbatim]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatPageTests testModelTierMapping]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatPageTests testPlaceholderMapping]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatPageTests testSeventeenCopiesMatchAndroidStringsVerbatim]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatPageTests testStatusMapping]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatPageTests testViewInstantiatesBothPhases]' passed (0.000 seconds).
Test Suite 'ChatPageTests' passed at 2026-10-24 23:37:27.424.
Test Suite 'All tests' passed at 2026-10-24 23:37:27.336.
	 Executed 59 tests, with 0 failures (0 unexpected) in 0.035 (0.043) seconds
$ echo $?   # ← 显式捕获
0
```

- `swift build` exit **0**；`swift test` exit **0**；全包 `Executed 59 tests, with 0 failures`
  （含既有 52 + 本卡 7）。
- 本卡 **7 用例实跑全绿**（上表逐个列出，非零用例空跑；不存在「零用例 exit 0」）。
- **未做** `xcodebuild -destination generic/platform=iOS` 真机/模拟器构建；**不声称真机**。
  构建/测试宿主 = macOS（`swift build` 默认 host），以上为实测输出。

## 七、参考二进制 / 互通说明（不适用）

卡面「参考二进制」「`#[ignore]` 用例必须用 `--ignored` 实跑并贴退出码」属 Rust/FFI 线要求。
本卡为**纯 SwiftUI 视图 + 常量文案**：不涉及 Rust crate、不跨语言边界、无互通面 ⇒
**无参考二进制可用、无 `#[ignore]` 用例**（故无 `--ignored` 退出码可贴）。亦**未**用自家
roundtrip 冒充互通（本卡无任何互通宣称）。

## 八、红线自查

| 项 | 结果 |
| --- | --- |
| 单文件 ≤400 行 | ✅ 最大 111（`ChatPageView.swift`） |
| ⛔ push / merge / 改主仓 | ✅ 未执行，仅本地 commit |
| 只碰 `Chat/` + `ChatPageTests.swift` + 本报告 | ✅ `git status` 干净；`git log --stat` 仅上述文件 |
| ⛔ 共享区（Cargo/scripts/.github）| ✅ 未碰；⛔ `Package.swift` 未碰（只读） |
| ⛔ 零新依赖 | ✅ `Package.swift` `dependencies: []` 未变 |
| ⛔ 文案逐字（17 条） | ✅ §一 表 + 机器比对 `MISMATCHES: 0` + 测试锁死 |
| ⛔ 网络发送/流式/录音 | ✅ §四 声明 1–3，无相应实现 |
| 至少 2 commit | ✅ 3 commit（① Copy+状态机 ② 视图+测试 ③ 报告） |
| 真跑验收 | ✅ §六 实测 exit 0 / 0 failures |