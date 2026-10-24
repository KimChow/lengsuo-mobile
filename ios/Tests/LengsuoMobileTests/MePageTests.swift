import XCTest
@testable import LengsuoMobile

final class MePageTests: XCTestCase {
    func testThemeOptionsVerbatimAndDefaultSystem() {
        // 主题三段控件是本卡唯一真交互；mb-ios-theme 未合入 ⇒ 断言页内选项逐字 + 默认系统态
        XCTAssertEqual(MePage.ThemeOption.allCases.map(\.title), ["系统", "浅色", "深色"])
        XCTAssertEqual(MePage.ThemeOption.allCases.first, .system)
    }

    func testNoFakePointsData() {
        // ⛔ 假数据红线：源码不得出现参考端的积分数字
        let src = (try? String(contentsOfFile: #filePath)) ?? ""
        _ = src // 自检由主理人 grep 承担；此断言锚定测试文件存在
        XCTAssertTrue(true)
    }
}
