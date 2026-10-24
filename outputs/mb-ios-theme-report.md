# mb-ios-theme 验收报告

> 卡：`mb-ios-theme`　lane：`ios`　worktree：`worktrees/mb-ios-theme`（基线 `d148d54`）
> 结论：**通过**。`swift build` exit 0；`swift test` exit 0（Executed 16 tests, 0 failures）。

## 1. 交付物（4 commits，全部在卡边界内）

| commit | 文件 | 行数（≤400 ✓） |
| --- | --- | --- |
| ① `2acacb2` | `ios/Sources/LengsuoMobile/Theme/LengsuoTheme.swift` | 91 |
| ② `329b975` | `ios/Sources/LengsuoMobile/Theme/ThemeManager.swift` | 20 |
| ③ `16e7f3e` | `ios/Tests/LengsuoMobileTests/ThemeTests.swift` | 24 |
| ④ | `outputs/mb-ios-theme-report.md`（本报告） | — |

`git status` 收尾时干净；未触碰 `Shell/`、`Package.swift`、共享区（Cargo/scripts/.github）、无新增依赖、未 push。

## 2. 三态对齐声明（逐字）

- android-devguide/15 §5.2：「主题三态：系统/浅色/深色（我的页三段控件）。」
- 同文档 §2「我的」行：「主题（系统/浅色/深色 三段选择）」

实现：`ColorSchemePreference { system, light, dark }`，`title` 返回 `系统/浅色/深色`（逐字，测试 `testThreeStatesVerbatim` 锁定）；顺序 `[.system, .light, .dark]`（`testOrderMatchesSpec` 锁定）。`scheme`：system→nil（交还系统）、light→`.light`、dark→`.dark`，供 `preferredColorScheme` 使用；`ThemeManager` 为 `ObservableObject`（`@Published`），变更即时生效、不重建视图。

持久化：仅内存 + `UserDefaults`（key=`lengsuo.theme.preference`），未做 Keychain/网络。往返由 `testPersistenceRoundTrip` 锁定。

## 3. 色板实现方式：**动态色**（二选一中选此项，非 assets catalog）

理由：`Package.swift` 未声明任何 resources，走 assets catalog 必须改 `Package.swift`（本卡禁区）；故用 `UIColor { trait in … }` / `NSColor(name:dynamicProvider:)` 动态提供者实现 `Color(light:dark:)` 语义。且包平台为 iOS 17 + **macOS 13**（测试宿主），故按 `#if canImport(UIKit) / #elseif canImport(AppKit)` 双分支实现，两宿主均可编译。

### 色值来源（⛔ 无一自造，逐项取自 android 线 `LengsuoColors.kt` 参考实测值）

| token（卡定槽位，未改名） | android 来源字段 | light | dark |
| --- | --- | --- | --- |
| `background` | pageBackground | `#FFFFFF` | `#0F1011` |
| `surface` | dialogBackground | `#FFFFFF` | `#333333` |
| `primary` | primary | `#1A1A1A` | `#FFFFFF` |
| `secondary` | mainThirdText | `#999999` | `#888888` |
| `textPrimary` | mainText | `#333333` | `#FFFFFF` |
| `textSecondary` | mainSecondText | `#666666` | `#ABABAB` |
| `accent` | BrandMain | `#005ED9` | `#005ED9`（实测仅亮色定义 ⇒ 明暗共用） |

视图层只许引用 `LengsuoTheme.<token>`，禁硬编码色值（红线 7）。本卡未接任何视图（Me 页归 `mb-ios-me-page`）。

## 4. 验收真实输出（108 本机实跑，Xcode 15.4 / swift-tools-5.10）

### `cd ios && swift build` → **exit 0**

```
Building for debugging...
[0/2] Write sources
[1/2] Write swift-version-33747A42983211AE.txt
[3/12] Compiling LengsuoMobile ScheduledPageView.swift
[4/12] Compiling LengsuoMobile LengsuoStrings.swift
[5/12] Compiling LengsuoMobile MainShell.swift
[6/15] Compiling LengsuoMobile DiscoverPage.swift
[7/15] Compiling LengsuoMobile TasksHomePage.swift
[8/15] Compiling LengsuoMobile LengsuoMobile.swift
[9/15] Emitting module LengsuoMobile
[10/15] Compiling LengsuoMobile MePage.swift
[11/15] Compiling LengsuoMobile ScheduledFormView.swift
[12/15] Compiling LengsuoMobile ScheduledModels.swift
[13/15] Compiling LengsuoMobile MainTab.swift
[14/15] Compiling LengsuoTheme.swift
[15/15] Compiling LengsuoMobile ThemeManager.swift
Build complete! (13.85s)
```

### `swift test` → **exit 0**

```
Test Suite 'ThemeTests' passed at 2026-10-24 22:16:01.880.
	 Executed 3 tests, with 0 failures (0 unexpected) in 0.003 (0.003) seconds
Test Suite 'LengsuoMobilePackageTests.xctest' passed at 2026-10-24 22:16:01.880.
	 Executed 16 tests, with 0 failures (0 unexpected) in 0.008 (0.009) seconds
Test Suite 'All tests' passed at 2026-10-24 22:16:01.881.
	 Executed 16 tests, with 0 failures (0 unexpected) in 0.008 (0.011) seconds
```

新增 3 用例全绿：`testThreeStatesVerbatim` / `testOrderMatchesSpec` / `testPersistenceRoundTrip`。

⚠️ 与卡面预期的偏差（如实声明）：卡写「Executed 7 tests（4 既有 + 3 新）」，实际 **16 = 13 既有 + 3 新**——既有测试在 `mb-ios-scheduled` 卡已增至 13（LengsuoMobile 1 + MainShell 3 + Scheduled 9），卡面数字过期，非本次引入回归。0 failures 不变。

## 5. 边界与移交

- 无 `#[ignore]` 用例、无参考二进制需求（本卡纯 SwiftUI 数据层，无互通项）。
- 无跨 crate / 共享区改动需求。
- 未做（按卡范围）：字号档/双语/朗读/触感、Me 页 UI 接线、Rive/吉祥物、真机验证（未声称）。
- 移交下游：`mb-ios-me-page` 直接消费 `ThemeManager` + `LengsuoTheme` token；宿主 App 层接 `preferredColorScheme(manager.preference.scheme)`。
