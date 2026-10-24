# mb-ios-me-settings-pages 报告 —— iOS「我的」页子页（字号/朗读/触感/存储/通知）

> 卡：`mb-ios-me-settings-pages`　lane：`ios`
> worktree：`~/lengsuo-mobile-dev/worktrees/mb-ios-me-settings-pages`
> 结论：**已完成并真跑通过**（`swift build` exit 0 / `swift test` exit 0，
> `Executed 37 tests, with 0 failures`；额外 `xcodebuild -destination generic/platform=iOS` **BUILD SUCCEEDED**）
> 本卡新增 **3 commit**，只碰 `ios/Sources/LengsuoMobile/Me/` + 本卡测试文件。

---

## 〇、交付物

| 文件 | 行数 | 说明 |
| --- | --- | --- |
| `ios/Sources/LengsuoMobile/Me/MeSettingsStore.swift` | 34 | 偏好模型 + UserDefaults 持久化（新建） |
| `ios/Sources/LengsuoMobile/Me/MeSettingsPages.swift` | 79 | 四个子页视图 + 字号档 → DynamicTypeSize 映射（新建） |
| `ios/Sources/LengsuoMobile/Me/MePage.swift` | 117 | 五行改 NavigationLink（改写） |
| `ios/Tests/LengsuoMobileTests/MeSettingsTests.swift` | 144 | 11 用例（新建） |

单文件均 ≤400 行（最大 144）。

---

## 一、入口名逐字对照（15§2 我的行）

任务书 §1.1 摘录原文：

> 「主题（系统/浅色/深色 三段选择）、语言（跟随系统）、字体大小、朗读设置、触感反馈（开）、
> 存储空间、消息通知设置」

| # | §1.1 入口名（逐字） | MePage 源码字面量 | 导航目标 | 导航？ |
| --- | --- | --- | --- | --- |
| 1 | 主题 | `Picker("主题", …)`（三段：系统/浅色/深色） | —（页内三段控件） | 否（本卡不动，见 §五偏差1） |
| 2 | 语言（跟随系统） | `row("语言", value: "跟随系统")` | — | **否**（⛔ 语言行保持静态） |
| 3 | 字体大小 | `navRow("字体大小")` | `FontSizePage` | ✅ |
| 4 | 朗读设置 | `navRow("朗读设置")` | `ReadingHapticsPage` | ✅ |
| 5 | 触感反馈（开） | `navRow("触感反馈", value: … ? "开" : "关")` | `ReadingHapticsPage` | ✅ |
| 6 | 存储空间 | `navRow("存储空间")` | `StoragePage` | ✅ |
| 7 | 消息通知设置 | `navRow("消息通知设置")` | `NotificationSettingsPage` | ✅ |

- 入口名字面量由 §八 的 `testEntryNamesAndSubPageTitlesVerbatim` **源码级断言**锚定（5 个可导航入口 + 语言行）。
- 「触感反馈（开）」的「（开）」是**默认态标注**：实现取「入口名 = 触感反馈」+「当前值 = 开/关」，
  默认 `hapticsEnabled = true` ⇒ 首屏逐字呈现「触感反馈　开」，与原文一致；且值随偏好真实变化（非假数据）。

### 1.1 子页标题（逐字）

| 子页 | navigationTitle |
| --- | --- |
| 字体大小 | `字体大小` |
| 朗读与触感 | `朗读与触感` |
| 存储空间 | `存储空间` |
| 消息通知设置 | `消息通知设置` |

---

## 二、字号档（15§5.2）

§1.2 摘录：「应用内字号档（/settings/font-size，configChanges 含 fontScale 自管）」⇒ 档位切换不重建页面。

`MeFontSize`（§2.1 逐字）= 五档：`小 / 标准 / 大 / 特大 / 最大`（rawValue 0…4，默认 `标准` = 1）。

档位 → SwiftUI `DynamicTypeSize` 映射（单调递增，施加在 `MePage` 的 `NavigationStack` 上）：

| 档位 | title（逐字） | dynamicTypeSize |
| --- | --- | --- |
| 0 | 小 | `.small` |
| 1 | 标准（默认） | `.large`（系统默认档） |
| 2 | 大 | `.xLarge` |
| 3 | 特大 | `.xxLarge` |
| 4 | 最大 | `.xxxLarge` |

- 「切换即时生效」= `.dynamicTypeSize(currentFontSize.dynamicTypeSize)` 作为**环境值**下发；
  `@StateObject` 承载偏好、子页 `@ObservedObject` 绑定 ⇒ 改档位只刷环境值，**不重建页面**，契合 15§5.2。
