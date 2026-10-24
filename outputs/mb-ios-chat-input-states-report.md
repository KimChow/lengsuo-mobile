# mb-ios-chat-input-states 报告 —— iOS 输入区两态交互 + 附件 chips（SwiftUI）

> 卡：`mb-ios-chat-input-states`　lane：`ios`　worktree：`~/lengsuo-mobile-dev/worktrees/mb-ios-chat-input-states`
> 结论：**已完成并真跑通过**（`cd ios && swift build` exit **0**（0 warning）/
> `swift test` exit **0**，全包 `Executed 81 tests, with 0 failures`，
> 其中本卡新增 `ChatInputTests` **18 用例全绿**——逐用例见 §六）。
> 本卡 **2 个实现 commit + 本报告 commit**，只碰 `ios/Sources/LengsuoMobile/Chat/`（新建 2 文件 +
> 改 `ChatPageView.swift` 的**输入栏区**，⛔ 未动消息区/工具条/顶栏）+
> `ios/Tests/LengsuoMobileTests/ChatInputTests.swift` + 本报告。
> 存在 **1 项必要偏差**（§五-1：`.onChange` 两参式在 macOS 13 目标下不可编译）与
> **5 项已声明扩展/差异**（§五-2～6），需主理人知悉。

---

## 〇、交付物与 commit

| 文件 | 行数 | 说明 |
| --- | --- | --- |
| `ios/Sources/LengsuoMobile/Chat/ChatInputModel.swift` | 69 | 新建：`ChatInputCopy` 4 条 04§5 原文串 + `InputPhase` 四态状态机（含 `afterTextChange`/`afterAttachmentAdded` 迁移纯函数）+ `InputValidator` 校验纯函数 |
| `ios/Sources/LengsuoMobile/Chat/ChatInputBar.swift` | 79 | 新建：输入栏增强版——附件 chips 横向 ScrollView + TextField 两态 + 🎤 高亮 + ⏯ 发送态 + 模拟上传注入点 |
| `ios/Sources/LengsuoMobile/Chat/ChatPageView.swift` | 112 | 仅改**输入栏区**：静态 `Text` 输入条 → `ChatInputBar`（+3 个 `@State`）；消息区/工具条/顶栏 0 改动 |
| `ios/Tests/LengsuoMobileTests/ChatInputTests.swift` | 157 | 新建：**18 用例**（卡面 §2.3 的 3 条 + 15 条补充） |

单文件均 ≤400 行（最大 157）。零新依赖（`Package.swift` 未改，`dependencies: []` 原样）；
未碰共享区（`Cargo.toml` / `Cargo.lock` / `rust-toolchain.toml` / `scripts` / `.github`）、
`android/` / `ffi/` / `Shell/` / `Home/` / `Me/` / `Discover/` / `Scheduled/` / `Automation/` / `Drawer/` / `Theme/`。

```
7e4b718 ① feat(ios/chat): ChatInputModel 四态状态机 + InputValidator 纯函数 + 04§5 原文串
de0f44c ② feat(ios/chat): ChatInputBar 附件 chips + TextField 两态 + 模拟上传注入，接入 ChatPageView 输入栏区
（③）docs(ios/chat): mb-ios-chat-input-states 卡报告 —— 即本报告所在 commit
```

---

## 一、04§5 原文串对照表（英文原文保留，⛔ 翻译/改写）

权威来源 = 派卡根 `tasks/mb-ios-chat-input-states.txt` §一（devguide 04§5 摘录；该文档不在仓内，
本仓 `android/strings.xml` 亦无此 4 条 ⇒ **卡面即唯一权威**，与 `mb-ios-chat-page` 卡先例一致）。

