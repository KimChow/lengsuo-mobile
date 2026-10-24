import XCTest
@testable import LengsuoMobile

final class AutomationListTests: XCTestCase {
    func testEmptyCopyVerbatim() {
        // devguide 11 §3/§8 原文
        XCTAssertEqual(AutomationCopy.emptyProject, "No automation tasks for this project")
        XCTAssertEqual(AutomationCopy.detailLoadFailed, "Failed to load task detail(s)")
        XCTAssertEqual(AutomationCopy.cacheUnavailable,
                       "Local task list cache unavailable (storage/account scope not ready)")
    }

    func testDetailArtifactPlaceholder() {
        let t = AutomationTask(name: "x", status: "scheduled", scheduleText: "0 8 * * *")
        XCTAssertEqual(t.artifact, "—")
    }

    // MARK: - 本地仓 → 详情视图模型映射（§8 三元素）

    func testMapFromStoredTaskKeepsIdNameAndSchedule() {
        let stored = StoredTask(id: "t-1", name: "每日汇报",
                                cronExpr: "0 8 * * *", frequency: "daily")
        let detail = AutomationTask.from(stored)

        XCTAssertEqual(detail.id, "t-1")
        XCTAssertEqual(detail.name, "每日汇报")
        XCTAssertEqual(detail.status, "scheduled")
        XCTAssertEqual(detail.scheduleText, "0 8 * * *")
        XCTAssertEqual(detail.artifact, "—")   // 本卡无产物通道，恒为占位
    }

    func testDetailViewRendersAllThreeElements() {
        let stored = StoredTask(id: "t-2", name: "n",
                                cronExpr: "30 9 * * 3", frequency: "weekly")
        let view = AutomationDetailView(task: stored)

        XCTAssertEqual(view.task.status, "scheduled")
        XCTAssertEqual(view.task.scheduleText, "30 9 * * 3")
        XCTAssertEqual(view.task.artifact, "—")
    }
}