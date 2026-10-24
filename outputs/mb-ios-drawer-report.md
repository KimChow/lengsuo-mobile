# mb-ios-drawer 报告 —— iOS 抽屉侧栏：对话列表（SwiftUI）

> 卡：`mb-ios-drawer`　lane：`ios`　worktree：`~/lengsuo-mobile-dev/worktrees/mb-ios-drawer`
> 结论：**已完成并真跑通过**（`swift build` exit 0 / `swift test` exit 0，
> `Executed 45 tests, with 0 failures`，其中本卡新增 `DrawerTests` 4 用例）
> 本卡 **2 commit**，只碰 `ios/Sources/LengsuoMobile/Drawer/`（新建目录）+ `ios/Tests/LengsuoMobileTests/DrawerTests.swift`。
> 存在 **1 项必要偏差**（§四：`Color(uiColor:)` 在 macOS 宿主不可编译），需主理人知悉。

---

## 〇、交付物

| 文件 | 行数 | 说明 |
| --- | --- | --- |
| `ios/Sources/LengsuoMobile/Drawer/DrawerCopy.swift` | 12 | 文案枚举 + 示例对话（新建） |
| `ios/Sources/LengsuoMobile/Drawer/DrawerView.swift` | 85 | 抽屉视图 + 双纯函数谓词（新建） |
| `ios/Tests/LengsuoMobileTests/DrawerTests.swift` | 26 | 4 用例（新建） |

单文件均 ≤400 行（最大 85）。零新依赖；未碰共享区 / `android/` / `ffi/` / `Package.swift` /
`Shell/` / `Me/` / `Discover/` / `Scheduled/` / `Automation/` / `Theme/`。

commit：

```
db9b3b7 feat(ios/drawer): DrawerView 抽屉侧栏 + 逐字文案
fd514ae test(ios/drawer): DrawerTests 4 用例（文案逐字 + 示例对话 + 双谓词）
```

---

## 一、4 条抽屉文案逐字对照表

来源 = 本仓 `android` `strings.xml` L45–48（本卡实读该文件取值，非转抄任务书）。

| # | strings.xml 名 | android 字面量（权威） | iOS `DrawerCopy` 字面量 | 逐字一致 |
| --- | --- | --- | --- | --- |
| 1 | `drawer_cloud_group` | `☁ 云端 ▾` | `cloudGroup = "☁ 云端 ▾"` | ✅ |
| 2 | `drawer_new_task` | `新建任务` | `newTask = "新建任务"` | ✅ |
| 3 | `drawer_empty_cloud` | `暂无云端任务` | `emptyCloud = "暂无云端任务"` | ✅ |
| 4 | `drawer_task_group` | `任务 ▾` | `taskGroup = "任务 ▾"` | ✅ |

> 上述 4 条同时被 `DrawerTests.testCopyVerbatim` 以字面量断言锁死（改一个字即测试红）。

## 二、3 条示例对话（写死惯例）声明

来源 = `android` `DrawerSidebar.kt`：

```kotlin
// 示例数据（写死；后续卡接真实会话流）
internal val DRAWER_SAMPLE_CONVERSATIONS = listOf("hello", "周报整理", "旅行计划")
```

iOS `DrawerCopy.sampleConversations = ["hello", "周报整理", "旅行计划"]`，**同款写死**，
顺序与字面量逐字一致，由 `DrawerTests.testSampleConversationsVerbatim` 断言。
**声明：本卡为静态数据，未接真实会话流**（对齐 Android 卡惯例；真实会话流属后续卡）。

## 三、交互语义对齐（`DrawerSidebar.kt` 注释逐字要点）

| Android 语义 | iOS 实现 | 位置 |
| --- | --- | --- |
| 分组下拉（☁ 行）不做真实切换（点击无操作） | `Text(DrawerCopy.cloudGroup)` + `.onTapGesture {}`（空体），仅 `contentShape` 保证可点区域 | `DrawerView.swift:36-39` |
| 「+」胶囊点击仅关抽屉（无跳转） | `Button { onNewTask(); onDismiss() }`，`onNewTask` 默认空实现 | `DrawerView.swift:40-49` |
| 选中态谓词：当前打开的对话 = 浅灰选中底 | `drawerIsSelected(index:selectedIndex:)` + `.listRowBackground(Color.gray.opacity(0.15))` | `DrawerView.swift:4-6, 62-65` |
| 列表空 ⇒ 不渲染分组与条目，直接渲染空态文案 | `drawerIsEmpty(_:)` ⇒ `if` 分支只渲染 `emptyCloud`，`else` 分支才渲染「任务 ▾」+ 列表 | `DrawerView.swift:9-11, 51-69` |
| 宽度 340dp（`DRAWER_WIDTH_DP`） | `.frame(width: 340)` | `DrawerView.swift:73` |

> 缺材料项（分组标题右侧列表管理图标）与 Android 一致：不做。

## 四、范围裁剪 + 宽度选型声明