- 越界 index（如未持久化的脏值）回退 `.standard`，不崩溃（见 §八用例）。
- ⚠️ **跨端档位数量不一致（请主理人裁决）**：Android 侧
  `android/.../theme/LengsuoType.kt:26` 明确为**三档**（`SMALL/DEFAULT/LARGE`，factor 0.9/1.0/1.15），
  而本卡任务书 §1.4 + §2.1 指定 iOS **五档**。本卡按任务书实现五档，两端差异如实上报，未擅自对齐。

---

## 三、通知偏好矩阵（11§7 四粒度）

| # | 粒度（UI 文案，逐字） | UserDefaults key | 默认 | 关闭后的行为（本卡实现语义） |
| --- | --- | --- | --- | --- |
| 1 | 任务结果通知 | `lengsuo.me.notifyTaskResult` | **开** | 不再投递「任务完成/失败」结果通知 |
| 2 | 定时任务结果 | `lengsuo.me.notifyScheduledResult` | **开** | 不再投递定时任务触发后的结果通知 |
| 3 | 待回复/待审批提醒 | `lengsuo.me.notifyPendingApproval` | **开** | 不再投递需人工介入（待回复/待审批）提醒 |
| 4 | 退后台后接收全部通知 | `lengsuo.me.notifyGlobal` | **开** | 总开关；关闭后 App 退后台不再接收全部推送 |

四粒度**互相独立**（用例 `testNotifyFourGranularitiesRoundTripIndependently` 逐项翻转 + 往返验证）。

⚠️ **声明（诚实边界）**：任务书 §1.3 引述的 11§7「关闭后果逐条明确」原文，
**在本 worktree 与派卡根中均不存在**（全盘 `find` 未找到 `android-devguide/` 或 11 册文档，
`~/lengsuo-mobile-dev` 下仅 `ios-feature-plan.md` 含本卡一行索引）。上表「关闭后行为」是
**本卡实现语义的自述**，非逐字摘录；⛔ 未编造 11§7 原文。若 11§7 的关闭后果文案有既定措辞，
需后续卡对齐（属文案对齐，不影响本卡结构）。

### 3.1 服务端下发过滤 = 未接（P2，明确声明）

11§7 R11-3「偏好…下发给服务端做推送侧过滤」：本卡**只做本地偏好 UI + UserDefaults 持久化**，
**未接**任何服务端下发/远端过滤（无网络层、无 API 调用）。服务端侧过滤属 **P2**，本卡不实现。

---

## 四、存储空间：占位声明（⛔ 假数据）

- `StoragePage` 仅一行：`LabeledContent("已用空间", value: "计算中…")`，**恒为占位**。
- **不接**文件系统/缓存统计，**不显示任何具体容量数字**。
- 由用例 `testStoragePageHasNoFakeNumbers` 机械保证：抽取 `StoragePage` 源码块，
  断言含「计算中…」且 `block.contains(where: \.isNumber) == false`（块内**一个数字字符都不许有**）。
- ⚠️ 真实占用统计属 P2。

---

## 五、接线偏差声明（本卡不改，需主理人知悉）

### 偏差 1：`Shell/MainShell.swift` 未提供 `NavigationStack` ⇒ 本页自持一层

`Shell/MainShell.swift:29` 以 `case .me: MePage()` 直接置于 `TabView`，**全树无 `NavigationStack`**
（`Scheduled/` 的 `CreateTaskSheet`/`ScheduledFormView` 各自内嵌，与本页无关）。
`NavigationLink` 无导航容器则不生效，而 `Shell/` 是本卡**禁区** ⇒ `MePage` 自持一层 `NavigationStack`。

**需主理人处理**：若后续壳层卡补上导航容器（或统一为每个 Tab 提供 `NavigationStack`），
应删除 `MePage` 这层，以免 `NavigationStack` 嵌套。

### 偏差 2：主题三段控件未接 `ThemeManager`（非本卡职责）

`MePage.swift` 的主题 `Picker` 仍绑定页内 `@State`。**前置卡 `mb-ios-theme` 现已合入**
（`ios/Sources/LengsuoMobile/Theme/ThemeManager.swift` 存在，本 worktree 实测可见），
但**接线属独立卡**；本卡仅**更新了原文件里已过时的「ThemeManager 未合入」注释**
（该注释现与仓库事实不符），**未改绑定逻辑**、未触碰 `Theme/`。

`MePage.ThemeOption` 嵌套枚举与既有 `MePageTests` 断言（`["系统","浅色","深色"]`、默认 `.system`）**保持不变**。

---

## 六、验收（真跑，含退出码）

命令：`cd ios && swift build && swift test`（Xcode 15.4 / Swift 5.10，task 指定验收）。

