# mb-ios-automation-list 报告 —— iOS 自动化任务总览 + 详情页（SwiftUI）

- 卡号：`mb-ios-automation-list`（lane: ios，派卡根 `~/lengsuo-mobile-dev`，108）
- 日期：2026-10-24
- worktree：`~/lengsuo-mobile-dev/worktrees/mb-ios-automation-list`
- 前置：`mb-scheduled-ios` 已合入（`Scheduled/ScheduledStore.swift` 的 `ScheduledTaskStore` / `StoredTask` 复用为只读 public API）
- 上位文档：`android-devguide/11-自动化与定时任务.md`（§3 生命周期 / §8 UI 结构 / §9 边界错误）
- 编译环境：macOS（Darwin 23.6.0）+ SwiftPM，Swift 5.10 / Xcode 15.4，**真跑** `swift build` / `swift test`

---

## 一、空态 / 错误文案逐字对照表

三处文案为 devguide 11 册原文逐字（⛔ 不得改写），落在 `AutomationCopy`（`AutomationModels.swift`）：

| 用途 | 常量 | 交付值（逐字） | 出处 |
|---|---|---|---|
| 项目维度空态 | `AutomationCopy.emptyProject` | `No automation tasks for this project` | §3 + §8「列表空态：项目维度（"No automation tasks for this project"）」 |
| 详情拉取失败 | `AutomationCopy.detailLoadFailed` | `Failed to load task detail(s)` | §9「任务详情拉取失败 \| "Failed to load task detail(s)" toast + 重试」 |
| 缓存未就绪 | `AutomationCopy.cacheUnavailable` | `Local task list cache unavailable (storage/account scope not ready)` | §3 + §9「任务列表有本地缓存，按账号 scope 隔离；缓存未就绪直接提示而非显示空列表（防误读）」 |

空态渲染位置：`AutomationListView.emptyState` —— 空列表时显示，标题 `Automation`，图标 `clock.badge.questionmark`，描述文案 = `emptyProject`（逐字）。

### 1.1 缓存未就绪文案：本卡为常量落地，暂不触发（按卡面 §1.4 声明）

`cacheUnavailable` 已按卡面 §1.4 要求以**常量文案**落地（`AutomationCopy.cacheUnavailable`），但本卡**不触发**该分支：数据源为本地仓 `ScheduledTaskStore`，总会返回 `[]` 或非空数组，不存在「storage/account scope not ready」态。故该文案本卡只做常量交付 + 逐字断言（`testEmptyCopyVerbatim`），**未接入任何 UI 分支**；账号 scope 隔离属 P2 云端通道层（见 §四）。

### 1.2 详情失败文案：同上

`detailLoadFailed` 同为常量落地（逐字断言覆盖），本卡详情数据取自行模型（本地仓条目直接映射），无网络拉取路径，故**无 toast/重试接线**；真实拉取失败链路属 P2。

## 二、详情页三元素声明（§8）

§8 原文：「`automation-detail` 图标族（5 张）→ 详情页元素：状态/时间/产物。」

`AutomationDetailView` 渲染三行（`List` + `LabeledContent`），顺序即「状态/时间/产物」：

| 行 | 标签 | 值来源 | 本卡取值 |
|---|---|---|---|
| 状态 | `Status` | `AutomationTask.status` | 恒为 `scheduled`（本地仓条目均处于已排期态） |
| 时间 | `Schedule` | `AutomationTask.scheduleText` | `StoredTask.cronExpr`（如 `0 8 * * *`） |
| 产物 | `Artifact` | `AutomationTask.artifact` | **恒为 `—`**（卡面 §1.4 明示「产物本卡恒为 `—`」；本卡无产物通道，属 P2） |

`automation-detail` 图标族（5 张）资源属设计侧交付，本卡以 SF Symbol 占位（列表中 `clock.badge.questionmark` 为总览图标），未引入图片资产（⛔ 零新依赖）。

## 三、「网络版属 P2」声明

- **数据源**：本卡列表与详情数据均取自**本地仓** `ScheduledTaskStore.load() -> [StoredTask]`（UserDefaults JSON，key `lengsuo.scheduled.tasks`），**⛔ 无网络请求**（铁律 7）。
- **网络版属 P2**：`/automation-task-manager` 的云端任务聚合、`/scheduler/tasks` 通道层、`automation-detail` 远端详情拉取、账号 scope 缓存隔离，均为 P2 通道层卡交付，不在本卡范围。
- 代码注释已逐字标注：「数据源 = ScheduledStore 本地仓；网络版属 P2 通道层。」
- 无参考二进制互通需求（纯本地 SwiftUI 卡，不涉及 ffi/rust 侧），故本卡**无 `#[ignore]` 用例**，不存在需 `--ignored` 实跑的项。

## 四、交付物

| 文件 | 内容 | 行数 |
|---|---|---|
| `ios/Sources/LengsuoMobile/Automation/AutomationModels.swift` | `AutomationCopy` 三文案 + `AutomationTask` 三元素模型 + `from(StoredTask)` 映射 | 38 |
| `ios/Sources/LengsuoMobile/Automation/AutomationListView.swift` | `AutomationListView`（空态/列表）+ `AutomationDetailView`（三行） | 74 |
| `ios/Tests/LengsuoMobileTests/AutomationListTests.swift` | 4 用例（空态逐字 / 产物占位 / 映射 / 详情三元素） | 44 |

未新增任何依赖（铁律 4）；`Scheduled/` 既有文件**零改动**（只读其 public API，铁律 5 / 红线）。

## 五、偏差声明（被迫，均因卡面伪码与已合入代码不一致）

