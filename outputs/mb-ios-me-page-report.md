# mb-ios-me-page 报告 —— iOS「我的」页五段分组骨架

> 卡：`mb-ios-me-page`　lane：`ios`　worktree：`~/lengsuo-mobile-dev/worktrees/mb-ios-me-page`
> 结论：**已完成并真跑通过**（build exit 0 / test exit 0，`Executed 17 tests, with 0 failures`；红线 grep 0 命中）。
> 但存在 **1 项前置缺失导致的接线偏差**（见下），需主理人裁决。

---

## 〇、⚠️ 前置缺失与接线偏差（最重要，先读）

**卡面前置声明「`mb-ios-theme` 已合入」与仓库事实不符**：本 worktree（HEAD 866a069）全树
`grep -rn "ThemeManager" ios/` 为 **0 命中**，`ios/Sources/LengsuoMobile/` 下不存在 `Theme/` 目录，
即 `ThemeManager` / `ColorSchemePreference`（mb-ios-theme 产物）**未合入本分支**。

由此产生两处与 §3.1 逐字代码的**必要偏差**（照抄必致编译失败）：

1. **主题三段控件无法真接线 `ThemeManager`** ⇒ 降级为页内嵌套枚举
   `MePage.ThemeOption`（system/light/dark，title 逐字「系统/浅色/深色」）+ `@State` 绑定，
   是本页唯一真交互（可在三段间真实切换），但**不落全局主题生效**。
   类型嵌套于 `MePage` 内，避免与 mb-ios-theme 合入后的顶层类型冲突。
2. **`init(theme:)` 不可行** ⇒ 保留无参 `public init()`：`Shell/MainShell.swift:29` 以
   `MePage()` 构造且 Shell/ 为本卡禁区，改签名必然连坐编译失败。

**需主理人处理（本卡不改，按边界规则写进报告）**：mb-ios-theme 合入后，应派一张小接线卡：
`MePage` 增 `theme: ThemeManager` 注入、`Picker` 改绑 `theme.preference`/`theme.set(_:)`、
`MainShell` 构造处传入，并删除 `ThemeOption` 嵌套枚举。

## 一、分组对齐声明（五段 vs §6 逐字）

§6 原文：「我的页分组：身份卡（头像/昵称/箭头）→ 套餐卡（档位图标+升级按钮+积分行）→
通用设置组（主题/语言/字号/朗读/触感/存储/通知）→ 资产组（记忆库/成长计划）→ 关于。」

iOS 实现 = `List`（insetGrouped 默认样式）五个 `Section`，逐段对齐：

| §6 段 | MePage Section | 内容 |
| --- | --- | --- |
| 身份卡 | 1 | `person.circle`(44) + 「未登录」headline + chevron.right（§3.1 逐字） |
| 套餐卡 | 2 | 「体验版」+「升级套餐」borderedProminent 按钮壳（无动作）+「我的积分」+「—」 |
| 通用设置组 | 3 | 「主题」三段 Picker（真交互）+ 语言/字体大小/朗读设置/触感反馈/存储空间/消息通知设置 7 行 |
| 资产组 | 4 | 记忆库 / 成长计划 |
| 关于 | 5 | 「关于 WorkBuddy」（§2 原文照抄，含产品名） |

文案全部取自 android-devguide/15 §2/§6 与卡面 §3.1 逐字代码，无自编。

## 二、占位项清单（诚实占位，⛔ 假数据）

- 「我的积分 3236.71」→「我的积分」+ `—`：参考端数字为用户数据，不显示；
- 「体验版+升级套餐」→ 账号态占位：「体验版」标签 +「升级套餐」按钮壳（无跳转）；
- 「未登录」：无登录态骨架的诚实占位（无昵称可显）；
- 通用设置组 6 行（语言「跟随系统」/字体大小/朗读设置/触感反馈「开」/存储空间/消息通知设置）、
  资产组 2 行（记忆库/成长计划）、关于 1 行：icon+文案+chevron 行占位，无动作；
- 头像 = SF Symbol `person.circle` 占位，无网络图片。

## 三、真跑输出（Xcode 15.4，108）

```
$ cd ios && swift build
Build complete! (14.90s)
BUILD_EXIT=0

$ swift test
Test Suite 'MePageTests' passed … Executed 2 tests, with 0 failures
Test Suite 'LengsuoMobilePackageTests.xctest' passed …
Test Suite 'All tests' passed …
	 Executed 17 tests, with 0 failures (0 unexpected) in 0.006 (0.010) seconds
TEST_EXIT=0
```

说明：卡面验收预期「Executed 9 tests」为**过期数字**（撰写时未计入 Scheduled 9 例等已合入用例）；
真跑以实际输出为准：既有 15 例（Lengsuo 1 + MainShell 3 + Scheduled 9 + TasksHome 2）+ 本卡 2 例
（`testThemeOptionsVerbatimAndDefaultSystem` / `testNoFakePointsData`）= **17 例全过**。
注：原 §3.2 `testThemeBindingAvailable` 断言 `ThemeManager()` 构造，因类型不存在改为断言
`ThemeOption` 三段文案逐字 + 首项默认 `.system`（语义等价：默认系统态）。

## 四、红线 grep 证据

```
$ grep -rn "3236" ios/Sources ios/Tests
GREP_EXIT=1        # 0 命中，假数据红线通过
```

## 五、边界自查与提交

- 改动仅限本卡允许路径：`ios/Sources/LengsuoMobile/Me/MePage.swift`（90 行 ≤400）、
  `ios/Tests/LengsuoMobileTests/MePageTests.swift`（17 行）、本报告。
  未触碰 Cargo.*、rust-toolchain.toml、scripts、.github、Package.swift、Shell/、android/、ffi/。
- 提交：① `a2f13c1` feat(ios/me) MePage 五段实现　② `2116145` test(ios/me) MePageTests 2 用例
  （+ 本报告提交）。⛔ 未 push。
