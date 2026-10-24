import Foundation

/// 快捷档：R11-1「cronFrequencyType 作为 UI 快捷档（免让用户写 cron）」。
public enum ScheduleQuickFrequency: String, CaseIterable, Identifiable {
    case once, daily, weekly, monthly

    public var id: String { rawValue }

    public var title: String {
        switch self {
        case .once: return "一次性"
        case .daily: return "每天"
        case .weekly: return "每周"
        case .monthly: return "每月"
        }
    }
}

/// 表单草稿 + 校验 + cron 生成（纯函数，可测）。
public struct ScheduleDraft: Equatable {
    public var name: String
    public var frequency: ScheduleQuickFrequency
    public var hour: Int      // 0...23
    public var minute: Int    // 0...59
    public var weekday: Int   // 1...7（仅 weekly 用，1=周一）

    public init(name: String = "", frequency: ScheduleQuickFrequency = .once,
                hour: Int = 8, minute: Int = 0, weekday: Int = 1) {
        self.name = name; self.frequency = frequency
        self.hour = hour; self.minute = minute; self.weekday = weekday
    }

    /// 校验错误；nil = 合法。
    public func validate() -> String? {
        if name.trimmingCharacters(in: .whitespaces).isEmpty { return "请填写任务名称" }
        guard (0...23).contains(hour) else { return "小时超出范围" }
        guard (0...59).contains(minute) else { return "分钟超出范围" }
        if frequency == .weekly && !(1...7).contains(weekday) { return "请选择星期" }
        return nil
    }

    /// cronExpr 自动生成（R11-1：调度收敛到 cron）。
    public func cronExpression() -> String {
        switch frequency {
        case .once: return "\(minute) \(hour) * * *"
        case .daily: return "\(minute) \(hour) * * *"
        case .weekly: return "\(minute) \(hour) * * \(weekday)"
        case .monthly: return "\(minute) \(hour) 1 * *"
        }
    }
}