首次 `swift build` 实证两处不一致，逐条记录：

### 5.1 卡面伪码 API 不存在（`ScheduledStore.load()` / `StoredTask.artifact`）

卡面 §2.2 伪码写 `@State private var tasks: [StoredTask] = ScheduledStore.load()` 与 `value: task.artifact`，但本分支实际 API（前置卡 `mb-scheduled-ios` 已合入）为：

| 卡面伪码 | 实际 API | 处理 |
|---|---|---|
| `ScheduledStore.load()` | `ScheduledTaskStore.load() -> [StoredTask]`（`ScheduledStore` 是内存 `ObservableObject` 仓库类，无 `load()`） | 改用 `ScheduledTaskStore.load()` |
| `task.artifact`（`StoredTask.artifact`） | `StoredTask` 字段为 `id/name/cronExpr/frequency/createdAt`，**无 `artifact`** | 产物经 `AutomationTask.artifact` 提供，恒 `—` |

`Scheduled/` 属既有目录（铁律 5 / 红线禁止改动），故本卡**不改 `StoredTask`**，改由本卡新增的视图模型 `AutomationTask` 承载「产物」元素，并加 `AutomationTask.from(_:)` 映射（cron → `scheduleText`，status → `scheduled`）。卡面 `AutomationTask` 模型本身逐字保留。

### 5.2 `ContentUnavailableView` 平台可用性（Package.swift 属共享区，不可改）

`ContentUnavailableView` 仅 **macOS 14+ / iOS 17+** 可用；而 `Package.swift` 平台下限为 `.iOS(.v17), .macOS(.v13)`，且测试宿主为 macOS。首次 `swift build` 实证报错：

```
AutomationListView.swift:13:17: error: 'ContentUnavailableView' is only available in macOS 14.0 or newer
AutomationListView.swift:13:17: error: 'init(_:systemImage:description:)' is only available in macOS 14.0 or newer
error: fatalError
```

`Package.swift` 属共享区（铁律 3，本卡不得改动），故在**调用点**按可用性分支：`if #available(iOS 17.0, macOS 14.0, *)` 用 `ContentUnavailableView`，否则用等价 `VStack`（同图标 / 同标题 / 同 `emptyProject` 逐字文案）。iOS 17 真机形态与卡面伪码**完全一致**；仅旧宿主走降级分支。修复后 `swift build` 通过（§六）。

## 六、真跑验收（逐字输出）

```
$ swift build            # cwd = <worktree>/ios
Build complete! (0.19s)
build exit=0

$ swift test
Test Suite 'LengsuoMobilePackageTests.xctest' passed at 2026-10-24 23:00:35.137.
	 Executed 30 tests, with 0 failures (0 unexpected) in 0.012 (0.015) seconds
Test Suite 'All tests' passed at 2026-10-24 23:00:35.137.
	 Executed 30 tests, with 0 failures (0 unexpected) in 0.012 (0.017) seconds
test exit=0

$ swift test --filter AutomationListTests
Test Case '-[LengsuoMobileTests.AutomationListTests testDetailArtifactPlaceholder]' passed (0.001 seconds).
Test Case '-[LengsuoMobileTests.AutomationListTests testDetailViewRendersAllThreeElements]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.AutomationListTests testEmptyCopyVerbatim]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.AutomationListTests testMapFromStoredTaskKeepsIdNameAndSchedule]' passed (0.000 seconds).
Test Suite 'AutomationListTests' passed at 2026-10-24 23:00:37.877.
	 Executed 4 tests, with 0 failures (0 unexpected) in 0.002 (0.002) seconds
filter exit=0
```

总数 = 既有 26 + 本卡新增 4 = **30**，0 failures，退出码 0。既有 `ScheduledTests`(9) / `ScheduledCreateTests`(4) / `ThemeTests`(3) / `MainShellTests`(3) / `DiscoverTests`(2) / `MePageTests`(2) / `TasksHomePageTests`(2) / `LengsuoMobileTests`(1) 全部保持通过，证明未破坏前置卡行为。**非零用例、非零退出码为空跑**：本卡 4 用例均实跑通过。

## 七、红线自查

- 单文件 ≤400 行：38 / 74 / 44 ✅
- 仅改 `ios/Sources/LengsuoMobile/Automation/`（本卡新建目录）+ `ios/Tests/LengsuoMobileTests/AutomationListTests.swift` + 本报告 ✅
- 未碰共享区（`Cargo.toml` / `Cargo.lock` / `rust-toolchain.toml` / `scripts` / `.github`）✅
- 未碰 `Package.swift` / `android/` / `ffi/` / `Shell/` / `Home/` / `Me/` / `Discover/` / `Theme/` / `Scheduled/` ✅
- ⛔ 无网络请求；⛔ 零新依赖 ✅
- 文案全部来自卡面 §一 摘录，无自编中文文案 ✅
- 未 push、未 merge、未改主仓 ✅
- 不声称真机 ✅（仅 macOS 测试宿主 + SwiftPM，未做 iOS 真机/模拟器运行）

## 八、commits

| # | commit | 内容 |
|---|---|---|
| ① | `223305c` | feat: AutomationListView 总览 + 详情页 + AutomationModels（页面+模型） |
| ② | （本 commit） | test: AutomationListTests 4 用例 + 本报告 |

## 九、跨 crate / 共享区所需改动（仅记录，未自行改）

如需彻底消除 §5.2 的可用性分支，需将 `Package.swift` 平台下限提到 `.macOS(.v14)`（或 iOS-only 宿主）。`Package.swift` 属共享区，由基建卡独占负责，本卡**未改**，仅在此登记。