| # | 卡面 §一 条目 | 原文串（权威，逐字） | iOS `ChatInputCopy` | 逐字一致 |
| --- | --- | --- | --- | --- |
| 1 | §1.1 附件上限 5 个 | `Attachment limit reached. You can add up to 5.` | `attachmentLimit` | ✅ |
| 2 | §1.1 上传/发送并发 | `Attachments are still uploading. Please wait.` | `stillUploading` | ✅ |
| 3 | §1.1 转发/上传计数上限 50 | `Attachment count exceeds the limit of 50.` | `forwardLimit` | ✅ |
| 4 | §1.2 语音录制达上限 | `Maximum voice recording length reached. Recognized text will appear in the input box.` | `voiceMaxLength` | ✅ |
| 5 | §1.3 布局内占位符 | `发消息或按住说话...`（半角 `0x2E`×3 结尾） | 复用 `ChatCopy.inputPlaceholderIdle`（`ChatInputBar` 默认占位符） | ✅ |

**机器校验（非目测）**：用脚本把卡面 §一 括号内英文串与 `ChatInputModel.swift` 的 4 个属性做
**逐字符等值比对**，输出 `MISMATCHES: 0` / `EXTRA_IN_SWIFT: []`；§1.3 占位符与
`ChatCopy.inputPlaceholderIdle` 亦字节全等（结尾三点为半角 U+002E×3，与「正在思考 ⋯」U+22EF 不混用）。
另有 `testAllFourCopiesVerbatim` / `testEnglishOriginalsNotTranslated`（4 条全 ASCII、未翻译）
以字面量断言锁死——改一个字即测试红。

**交接说明**：派卡消息里 §2.1 的 `voiceMaxLength =` 行被截断（无右值）；本报告以派卡根
`tasks/mb-ios-chat-input-states.txt` L45 的完整原文串为准（已实读 + 字节比对，非转抄记忆）。

## 二、输入状态机 `InputPhase { idle, ready, uploading, recording }`

```text
                 文本非空 (afterTextChange)
    ┌────────┐ ────────────────────────────▶ ┌────────┐
    │  idle  │ ◀──────────────────────────── │ ready  │
    │ 占位符 │        文本清空/纯空白          │ 有文本 │  ⏯ = arrow.up.circle.fill（发送态）
    └────────┘                               └────────┘
         │  ▲                                     │
         │  │ +附件且模拟上传未完成(uploadStalled)  │
         │  │                                     ▼
         │  │            ┌────────────┐  上传完成(注入器返回非 nil)
         │  └──────────  │ uploading  │ ─────────────▶ ready/idle（⛔ 非死态）
         │     仍在上传   └────────────┘
         │     (stillUploading 文案；canSend=false，⏯ 发送被挡)
         │
         │  宿主注入 phase=.recording（🎤 mic.fill 高亮；真录音属分册 10）
         ▼
    ┌────────────┐   文本事件/附件事件均不覆盖本态（afterTextChange/afterAttachmentAdded 守卫）
    │ recording  │
    └────────────┘

    ⏯ 发送（card §2.2 原逻辑）: canSend(text 非空 && !uploading) ⇒ onSend() + text="" + phase=.idle
```

迁移纯函数与用例（视图 `onChange`/`+` 钮只调纯函数，状态机逻辑可单测）：

| 事件 | 迁移 | 纯函数 | 锁死用例 |
| --- | --- | --- | --- |
| 文本非空（idle/ready） | → `.ready` | `InputPhase.afterTextChange` | `testAfterTextChangeIdleReady` ✅ |
| 文本清空/纯空白 | → `.idle` | 同上 | 同上 ✅ |
| 文本事件（recording/uploading 中） | **保持原态**（不被文本覆盖） | 同上 | `testAfterTextChangeDoesNotClobberRecordingOrUploading` ✅ |
| `+` 附件、模拟上传未完成 | → `.uploading`（等待文案 + 发送被挡） | `InputPhase.afterAttachmentAdded` | `testAfterAttachmentAdded` / `testUploadingBlocksSend` ✅ |
| `+` 附件、上传完成（原 uploading） | → `.ready`/`.idle`（**uploading 可退出，非死态**） | 同上 | `testUploadingIsNotATerminalState` ✅ |
| `+` 附件（recording 中） | 保持 `.recording`（不被附件事件打断） | 同上 | `testAfterAttachmentAdded` ✅ |
| 附件数 ≥ 5 | 显示 `attachmentLimit` 文案（**不再加 chip**，见 §五-2） | `InputValidator.attachmentLimitError` | `testAttachmentLimitVerbatim` / `testAttachmentLimitBoundary` ✅ |
| ⏯ 点击 | `canSend` 通过 ⇒ `onSend()`+清文本+`.idle` | `InputValidator.canSend` | `testCanSend` ✅ |

