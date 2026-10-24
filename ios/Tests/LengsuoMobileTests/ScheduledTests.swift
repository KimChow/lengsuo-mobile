import XCTest
@testable import LengsuoMobile

final class ScheduledTests: XCTestCase {
    // MARK: - 状态机 §1.3

    func testFrequencyOrderMatchesAndroidCard() {
        // 每天 → 每 30 分钟 → 每 1 小时 → 每 2 小时 → 每 4 小时 → 每 8 小时 → 双周
        XCTAssertEqual(ScheduleFrequency.allCases, [
            .daily, .every30Minutes, .everyHour, .every2Hours, .every4Hours, .every8Hours, .biweekly,
        ])
    }

    func testDefaultFrequencyIsDaily() {
        XCTAssertEqual(ScheduleFrequency.default, .daily)
        let store = ScheduledStore()
        store.add(name: "n", prompt: "p")
        XCTAssertEqual(store.tasks.first?.frequency, .daily)
    }

    func testAddOptionOrderMatchesAndroidCard() {
        // 弹层 3 项顺序：自定义添加 → 以对话方式添加 → 取消
        XCTAssertEqual(ScheduleAddOption.allCases, [.custom, .conversation, .cancel])
    }

    func testFormValidationRequiresNameAndPrompt() {
        XCTAssertFalse(isFormCreatable(name: "", prompt: "p"))
        XCTAssertFalse(isFormCreatable(name: "n", prompt: ""))
        XCTAssertFalse(isFormCreatable(name: "", prompt: ""))
        XCTAssertFalse(isFormCreatable(name: "  ", prompt: "p"))
        XCTAssertFalse(isFormCreatable(name: "n", prompt: " \n "))
        XCTAssertTrue(isFormCreatable(name: "n", prompt: "p"))
        XCTAssertTrue(isFormCreatable(name: " 每日同步 ", prompt: " 汇总动态 "))
    }

    func testStoreAddInsertsAtFrontAndRemove() {
        let store = ScheduledStore()
        store.add(name: "第一", prompt: "p1", frequency: .every30Minutes)
        store.add(name: "第二", prompt: "p2", frequency: .daily)
        // 新增插到列表首位（最新在前）
        XCTAssertEqual(store.tasks.map(\.name), ["第二", "第一"])
        XCTAssertEqual(store.tasks.first?.frequency, .daily)

        store.remove(store.tasks[0])
        XCTAssertEqual(store.tasks.map(\.name), ["第一"])

        store.add(name: "第三", prompt: "p3")
        store.remove(atOffsets: IndexSet(integer: 1))
        XCTAssertEqual(store.tasks.map(\.name), ["第三"])
    }

    func testScheduledTaskFieldsRoundtrip() {
        let task = ScheduledTask(name: "n", prompt: "p", frequency: .biweekly)
        XCTAssertEqual(task.name, "n")
        XCTAssertEqual(task.prompt, "p")
        XCTAssertEqual(task.frequency, .biweekly)
    }

    // MARK: - 文案绑定 §1.2（逐字对照，照抄 MainShellTests 双保险惯例）

    func testScheduledStringsVerbatim() {
        XCTAssertEqual(LengsuoStrings.Sched.title, "定时任务")
        XCTAssertEqual(LengsuoStrings.Sched.empty, "暂无定时任务")
        XCTAssertEqual(LengsuoStrings.Sched.fabAdd, "添加定时任务")
        XCTAssertEqual(LengsuoStrings.Sched.sheetCustom, "自定义添加")
        XCTAssertEqual(LengsuoStrings.Sched.sheetConversation, "以对话方式添加")
        XCTAssertEqual(LengsuoStrings.Sched.sheetCancel, "取消")
        XCTAssertEqual(LengsuoStrings.Sched.formHint, "定时任务仅支持「定时执行」一种触发形态")
        XCTAssertEqual(LengsuoStrings.Sched.namePlaceholder, "例如：每日同步项目动态")
        XCTAssertEqual(LengsuoStrings.Sched.promptPlaceholder, "在这里输入要让 AI 完成的内容，例如：汇总过去 24 小时的项目动态，并整理待跟进事项。")
        XCTAssertEqual(LengsuoStrings.Sched.scheduleTime, "执行频率")
        XCTAssertEqual(LengsuoStrings.Sched.create, "创建")
        XCTAssertEqual(LengsuoStrings.Sched.validity, "有效期")
        XCTAssertEqual(LengsuoStrings.Sched.validityLongTerm, "长期有效")
        XCTAssertEqual(LengsuoStrings.Sched.experts, "专家")
        XCTAssertEqual(LengsuoStrings.Sched.skills, "技能")
        XCTAssertEqual(LengsuoStrings.Sched.connectors, "连接器")
        XCTAssertEqual(LengsuoStrings.Sched.noSelection, "未选择 (0)")
        XCTAssertEqual(LengsuoStrings.Sched.cardRun, "运行")
        XCTAssertEqual(LengsuoStrings.Sched.actionMore, "更多操作")
        XCTAssertEqual(LengsuoStrings.Sched.errorNameRequired, "请填写定时任务名称")
    }

    func testFrequencyTitlesVerbatim() {
        XCTAssertEqual(ScheduleFrequency.allCases.map(\.title), [
            "每天", "每 30 分钟", "每 1 小时", "每 2 小时", "每 4 小时", "每 8 小时", "双周",
        ])
    }

    func testAddOptionTitlesVerbatim() {
        XCTAssertEqual(ScheduleAddOption.allCases.map(\.title), ["自定义添加", "以对话方式添加", "取消"])
    }
}
