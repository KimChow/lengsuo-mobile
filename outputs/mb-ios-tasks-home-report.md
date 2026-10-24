# mb-ios-tasks-home 报告 —— iOS「任务」首页：欢迎区静态骨架（SwiftUI）

- 任务：`mb-ios-tasks-home`　lane：`ios`　派卡根：`~/lengsuo-mobile-dev`（108）
- 改动文件（均在卡边界内）：
  - `ios/Sources/LengsuoMobile/Home/TasksHomePage.swift`（整体替换主理人占位骨架）
  - `ios/Tests/LengsuoMobileTests/TasksHomePageTests.swift`（新建，2 用例）
  - `outputs/mb-ios-tasks-home-report.md`（本报告）
- 零新依赖；未触碰共享区 / `android/` / `ffi/` / `Package.swift` / `Shell/` / `Theme/` / `Me/`。

## 一、逐字文案清单（与 android-devguide/15 §2 首行 + §6 对齐）

| 位置 | 文案（逐字） | 实现载体 |
| --- | --- | --- |
| 顶栏抽屉钮 | `line.3.horizontal` SF Symbol | `TasksHomePage.topBar` |
| 顶栏标题 | `WorkBuddy` | `Text("WorkBuddy").font(.headline)` |
| 顶栏工作模式 | `☁ 工作模式 >`（文本 `☁ 工作模式` + `chevron.right`） | `HStack`，`.secondary` 前景 |
| 欢迎语 | `WorkBuddy, 我帮你` | `Text(...).font(.title2.bold())` |
| 快捷胶囊 1 | `进行数据分析及可视化` | `TasksHomePage.initialChips[0]` |
| 快捷胶囊 2 | `进行文档处理` | `TasksHomePage.initialChips[1]` |
| 快捷胶囊 3 | `做一份幻灯片` | `TasksHomePage.initialChips[2]` |
| 快捷胶囊 4 | `制作工作台` | `TasksHomePage.initialChips[3]` |
| 换一换 | `换一换` | `Button("换一换")`，点击 `chips.shuffle()`（纯本地随机重排，无网络/无建议集刷新） |
| 输入栏占位 | `发消息或按住说话` | `Text(...)`，`.secondary` 前景 |
| 输入栏 4 元素 | `plus` / 文本占位 / `mic` / `play.fill`（SF Symbol） | `TasksHomePage.inputBar` |

文案由 `TasksHomePageTests.testInitialChipsVerbatim` / `testChipsCountIsFour` 逐字断言锁定。

## 二、占位差异声明（本卡裁剪，后续卡补齐）

1. **3D 吉祥物（Rive）未实现**：以 `face.smiling` SF Symbol（56pt）占位。卡面明令禁用 👀 emoji，已遵守。
2. **运营 banner 未实现**：卡面明令 ⛔ 不放假 banner，本卡完全未渲染 banner 元素（顶栏下方直接进入欢迎区）。
3. **发送 / 语音逻辑未实现**：输入栏为静态骨架，4 元素均无交互动作（`plus`/`mic`/`play.fill` 为纯 `Image`，文本为纯占位）。
4. **抽屉、工作模式切换逻辑未实现**：抽屉钮为空动作 `Button {} label: {...}`；工作模式入口仅静态展示，点击无响应。
5. **快捷胶囊「点击即预填发送」未实现**：本卡胶囊为纯 `Text` 展示，无点击预填（发送逻辑属后续卡）。
6. **「换一换」实现差异**：规格为「刷新建议集」，本卡为同一 4 条建议的本地随机重排（无外部建议集），符合卡面「点击随机重排 4 条，纯本地」的裁剪定义。

## 三、真跑验收输出（108，Xcode 15.4 / SwiftPM）

### swift build（exit 0）

```
Building for debugging...
[3/11] Emitting module LengsuoMobile
[4/13] Compiling LengsuoMobile MainShell.swift
[5/13] Compiling LengsuoMobile MainTab.swift
[6/13] Compiling LengsuoMobile ScheduledPageView.swift
[7/13] Compiling LengsuoMobile LengsuoStrings.swift
[8/13] Compiling LengsuoMobile MePage.swift
[9/13] Compiling LengsuoMobile ScheduledFormView.swift
[10/13] Compiling LengsuoMobile ScheduledModels.swift
[11/13] Compiling LengsuoMobile DiscoverPage.swift
[12/13] Compiling LengsuoMobile TasksHomePage.swift
[13/13] Compiling LengsuoMobile LengsuoMobile.swift
Build complete! (15.19s)
BUILD_EXIT=0
```

### swift test（exit 0）

```
Test Suite 'TasksHomePageTests' started at 2026-10-24 22:08:16.442.
Test Case '-[LengsuoMobileTests.TasksHomePageTests testChipsCountIsFour]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.TasksHomePageTests testInitialChipsVerbatim]' passed (0.000 seconds).
Test Suite 'TasksHomePageTests' passed at 2026-10-24 22:08:16.442.
        Executed 2 tests, with 0 failures (0 unexpected) in 0.000 (0.001) seconds
Test Suite 'All tests' passed at 2026-10-24 22:08:16.442.
        Executed 15 tests, with 0 failures (0 unexpected) in 0.005 (0.009) seconds
TEST_EXIT=0
```

**用例数说明**：卡面预期「Executed 11 tests, 0 failures」，实际为 **15 tests, 0 failures**。差异原因：卡面撰写时基线为 9 例；主理人后续合入的 `ScheduledTests` 已增至 9 例（含 `28bdfaf` 新增的 `testStoreAddInsertsAtFrontAndRemove`），现基线 13 例 + 本卡新增 2 例 = 15 例。本卡 2 个新用例全部通过，基线用例零回归。

## 四、commit 划分（≥2）

1. `feat(ios/home)`: `TasksHomePage.swift` 整体替换占位骨架为静态实现
2. `test(ios/home)`: `TasksHomePageTests.swift` 新建逐字断言
3. `docs(ios/home)`: 本报告

## 五、红线自查

- 单文件 ≤400 行：`TasksHomePage.swift` 90 行、测试 18 行 ✅
- 零新依赖（无 `Package.swift` 改动）✅
- 未 push / 未 merge / 未改主仓 ✅
- 无假数据 / 假运营内容（无 banner、无假建议集）✅
- 未声称真机验证（仅 SwiftPM 模拟器态构建与单测）✅
- 无需跨 crate 改动，无需动共享区 ✅