§1.4 ↔ §2.1 API 命名映射（§1.4 三个名字 → §2.1 具体签名）：`canAddAttachment` →
`InputValidator.canAddAttachment(current:)`（已实现）；`sendableText` → `canSend(text:attachmentsUploading:)`；
`上限文案` → `attachmentLimitError(current:)` / `forwardLimitError(count:)`。

## 三、布局对齐（§1.3「`[+] 附件 | 文本框"发消息或按住说话..." | 🎤语音 | ⏯`」）

| 区块 | 卡面要求 | iOS 实现 |
| --- | --- | --- |
| 附件 chips | 输入栏上方横向 ScrollView（名字 + × 移除） | `ScrollView(.horizontal)` + `ForEach` capsule chip：📎 图标 + 名字 + `xmark` 移除钮（card §2.2 逐字） |
| `[+]` 附件 | 加附件；超 5 个触发上限文案 | `plus` 按钮：`attachmentLimitError` 命中 ⇒ 显示逐字上限文案（§五-2 声明的形态）；否则经 `uploadSimulator` 注入取名追加 chip |
| 文本框 | 占位符「发消息或按住说话...」两态 | `TextField`（默认占位符 = §1.3 原文串；宿主 `ChatPageView` 注入 `ChatPhase.inputPlaceholder` 保留「发消息...」发送中态，见 §五-4） |
| 🎤 语音 | 录音态高亮 | `phase == .recording ? "mic.fill" : "mic"`（card §2.2 逐字；真录音属分册 10） |
| ⏯ | idle=播放态 / 有文本=发送态 | `phase == .idle ? "play.fill" : "arrow.up.circle.fill"`（card §2.2 逐字） |
| 上传等待行 | uploading 显示等待文案 | `hint ?? (phase == .uploading ? stillUploading : nil)` 同一 caption 行（§五-2） |

## 四、声明（本卡边界）

1. **不真上传**：`ChatInputBar.uploadSimulator: (Int) -> String?` 为**模拟闭包注入点**（铁律 7 要求的形态）——
   返回最终附件名 = 上传完成；返回 `nil` = 上传未完成 ⇒ 停 `.uploading`、显示等待文案、发送被
   `canSend` 挡住。默认注入器返回「附件N.pdf」（= 卡面 §2.2 的演示名）。**无任何网络/URLSession/进度回调**。
2. **不真录音**：🎤 为静态图标（card §2.2 逐字，无 `AVAudioRecorder`/麦克风/权限）；`.recording`
   态由宿主经 `phase` binding 注入驱动（真录音状态机属分册 10）。
3. **不真发送**：`onSend` 默认空闭包（宿主未接发送通道）；⏯ 逻辑仅做 `canSend` 校验 + 清文本 + 回 `.idle`。
4. **不接宿主发送/抽屉**：`ChatPageView` 仅替换输入栏区并持有输入态；无 sheet/导航/网络接线。
5. **不声称真机**：以下 build/test 均为 108 宿主 `swift build`（macOS host，Swift 5.10）实测，
   未做 `xcodebuild -destination generic/platform=iOS`，未在真机/模拟器验证。

## 五、偏差声明（1 项必要 + 5 项已声明）

### 1）⚠️ 必要偏差：`.onChange(of:){ _, v in }` 两参式在 macOS 13 目标不可编译

