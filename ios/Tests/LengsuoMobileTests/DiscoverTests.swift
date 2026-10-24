import XCTest
@testable import LengsuoMobile

final class DiscoverTests: XCTestCase {
    func testFiveEntriesVerbatim() {
        // 15§2 发现行逐字：资料库/专家/技能/连接器/项目
        XCTAssertEqual(DiscoverPage.entries.map(\.title),
                       ["资料库", "专家", "技能", "连接器", "项目"])
    }

    func testEntryCountIsFive() {
        XCTAssertEqual(DiscoverPage.entries.count, 5)
    }
}
