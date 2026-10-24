import Foundation
import Combine

/// 频率 7 档，顺序与 Android 卡逐字一致：每天 → 每 30 分钟 → 每 1 小时 → 每 2 小时 → 每 4 小时 → 每 8 小时 → 双周。
public enum ScheduleFrequency: String, CaseIterable, Identifiable {
    case daily
    case every30Minutes
    case everyHour
    case every2Hours
    case every4Hours
    case every8Hours
    case biweekly

    /// 默认档：每天（与 Android 卡一致）。
    public static let `default`: ScheduleFrequency = .daily

    public var id: String { rawValue }

    public var title: String {
        switch self {
        case .daily: return LengsuoStrings.Sched.frequencyDaily
        case .every30Minutes: return LengsuoStrings.Sched.frequencyEvery30Minutes
        case .everyHour: return LengsuoStrings.Sched.frequencyEveryHour
        case .every2Hours: return LengsuoStrings.Sched.frequencyEvery2Hours
        case .every4Hours: return LengsuoStrings.Sched.frequencyEvery4Hours
        case .every8Hours: return LengsuoStrings.Sched.frequencyEvery8Hours
        case .biweekly: return LengsuoStrings.Sched.frequencyBiweekly
        }
    }
}

/// 添加弹层 3 项，顺序与 Android 卡逐字一致：自定义添加 → 以对话方式添加 → 取消。
public enum ScheduleAddOption: Int, CaseIterable, Identifiable {
    case custom
    case conversation
    case cancel

    public var id: Int { rawValue }

    public var title: String {
        switch self {
        case .custom: return LengsuoStrings.Sched.sheetCustom
        case .conversation: return LengsuoStrings.Sched.sheetConversation
        case .cancel: return LengsuoStrings.Sched.sheetCancel
        }
    }
}

/// 定时任务条目（本地内存；无持久化/网络/调度）。
public struct ScheduledTask: Identifiable, Equatable {
    public let id: UUID
    public var name: String
    public var prompt: String
    public var frequency: ScheduleFrequency
    public let createdAt: Date

    public init(
        id: UUID = UUID(),
        name: String,
        prompt: String,
        frequency: ScheduleFrequency = ScheduleFrequency.default,
        createdAt: Date = Date()
    ) {
        self.id = id
        self.name = name
        self.prompt = prompt
        self.frequency = frequency
        self.createdAt = createdAt
    }
}

/// 本地内存仓库：新增插到列表首位（最新在前，与 Android 卡一致）。
public final class ScheduledStore: ObservableObject {
    @Published public private(set) var tasks: [ScheduledTask] = []

    public init() {}

    public func add(name: String, prompt: String, frequency: ScheduleFrequency = ScheduleFrequency.default) {
        tasks.insert(ScheduledTask(name: name, prompt: prompt, frequency: frequency), at: 0)
    }

    public func remove(at offsets: IndexSet) {
        tasks.remove(atOffsets: offsets)
    }

    public func remove(_ task: ScheduledTask) {
        tasks.removeAll { $0.id == task.id }
    }
}

/// 表单校验：名称与内容均非空才可创建（与 Android 卡一致）。
public func isFormCreatable(name: String, prompt: String) -> Bool {
    !name.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
        && !prompt.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
}
