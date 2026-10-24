import XCTest
@testable import LengsuoMobile

final class DrawerTests: XCTestCase {
    func testCopyVerbatim() {
        // android strings.xml drawer_* 4 条
        XCTAssertEqual(DrawerCopy.cloudGroup, "☁ 云端 ▾")
        XCTAssertEqual(DrawerCopy.newTask, "新建任务")
        XCTAssertEqual(DrawerCopy.emptyCloud, "暂无云端任务")
        XCTAssertEqual(DrawerCopy.taskGroup, "任务 ▾")
    }

    func testSampleConversationsVerbatim() {
        XCTAssertEqual(DrawerCopy.sampleConversations, ["hello", "周报整理", "旅行计划"])
    }

    func testSelectedPredicate() {
        XCTAssertTrue(drawerIsSelected(index: 1, selectedIndex: 1))
        XCTAssertFalse(drawerIsSelected(index: 1, selectedIndex: nil))
    }

    func testEmptyPredicate() {
        XCTAssertTrue(drawerIsEmpty([]))
        XCTAssertFalse(drawerIsEmpty(["a"]))
    }
}