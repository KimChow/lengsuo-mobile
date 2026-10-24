import XCTest
@testable import LengsuoMobile

final class TasksHomePageTests: XCTestCase {
    func testInitialChipsVerbatim() {
        // §2 逐字：4 条快捷胶囊文案
        XCTAssertEqual(TasksHomePage.initialChips, [
            "进行数据分析及可视化", "进行文档处理", "做一份幻灯片", "制作工作台",
        ])
    }

    func testChipsCountIsFour() {
        XCTAssertEqual(TasksHomePage.initialChips.count, 4)
    }
}