卡面 §2.2 逐字给的是 `.onChange(of: text) { _, v in ... }`（`onChange(of:initial:_:)`，macOS 14+）。
本包 `Package.swift`（⛔ 共享区未改）`platforms: [.iOS(.v17), .macOS(.v13)]`，验收命令
`swift build` 在 macOS 宿主以 13 为 deployment target 构建 ⇒ **照抄必编译错**（实测 error：
`'onChange(of:initial:_:)' is only available in macOS 14.0 or newer`）。
处置：改为 macOS 13 可用的单参式 `.onChange(of: text) { v in ... }`（语义等价，闭包入参同为新值），
构建 0 warning 0 error。

### 2）`+` 钮上限行为的形态替换（文案逐字不变）

卡面 §2.2 原行是把上限**文案本身 append 进 attachments**（自注「仅供演示的溢出提示占位」）——照抄会
出现一个名字是整句英文的 chip，且第 6 个附件越过 5 上限。§1.4 的要求是「超 5 个**触发上限文案**」。
处置：命中 `attachmentLimitError` ⇒ 不改 attachments，改为在输入栏内的提示行**显示逐字文案**
（与 `.uploading` 等待文案同槽），输入/附件事件即清除。文案一字未改；如需还原卡面原行为，
把 `hint = e` 换回 `attachments.append(e)` 即可（一行）。

### 3）文本事件守卫从「仅 recording」扩为「recording + uploading」

卡面 §2.2 `onChange` 只守 `phase != .recording`——typing 会把 `.uploading` 打回 `.ready`，使
04§5「发送前等待上传完成」的等待文案被清掉且发送被放开。处置：`afterTextChange` 同时守
`.uploading`（配套 `afterAttachmentAdded` 提供 uploading 的**完成退出**路径，非死态）。
此为对卡面语义缺陷的修正，`testUploadingIsNotATerminalState` 锁死退出路径。

### 4）`ChatInputBar` 签名扩展：`placeholder` 注入参

卡面 §2.2 占位符为硬编码字面量。为不回退前置卡已交付的 `ChatPhase.inputPlaceholder` 两态占位符
（发送中 = 「发消息...」，`ChatPageTests` 已锁死），`ChatInputBar` 增加 `placeholder` 参数，
**默认值与卡面字面量同字节**（复用 `ChatCopy.inputPlaceholderIdle`，`testInputBarDefaultPlaceholderVerbatim`
双断言锁死）。

### 5）`InputValidator` 新增 API（超出 §2.1 清单）

- `canAddAttachment(current:)`：§1.4 点名的谓词（§2.1 未给签名），1 行实现 + 用例；
- `maxForwardCount = 50` + `forwardLimitError(count:)`：否则 §1.1 第 3 条原文串（50 上限）无任何
  承载/校验面。边界取 `count > 50`（文案自述 "exceeds the limit of 50"），`testForwardLimitBoundary` 锁死。

### 6）`ChatPageView` 输入栏区接线后 `ChatCopy.inputPlus/inputVoice` 不再被视图渲染

输入条改用 SF Symbols（`plus` / `mic`，卡面 §2.2 形态）。两个常量**保留**在 `ChatCopy`（strings.xml
`chat_input_plus`/`chat_input_voice` 的逐字载体），`ChatPageTests` 17 条断言原样全绿，未删未改。

## 六、真跑验收输出（108 宿主，实测）

```console
$ cd ios && swift build          # 触碰 4 文件后全量编译
Building for debugging...
[0/2] Write sources
[1/2] Write swift-version-33747A42983211AE.txt
[3/6] Compiling LengsuoMobile ChatInputModel.swift
[4/6] Compiling LengsuoMobile ChatInputBar.swift
[5/6] Compiling LengsuoMobile ChatPageView.swift
[6/6] Emitting module LengsuoMobile
Build complete! (1.27s)
$ echo $?
0
```

（`grep -c "warning:"` = **0**；宿主 swift-driver 1.90.11.1 / Apple Swift 5.10，x86_64-apple-macosx14。）