```
$ swift build
[4/5] Compiling LengsuoMobile MeSettingsPages.swift
[5/5] Compiling LengsuoMobile MePage.swift
Build complete! (1.21s)
BUILD_EXIT=0

$ swift test
Test Suite 'All tests' passed at 2026-10-24 23:02:09.356.
	 Executed 37 tests, with 0 failures (0 unexpected) in 0.027 (0.032) seconds
TEST_EXIT=0
```

本卡用例（`swift test --filter MeSettingsTests`）：

```
Test Suite 'MeSettingsTests' passed at 2026-10-24 23:01:22.140.
	 Executed 11 tests, with 0 failures (0 unexpected) in 0.014 (0.014) seconds
```

11 个用例：`testHapticsDefaultOn`、`testHapticsRoundTrips`、`testFontSizeFiveLevels`、
`testFontSizeTitlesVerbatimAndDefaultStandard`、`testFontSizeIndexMapsToAscendingDynamicTypeSize`、
`testFontSizePersistence`、`testReadingDefaultsOffAndRoundTrips`、`testNotifyDefaultsAllOn`、
`testNotifyFourGranularitiesRoundTripIndependently`、`testEntryNamesAndSubPageTitlesVerbatim`、
`testStoragePageHasNoFakeNumbers`（其中前 4 个为任务书 §2.4 指定用例，逐字保留）。

### 6.1 额外证据：真 iOS SDK 编译

```
$ xcodebuild -scheme LengsuoMobile -destination 'generic/platform=iOS' build
    ... -target arm64-apple-ios17.0 -isysroot .../iPhoneOS17.5.sdk ...
** BUILD SUCCEEDED **
IOS_BUILD_EXIT=0
```

即四个子页（`Form`/`List`/`LabeledContent`/`DynamicTypeSize`）在 **iOS 17 SDK 下真实编译通过**。

### 6.2 ⛔ 不声称真机

以上为**编译 + 单元测试**级真跑。**未**在真机/模拟器上做 UI 交互验收，
**不声称**已真机验证（本卡无真机环境，按铁律 8 如实声明）。

### 6.3 关于 `--ignored` / 参考二进制

本卡是**纯 Swift（SwiftPM）卡**，无 Rust、无 FFI、无参考二进制互通路径 ⇒
**无 `#[ignore]` 用例，无零用例 exit 0 的冒充**。互通类验证不适用本卡
（iOS↔服务端偏好下发属 P2，未接）。

---

## 七、负向对照（证明用例非空转）

用「故意违反 → 断言必须失败」验证两条源码级用例**不是恒真**：

| 注入的违规 | 期望 | 实测 |
| --- | --- | --- |
| `StoragePage` 内加 `Text("128 MB")` | `testStoragePageHasNoFakeNumbers` 失败 | ✅ 失败：`XCTAssertFalse failed - 存储页不得出现任何具体数字（⛔ 假数据）` |
| `MePage` 入口名「触感反馈」→「触感」 | `testEntryNamesAndSubPageTitlesVerbatim` 失败 | ✅ 失败：`XCTAssertTrue failed - 入口名缺失：触感反馈` |

结果：`Executed 11 tests, with 2 failures`。随后**已完整回滚**，
`git status` 确认源码与 commit #2 一致（仅测试文件未跟踪），重跑 37/37 全绿。

---

## 八、边界与红线自检

- ✅ 只改 `ios/Sources/LengsuoMobile/Me/` + `ios/Tests/LengsuoMobileTests/MeSettingsTests.swift`
  + 本报告 `outputs/mb-ios-me-settings-pages-report.md`。
- ✅ **未碰**共享区（`Cargo.toml`/`Cargo.lock`/`rust-toolchain.toml`/`scripts`/`.github`）、
  `android/`、`ffi/`、`Package.swift`、`Shell/`、`Home/`、`Discover/`、`Theme/`、`Scheduled/`、`Automation/`
  （`git diff --stat HEAD~2 -- ios/Package.swift Cargo.toml` 为空）。
- ✅ **零新依赖**（`Package.swift` 未改；只用 SwiftUI/Foundation）。
- ✅ 单文件 ≤400 行。⛔ 无 push / merge；未动主仓。
- ✅ 未改 `Shell/LengsuoStrings.swift`：`navigationTitle` 复用其既有 `tabMe`。

---

## 九、遗留 / 交主理人

1. **导航容器归属**（§五偏差1）：壳层补 `NavigationStack` 后删 `MePage` 自持层。
2. **主题接线**（§五偏差2）：派独立接线卡把主题 `Picker` 接到 `ThemeManager`。
3. **11§7 关闭后果文案对齐**（§三声明）：需权威文案来源。
4. **服务端推送侧过滤**（§3.1）：P2，未接。
5. **字号档跨端不一致**（§二）：iOS 五档 vs Android 三档，待主理人裁决是否收敛。
6. **存储占用统计**：P2，恒占位「计算中…」。