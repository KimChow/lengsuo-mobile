# mb-ios-scheduled-create 报告 —— iOS 定时任务创建：表单校验 + 本地持久化

- 卡号：`mb-ios-scheduled-create`（lane: ios，派卡根 108）
- 日期：2026-10-24
- worktree：`~/lengsuo-mobile-dev/worktrees/mb-ios-scheduled-create`
- 前置：`mb-scheduled-ios` 已合入（`Scheduled/` 目录已有 ScheduledPageView / ScheduledModels / ScheduledFormView）
- 编译环境：macOS（Darwin 23.6.0）+ SwiftPM，**真跑** `swift build` / `swift test`

---

## 一、R11-1 对齐声明

devguide 11 册 §2 规则 R11-1（逐字）：

> 「调度统一收敛到 cron 表达式表达，`cronFrequencyType` 作为 UI 快捷档（免让用户写 cron）；`triggerType` 区分定时触发与其他触发源。」

本卡实现与 R11-1 的对应关系：

1. **UI 快捷档**：`ScheduleQuickFrequency` 四选（一次性 / 每天 / 每周 / 每月），用户不写 cron —— 即「cronFrequencyType 作为 UI 快捷档」。
2. **cron 自动生成**：`ScheduleDraft.cronExpression()` 由快捷档 + 时/分自动产出 cronExpr，无手写 cron 输入 —— 即「调度统一收敛到 cron 表达式表达」。
3. **triggerType**：本卡全部为定时触发（`once/daily/weekly/monthly` 均生成 cron），不涉及事件触发（devguide §2 原文即注明「事件【枚举值未恢复】」），不在本卡范围。
4. §1.3 创建入口「/todo/create → Agent 解析 → /scheduler/tasks」属云端链路（见 §五 未做声明）；本卡交付其 iOS 本地形态：表单 → 校验 → 生成 cron → 本地持久化。

## 二、cron 生成规则表

`ScheduleDraft.cronExpression()`（分 时 [日] [月] [周] 五段）：

| 快捷档 | 输入 | 生成 cronExpr | 说明 |
|---|---|---|---|
| `once`（一次性） | 08:00 | `0 8 * * *` | 与 daily 同形；一次性语义由上层生命周期区分（本卡不做调度执行） |
| `daily`（每天） | 08:00 | `0 8 * * *` | 卡面示例「每天 = `0 8 * * *`」 |
| `weekly`（每周） | 周三 09:30 | `30 9 * * 3` | weekday 1=周一…7=周日，落在第五段 |
| `monthly`（每月） | 08:00 | `0 8 1 * *` | 固定每月 1 日（卡面 §1.4 未给月规则，取每月 1 号） |

校验规则（`ScheduleDraft.validate()`，纯函数）：

| 校验项 | 规则 | 错误文案（逐字） |
|---|---|---|
| 名称 | 去空白后非空 | 「请填写任务名称」 |
| 小时 | 0...23 | 「小时超出范围」 |
| 分钟 | 0...59 | 「分钟超出范围」 |
| 星期（仅 weekly） | 1...7 | 「请选择星期」 |

## 三、交付物

| 文件 | 内容 |
|---|---|
| `ios/Sources/LengsuoMobile/Scheduled/ScheduleDraft.swift` | 快捷档枚举 + 草稿模型 + 校验 + cron 生成（51 行） |
| `ios/Sources/LengsuoMobile/Scheduled/ScheduledStore.swift` | `StoredTask`（Codable）+ UserDefaults JSON 持久化，key=`lengsuo.scheduled.tasks`，增/删/查 |
| `ios/Sources/LengsuoMobile/Scheduled/CreateTaskSheet.swift` | 创建表单弹层：名称 + 快捷档 + 星期（weekly 时）+ 时间选择，提交前校验 |
| `ios/Tests/LengsuoMobileTests/ScheduledCreateTests.swift` | 4 用例（见 §四） |

未新增任何依赖（铁律 4）；未改动 ScheduledPageView 及其他既有文件的任何行为（铁律 5，仅新增文件）。

## 四、真跑验收（逐字输出）