```console
$ cd ios && swift test
Test Suite 'ChatInputTests' started at 2026-10-25 01:50:05.104.
Test Case '-[LengsuoMobileTests.ChatInputTests testAfterAttachmentAdded]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatInputTests testAfterTextChangeDoesNotClobberRecordingOrUploading]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatInputTests testAfterTextChangeIdleReady]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatInputTests testAllFourCopiesVerbatim]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatInputTests testAttachmentLimitBoundary]' passed (0.001 seconds).
Test Case '-[LengsuoMobileTests.ChatInputTests testAttachmentLimitVerbatim]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatInputTests testCanAddAttachment]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatInputTests testCanSend]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatInputTests testCopyVerbatim]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatInputTests testEnglishOriginalsNotTranslated]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatInputTests testForwardLimitBoundary]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatInputTests testInputBarDefaultPlaceholderVerbatim]' passed (0.002 seconds).
Test Case '-[LengsuoMobileTests.ChatInputTests testInputBarDefaultUploadSimulator]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatInputTests testInputBarInjectedSimulatorCanStallUploading]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatInputTests testInputBarInstantiatesAllPhases]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatInputTests testPhaseIsFourStateAndEquatable]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatInputTests testUploadingBlocksSend]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ChatInputTests testUploadingIsNotATerminalState]' passed (0.000 seconds).
Test Suite 'ChatInputTests' passed at 2026-10-25 01:50:05.109.
	 Executed 18 tests, with 0 failures (0 unexpected) in 0.004 (0.006) seconds
Test Suite 'All tests' passed at 2026-10-25 01:50:05.146.
	 Executed 81 tests, with 0 failures (0 unexpected) in 0.038 (0.046) seconds
$ echo $?
0
```

- `swift build` exit **0**；`swift test` exit **0**；全包 81 tests 0 failures（含既有 63 + 本卡 18）。
- 本卡 **18 用例逐个列出且全 passed**（上表非零用例空跑；不存在「零用例 exit 0」）。
- 卡面 §2.3 的 3 条用例（`testAttachmentLimitVerbatim` / `testCopyVerbatim` / `testCanSend`）
  **逐字保留**在文件内，另加 15 条补充用例。

## 七、参考二进制 / 互通说明（不适用）

卡面「参考二进制」「`#[ignore]` 用例必须 `--ignored` 实跑并贴退出码」属 Rust/FFI 线要求。本卡为
**纯 SwiftUI 视图 + 常量文案 + 纯函数**：不涉及 Rust crate、不跨语言边界、无互通面 ⇒
**无参考二进制可用、无 `#[ignore]` 用例**（故无 `--ignored` 退出码可贴）。亦**未**用自家
roundtrip 冒充互通（本卡无任何互通宣称）。

## 八、红线自查

| 项 | 结果 |
| --- | --- |
| 单文件 ≤400 行 | ✅ 最大 157（`ChatInputTests.swift`）；源文件最大 112 |
| ⛔ push / merge / 改主仓 | ✅ 未执行，仅本地 commit |
| 只碰 `Chat/` + `ChatInputTests.swift` + 本报告 | ✅ `git diff --name-only 77ddc9b...HEAD` = 恰 4 文件全在 touches 内；工作区 clean |
| ⛔ 动 ChatPageView 消息区/工具条/顶栏 | ✅ diff 仅 +3 `@State`（输入栏区状态）+ 替换 `inputBar`；消息区/工具条/顶栏 0 改动 |
| ⛔ 共享区（Cargo/scripts/.github）/ `Package.swift` | ✅ 未碰（只读） |
| ⛔ 零新依赖 | ✅ `Package.swift` `dependencies: []` 未变 |
| ⛔ 真上传/真录音/真发送 | ✅ 均为模拟注入/静态图标/空闭包（§四声明） |
| ⛔ 文案逐字 | ✅ 4 条英文原文 + 占位符脚本级字节比对 `MISMATCHES: 0`，18 用例含 6 条文案锁死断言 |
