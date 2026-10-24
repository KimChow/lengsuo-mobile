import XCTest
@testable import LengsuoMobile

final class MainShellTests: XCTestCase {
    func testTabOrderMatchesSpec() {
        // android-devguide/15 §2：任务 → 定时任务 → 发现 → 我的
        let titles = MainTab.allCases.map(\.title)
        XCTAssertEqual(titles, ["任务", "定时任务", "发现", "我的"])
    }

    func testFirstTabIsTasks() {
        XCTAssertEqual(MainTab.allCases.first, .tasks)
    }

    func testComingSoonCopyVerbatim() {
        XCTAssertEqual(LengsuoStrings.comingSoon, "敬请期待")
    }
}
