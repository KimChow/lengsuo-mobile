import XCTest
@testable import LengsuoMobile

final class DiscoverMarketTests: XCTestCase {
    // MARK: - 卡内指定用例（逐字）

    func testCategoryTitlesRoundTrip() {
        // 五入口名与 DiscoverPage.entries 一致（15§2 逐字）
        let titles = DiscoverPage.entries.map(\.title)
        XCTAssertEqual(titles, ["资料库", "专家", "技能", "连接器", "项目"])
        // 子页标题可由任一入口构造
        let page = MarketListPage(category: titles[0], query: "")
        XCTAssertEqual(MarketListPage(category: titles[0]).category, "资料库")
        _ = page
    }

    func testQueryPassthrough() {
        XCTAssertEqual(MarketListPage(category: "专家", query: "周报").query, "周报")
    }

    // MARK: - 本卡补充用例

    func testCategoryRoundTripsForAllFiveEntries() {
        // 五个入口逐个构造子页：标题即入口名，一字不改（12 册市场分类名逐字）
        for entry in DiscoverPage.entries {
            XCTAssertEqual(MarketListPage(category: entry.title).category, entry.title)
        }
        // 默认 query 为空 ⇒ 不渲染带入搜索行
        XCTAssertEqual(MarketListPage(category: "连接器").query, "")
    }

    func testQueryIsPassthroughOnlyNotRetrieval() {
        // 带入的 query 原样保存，本卡不做任何检索/裁剪（检索属 P2）
        let raw = "  周报  "
        XCTAssertEqual(MarketListPage(category: "技能", query: raw).query, raw)
    }

    // MARK: - 文案逐字 / 红线（源码级）

    func testEmptyStateAndNoFakeData() throws {
        let src = try String(contentsOf: discoverSourceDir().appendingPathComponent("MarketListPage.swift"),
                             encoding: .utf8)
        XCTAssertTrue(src.contains("暂无内容"), "空态应为「暂无内容」通用占位")
        XCTAssertTrue(src.contains("搜索："), "带入 query 应展示「搜索：」前缀")
        // ⛔ 无任何假商品/假专家数据：只看渲染体（渲染体才是用户可见文案；
        //    文件头注释里的「⛔ 假数据」声明不算违规）
        let body = try XCTUnwrap(src.range(of: "public var body: some View {"))
        let rendered = String(src[body.lowerBound...])
        for banned in ["商品", "价格", "¥", "$", "下载量", "评分"] {
            XCTAssertFalse(rendered.contains(banned), "发现子页渲染体不得出现假数据文案：\(banned)")
        }
    }

    func testDiscoverPageWiresAllEntrancesToMarketListPage() throws {
        let src = try String(contentsOf: discoverSourceDir().appendingPathComponent("DiscoverPage.swift"),
                             encoding: .utf8)
        // 五入口行全部包进 NavigationLink → MarketListPage(category:query:)
        XCTAssertTrue(src.contains("NavigationLink"), "入口行应接 NavigationLink")
        XCTAssertTrue(src.contains("MarketListPage(category: e.title, query: query)"),
                      "子页应透传入口名与搜索 query")
        // List 行内 NavigationLink 自带 chevron ⇒ 不得再自绘，避免双箭头
        XCTAssertFalse(src.contains("chevron.right"), "入口行不得再自绘 chevron（避免双箭头）")
    }

    func testSourceFilesWithinLineBudget() throws {
        for name in ["DiscoverPage.swift", "MarketListPage.swift"] {
            let src = try String(contentsOf: discoverSourceDir().appendingPathComponent(name), encoding: .utf8)
            XCTAssertLessThanOrEqual(src.split(separator: "\n").count, 400, "\(name) 超出 400 行红线")
        }
    }

    /// `ios/Tests/LengsuoMobileTests/<本文件>` → `ios/Sources/LengsuoMobile/Discover`
    private func discoverSourceDir() -> URL {
        URL(fileURLWithPath: #filePath)          // .../ios/Tests/LengsuoMobileTests/DiscoverMarketTests.swift
            .deletingLastPathComponent()          // .../ios/Tests/LengsuoMobileTests
            .deletingLastPathComponent()          // .../ios/Tests
            .deletingLastPathComponent()          // .../ios
            .appendingPathComponent("Sources/LengsuoMobile/Discover")
    }
}