```
$ swift build --package-path ios
Build complete! (3.60s)

$ swift test --package-path ios
Test Suite 'All tests' passed at 2026-10-24 22:31:46.650.
	 Executed 24 tests, with 0 failures (0 unexpected) in 0.012 (0.017) seconds
（退出码 0）

$ swift test --package-path ios --filter ScheduledCreateTests
Test Case '-[LengsuoMobileTests.ScheduledCreateTests testDailyCron]' passed (0.001 seconds).
Test Case '-[LengsuoMobileTests.ScheduledCreateTests testEmptyNameInvalid]' passed (0.000 seconds).
Test Case '-[LengsuoMobileTests.ScheduledCreateTests testStoreRoundTrip]' passed (0.004 seconds).
Test Case '-[LengsuoMobileTests.ScheduledCreateTests testWeeklyCronUsesWeekday]' passed (0.000 seconds).
Test Suite 'ScheduledCreateTests' passed at 2026-10-24 22:31:57.079.
	 Executed 4 tests, with 0 failures (0 unexpected) in 0.005 (0.006) seconds
（退出码 0）
```

总数 = 既有 20 + 本卡新增 4 = 24，0 failures。既有 `ScheduledTests`（9 用例）等全部保持通过，证明未破坏前置卡行为。

## 五、偏差与未做声明

### 5.1 命名偏差（被迫，2 处，逻辑逐字保留）

卡内代码与前置卡 `mb-scheduled-ios` 已合入的 `ScheduledModels.swift` 存在两类**同名冲突**，首次 `swift build` 实证报错（`invalid redeclaration of 'ScheduleFrequency'` / `invalid redeclaration of 'ScheduledStore'`）。既有类型被 `ScheduledPageView`/`ScheduledFormView` 使用，本卡铁律 5 禁止改其公开行为，故只对本卡新类型改名：

| 卡内原型 | 实际交付 | 原因 |
|---|---|---|
| `ScheduleFrequency`（once/daily/weekly/monthly 快捷档） | `ScheduleQuickFrequency` | 既有 `ScheduleFrequency` 是 7 档间隔枚举（daily/every30Minutes/…/biweekly，对齐 Android 卡），语义不同且同名；不可复用（无 once/weekly/monthly 档）也不可改名（会破坏既有公开行为） |
| `ScheduledStore`（UserDefaults 仓库） | `ScheduledTaskStore` | 既有 `ScheduledStore` 是内存 ObservableObject 仓库类，ScheduledPageView 正在 `@StateObject` 使用 |

key 仍为卡面指定的 `lengsuo.scheduled.tasks`；`StoredTask` / `ScheduleDraft` / `CreateTaskSheet` 与卡面逐字一致。

### 5.2 未做（按卡面 §二 ⛔ 清单，属后续卡）

- **云端 scheduler 未做**：`/scheduler/tasks` 通道层、`/todo/create`、Agent 时间意图解析均不在本卡（P2）；本卡仅本地 UserDefaults 持久化。
- **推送/通知未做**：devguide 11 册 §4–6（通知推送、日历集成、企微派发）属后续卡。
- **不接网络**（铁律 7）：生命周期中的「云端执行」部分本卡不做；`once` 档仅生成 cron 字段，不做本地触发执行。
- **无 `#[ignore]` 用例**：本卡全部 4 用例均已实跑（Swift 测试无 ignore 标记），不存在「--ignored 实跑」项；无参考二进制互通需求（纯本地 SwiftUI 卡）。

## 六、红线自查

- 单文件 ≤400 行：51 / 40 / 44 / 30 行 ✅
- 仅改 `ios/Sources/LengsuoMobile/Scheduled/` + 本卡测试 + 本报告 ✅
- 未碰共享区（Cargo.toml / Cargo.lock / rust-toolchain.toml / scripts / .github）、未碰 android/ ffi/ Package.swift Shell/ Home/ Me/ Discover/ Theme/ ✅
- 未 push、未 merge 主仓 ✅
- 文案全部来自卡面 §一/§三 摘录，无自编中文文案 ✅

## 七、commits

| # | commit | 内容 |
|---|---|---|
| ① | `3e6af06` | feat: ScheduleDraft 表单校验 + cron 生成 |
| ② | `32b180a` | feat: ScheduledTaskStore 持久化 + CreateTaskSheet UI |
| ③ | （本 commit） | test: ScheduledCreateTests 4 用例 + 本报告 |
