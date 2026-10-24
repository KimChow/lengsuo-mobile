# mb-ios-discover 报告 —— iOS 发现 Tab:搜索 + 五入口骨架

- lane:`ios`　派卡根:108　worktree:`~/lengsuo-mobile-dev/worktrees/mb-ios-discover`
- 日期:2026-10-24　环境:macOS 14.6 (Darwin 23.6.0) + SwiftPM (swift-tools 5.10)
- 边界:仅 `ios/Sources/LengsuoMobile/Discover/`、`ios/Tests/LengsuoMobileTests/DiscoverTests.swift`、本报告。共享区未动,未 push,未碰 `Package.swift`。

## 一、对齐:五入口逐字对照

上位依据 android-devguide/15 §2 发现行原文:
> 「发现 | 生态市场 | 搜索 + 资料库/专家/技能/连接器/项目(分册12)」

页面 = 顶部搜索框 + 五入口,`DiscoverPage.entries` 与原文逐字对照:

| # | 15§2 原文 | entries[i].title | 一致 |
|---|-----------|------------------|------|
| 1 | 资料库 | `资料库` | ✅ |
| 2 | 专家 | `专家` | ✅ |
| 3 | 技能 | `技能` | ✅ |
| 4 | 连接器 | `连接器` | ✅ |
| 5 | 项目 | `项目` | ✅ |

五项命名无增删改;测试 `testFiveEntriesVerbatim` / `testEntryCountIsFive` 锚定。

## 二、形态声明

- 搜索框:`TextField("搜索")` + `magnifyingglass` 图标,可输入(@State query),**本卡不触发任何检索,无任何搜索结果渲染**(无市场数据/网络调用;12 册市场接口属 P2)。
- 五入口:`List` 五行,每行 = SF Symbol 图标占位 + 文案 + `chevron.right`;**点击无动作**(NavigationLink/跳转属 P1 `discover-market` 卡)。
- 列表不放任何市场商品/专家假数据(红线 7)。
- 旧骨架的 `PlaceholderPage` 占位渲染已整体替换;`PlaceholderPage` 本体不在本卡边界内,未动。

## 三、图标占位声明

- 图标为 SF Symbol 占位,非最终设计:`资料库=folder`、`专家=person.2`、`技能=sparkles`、`连接器=cable.connector`、`项目=square.stack.3d.up`。
- `cable.connector` 在 iOS 17 SDK(SF Symbols 5)可用,Package.swift 平台为 `.iOS(.v17)`,按卡内模板**逐字保留,未降级为 `cable`**。注:包同时声明 `.macOS(.v13)`,`cable.connector` 属 macOS 14+ 图标,macOS 13 下该符号运行时不渲染(仅图标占位缺失,不影响文案与逻辑);`swift test` 在 macOS 下运行,测试不断言渲染。

## 四、真跑验收

命令(108 本机实跑):

```
$ cd ios && swift build
Build complete! (14.26s)                          # exit 0

$ swift test
Test Suite 'All tests' passed at 2026-10-24 22:29:47.686.
	 Executed 22 tests, with 0 failures (0 unexpected) in 0.008 (0.012) seconds   # exit 0

$ swift test --filter DiscoverTests
Test Case '-[LengsuoMobileTests.DiscoverTests testEntryCountIsFive]' passed (0.001 seconds).
Test Case '-[LengsuoMobileTests.DiscoverTests testFiveEntriesVerbatim]' passed (0.000 seconds).
	 Executed 2 tests, with 0 failures (0 unexpected)                              # exit 0
```

- 全套件 22 tests / 0 failures,本卡新增 2 用例均通过,退出码 0。
- 本卡无 `#[ignore]` 用例、无需参考二进制互通(纯静态 SwiftUI 骨架,无 Rust/FFI 面);未用 roundtrip 冒充互通。

## 五、变更清单(3 commits)

1. `52a881b` feat(ios/discover): DiscoverPage 搜索框+五入口骨架(资料库/专家/技能/连接器/项目)
2. `0e2e439` test(ios/discover): DiscoverTests 五入口逐字+数量=5(2 用例)
3. 本提交 docs(ios/discover): 本报告

文件:改写 `ios/Sources/LengsuoMobile/Discover/DiscoverPage.swift`(36 行,≤400 行红线内);新建 `ios/Tests/LengsuoMobileTests/DiscoverTests.swift`(14 行)。

## 六、遗留/跨卡事项

- 子页(资料库/专家/技能/连接器/项目各列表页)与点击跳转:属 P1 `discover-market` 卡。
- 市场数据接口与网络:属 P2(12 册)。
- 图标最终定稿:待设计;现行为 SF Symbol 占位惯例。
- 无需改共享区/`android/`/`ffi/`/`Package.swift`。
