import XCTest
@testable import LengsuoMobile

final class ScheduledCreateTests: XCTestCase {
    func testEmptyNameInvalid() {
        XCTAssertEqual(ScheduleDraft(name: "  ").validate(), "请填写任务名称")
    }

    func testDailyCron() {
        let d = ScheduleDraft(name: "x", frequency: .daily, hour: 8, minute: 0)
        XCTAssertEqual(d.cronExpression(), "0 8 * * *")
    }

    func testWeeklyCronUsesWeekday() {
        let d = ScheduleDraft(name: "x", frequency: .weekly, hour: 9, minute: 30, weekday: 3)
        XCTAssertEqual(d.cronExpression(), "30 9 * * 3")
    }

    func testStoreRoundTrip() {
        UserDefaults.standard.removeObject(forKey: "lengsuo.scheduled.tasks")
        let d = ScheduleDraft(name: "测试任务", frequency: .daily)
        let t = ScheduledTaskStore.add(name: d.name, draft: d)
        XCTAssertEqual(ScheduledTaskStore.load().first?.name, "测试任务")
        ScheduledTaskStore.remove(id: t.id)
        XCTAssertTrue(ScheduledTaskStore.load().isEmpty)
        UserDefaults.standard.removeObject(forKey: "lengsuo.scheduled.tasks")
    }
}
