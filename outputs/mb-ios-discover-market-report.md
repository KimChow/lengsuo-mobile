# mb-ios-discover-market 报告 —— iOS 发现子页：五入口落地 + 列表骨架（SwiftUI）

- lane：`ios`　派卡根：108　worktree：`~/lengsuo-mobile-dev/worktrees/mb-ios-discover-market`
- 日期：2026-10-24　环境：macOS 14.6 (Darwin 23.6.0) + Xcode 15.4 + Swift 5.10 (swift-tools 5.10)
- 边界：仅 `ios/Sources/LengsuoMobile/Discover/`、`ios/Tests/LengsuoMobileTests/DiscoverMarketTests.swift`、本报告。
  共享区（Cargo.*/rust-toolchain.toml/scripts/.github）、`android/`、`ffi/`、`Package.swift`、`Shell/`、其他页面目录**均未动**；未 push / 未 merge。

## 一、对齐条目（逐字）

上位依据 android-devguide `15` §2 发现行原文：
> 「搜索 + 资料库/专家/技能/连接器/项目（分册12）」

五入口 → 子页接线，入口名与 `DiscoverPage.entries` 逐字一致：

| # | 15§2 原文 | entries[i].title | 子页 `navigationTitle` | 一致 |
|---|-----------|------------------|------------------------|------|
| 1 | 资料库 | `资料库` | `资料库` | ✅ |
| 2 | 专家 | `专家` | `专家` | ✅ |
| 3 | 技能 | `技能` | `技能` | ✅ |
| 4 | 连接器 | `连接器` | `连接器` | ✅ |
| 5 | 项目 | `项目` | `项目` | ✅ |

五项命名无增删改；`testCategoryTitlesRoundTrip` / `testCategoryRoundTripsForAllFiveEntries` 锚定。

## 二、五入口 → 子页接线声明

`DiscoverPage.swift`（36 行）：

- 五入口行整行包进 `NavigationLink { MarketListPage(category: e.title, query: query) }`，
  搜索框 `@State query` **透传**给子页（`testDiscoverPageWiresAllEntrancesToMarketListPage` 断言该调用式逐字存在）。
- **删除自绘 `chevron.right`**：`List` 行内 `NavigationLink` 自带 chevron，保留会出双箭头；
  断言 `DiscoverPage.swift` 全文不再出现 `chevron.right`。
- **接线偏差（本卡声明，非本卡职责）**：`Shell/MainShell.swift` 未提供 `NavigationStack`，而 `Shell/` 为本卡禁区
  ⇒ 本页自持一层 `NavigationStack`（与已合入的 `Me/MePage.swift` 同处理），并挂
  `navigationTitle(LengsuoStrings.tabDiscover)`（复用既有常量，未新增文案）。
  若后续壳层补上导航容器，应删除本页这层以免嵌套。**未自行修改 `Shell/`。**

`MarketListPage.swift`（31 行）：五入口统一子页 `MarketListPage(category:query:)`，
`category` 作 `navigationTitle`；`query` 非空时展示「搜索：<q>」脚注行（**仅展示 @ 前缀，不触发检索**）。

## 三、空态策略（⛔ 假数据）

- 列表恒为通用空态占位「**暂无内容**」（页内 `frame(maxWidth: .infinity, alignment: .center)` 居中）。
- **⛔ 不造假商品/假专家**：不落任何条目数组、名称、价格、评分、下载量。
  `testEmptyStateAndNoFakeData` 扫描渲染体，断言不含 `商品/价格/¥/$/下载量/评分` 任一文案
  （扫描范围限定 `public var body` 之后的渲染体；文件头注释里的「⛔ 假数据」声明不算违规）。
- 与既有 `Me/MeSettingsPages.swift` 的 `StoragePage`「计算中…」同策：占位而非假数字。

## 四、「市场接口属 P2」声明