**1）不接线宿主页（属后续卡）**：本卡只交付**独立** `DrawerView`，**未修改** `Home/TasksHomePage.swift`，
`TasksHomePage` 的抽屉钮保持现状（不渲染 `DrawerView`、不传回调）。理由 = 卡面 §1.4：接线需改
`Home/`，与本卡 `Drawer/` 目录分离以保持并行；宿主页接入属后续卡 `mb-ios-drawer-wire`。
`DrawerView` 以 `public init` 暴露 `conversations` / `selectedIndex` / `onDismiss` /
`onSelectConversation` / `onNewTask` 五个注入点，后续卡零改动即可接线。

**2）宽度选型 = 定宽 340pt 居左，未用 `.presentationDetents([.medium])`**：
选 **340pt 定宽**，因为 Android 权威实现是 `DRAWER_WIDTH_DP = 340`（`DrawerSidebar.kt` 实测
343dp 取整），定宽可与 Android 逐值对齐；`.presentationDetents([.medium])` 是相对屏高的比例
detent（约半屏），值随设备变化，无法锁 340。宿主以 `.sheet`/`ZStack` 条件渲染时，
`DrawerView` 自带 `.frame(width: 340)` 即为居左半屏观感。

## 五、⚠️ 必要偏差（1 项，需主理人知悉）

**卡面 §2.2 逐字代码 `.background(Color(uiColor: .systemBackground))` 在本仓不可编译。**

原因：`Package.swift` 的 `platforms: [.iOS(.v17), .macOS(.v13)]` 使 `swift build`（本仓验收命令）
在 **macOS 宿主**构建，`Color(uiColor:)` init 不存在，真跑报错：

```
error: extraneous argument label 'uiColor:' in call
error: type 'CGColor' has no member 'systemBackground'
```

处置：按**本仓既有惯例**改写为平台分支（与 `Theme/LengsuoTheme.swift` 的
`#if canImport(UIKit)` / `#elseif canImport(AppKit)` 双宿主写法完全一致）：

```swift
private var drawerBackground: Color {
#if canImport(UIKit)
    Color(uiColor: .systemBackground)
#elseif canImport(AppKit)
    Color(nsColor: .windowBackgroundColor)
#endif
}
```

iOS 分支即卡面原文 `Color(uiColor: .systemBackground)`，语义零偏差；macOS 分支仅为让宿主构建通过。
其余逐字代码（含中文注释、`.font(.subheadline)`、`.quaternary Capsule`、`Color.gray.opacity(0.15)`
选中底、`frame(width: 340)`）**逐字照抄未改**。

## 六、真跑验收输出

```console
$ cd ios && swift build
Building for debugging...
[0/1] Write swift-version-33747A42983211AE.txt
Build complete! (0.44s)
$ echo $?
0
```

```console
$ cd ios && swift test
Test Suite 'DrawerTests' started at 2026-10-24 23:32:02.593.
Test Case '-[LengsuoMobileTests.DrawerTests testCopyVerbatim]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.DrawerTests testEmptyPredicate]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.DrawerTests testSampleConversationsVerbatim]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.DrawerTests testSelectedPredicate]' passed (0.000 seconds).
Test Suite 'DrawerTests' passed at 2026-10-24 23:32:02.594.
	 Executed 45 tests, with 0 failures (0 unexpected) in 0.026 (0.030) seconds
$ echo $?
0
```

- `swift build` exit **0**；`swift test` exit **0**；全包 `Executed 45 tests, with 0 failures`（含既有 41 + 本卡 4）。
- 本卡 4 用例全绿，非零用例（未出现"零用例 exit 0"式空跑）。
- **未做** `xcodebuild -destination generic/platform=iOS` 真机/模拟器构建；**不声称真机**。构建宿主 = macOS
  （`swift build` 默认），上表为实测输出。

## 七、参考二进制 / 互通说明（不适用）

卡面「参考二进制」「`#[ignore]` 用例须 `--ignored` 实跑」为 Rust/FFI 线要求。本卡为**纯 SwiftUI 视图 +
常量文案**，不涉及 Rust crate、不跨语言边界、无互通面 ⇒ **无参考二进制可用、无 `#[ignore]` 用例**。
未用自家 roundtrip 冒充互通（本卡无互通宣称）。

## 八、红线自查

| 项 | 结果 |
| --- | --- |
| 单文件 ≤400 行 | ✅ 最大 84（`DrawerView.swift`） |
| ⛔ push / merge / 改主仓 | ✅ 未执行，仅本地 2 commit |
| 只碰 `Drawer/` + `DrawerTests.swift` | ✅ `git status` 干净；`git log --stat` 仅上述 3 文件 |
| ⛔ 共享区（Cargo/scripts/.github）| ✅ 未碰；⛔ `Package.swift` 未碰（只读） |
| ⛔ 零新依赖 | ✅ `Package.swift` `dependencies: []` 未变 |
| ⛔ 文案逐字 | ✅ §一 表 4 条 + §二 3 条，测试锁死 |
| 至少 2 commit | ✅ 2 commit（① DrawerView ② 测试） |