- 市场域（12 册概览要点）：技能市场 `skill-market/{categories,list}` + 专家 `expert` + 连接器 `connector` + 项目 `project`。
  其接口/通道层属 **P2**，本卡**不接网络、不落任何真实分类数据**（12 册市场分类名本卡未使用，
  因列表为空态 ⇒ 无分类名可逐字对齐；分类名逐字要求待 P2 落数据时生效）。
- 路由证据（15§3）：`/expert(6) /skill(3) /skill-market/{categories,list} /experts/custom /project(-apply(/))(/create) /projects/`
  —— iOS 侧本卡仅落到统一 `MarketListPage` 骨架，未映射具体路由。
- 搜索框带入的 `query` **只展示不检索**（`testQueryIsPassthroughOnlyNotRetrieval`：含空白原样保留，不做裁剪/触发）。

## 五、真跑验收（108 本机实跑）

```
$ cd ios && swift build
Build complete! (0.20s)                                                        # exit 0

$ swift test
Test Suite 'All tests' passed at 2026-10-24 23:32:15.737.
	 Executed 48 tests, with 0 failures (0 unexpected) in 0.029 (0.034) seconds   # exit 0

$ swift test --filter DiscoverMarketTests
Test Suite 'DiscoverMarketTests' passed at 2026-10-24 23:32:14.971.
	 Executed 7 tests, with 0 failures (0 unexpected) in 0.006 (0.009) seconds   # exit 0
```

- 三处退出码均为 **0**：build 0、全套件 48 tests / 0 failures、本卡 7 用例 / 0 failures。
- 首轮实跑曾 **1 failure**：自写的 `testEmptyStateAndNoFakeData` 因文件头注释含「⛔ 假商品/假专家数据」
  被朴素全文扫描误报 ⇒ 已把扫描范围收敛到渲染体后复跑全绿（该失败与修复均如实记录）。
- **无 `#[ignore]` 用例**：本卡为纯静态 SwiftUI 骨架，无 Rust/FFI 互通面，无参考二进制可互操作对象
  ——故无 `--ignored` 实跑项，**未用自家 roundtrip 冒充互通**，也不存在「零用例 exit 0 报成功」。
- 未声称真机运行；以上均为 macOS 下 SwiftPM 编译与 XCTest 结果。

## 六、变更清单（3 commits）

1. `4165201` feat(ios/discover): 五入口接入 MarketListPage 子页（DiscoverPage.swift 改写 + MarketListPage.swift 新建）
2. `9535390` test(ios/discover): DiscoverMarketTests 7 用例
3. 本提交 docs(ios/discover): 本报告

文件：改写 `ios/Sources/LengsuoMobile/Discover/DiscoverPage.swift`（36 行）、
新建 `ios/Sources/LengsuoMobile/Discover/MarketListPage.swift`（31 行）、
新建 `ios/Tests/LengsuoMobileTests/DiscoverMarketTests.swift`（约 85 行）——均 ≤400 行红线内。

## 七、遗留 / 跨卡事项（未自行改动，供收尾）

1. **`Shell/MainShell.swift` 缺少 `NavigationStack`**：现由 `MePage` 与 `DiscoverPage` 各自自持一层；
   建议壳层卡统一补一层 `NavigationStack`，届时删除两页各自持有的导航容器（避免嵌套）。
   本卡按铁律未改 `Shell/`。
2. 五入口子页的真实列表、分类名逐字（12 册 `skill-market/{categories,list}` 等）与路由映射：属后续卡。
3. 市场数据接口与网络：属 **P2**（12 册）。
4. 图标仍为 SF Symbol 占位（沿用 mb-ios-discover 卡惯例）；`cable.connector` 属 macOS 14+ 符号，
   包声明 `.macOS(.v13)` ⇒ macOS 13 下该图标不渲染（本卡不改 `Package.swift`，测试不断言渲染）。
5. 无需改共享区 / `android/` / `ffi/` / `Package.swift` —— 本卡无跨 crate 改动需求。