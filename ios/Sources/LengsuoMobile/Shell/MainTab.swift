import Foundation

/// 底部 Tab 枚举：顺序 = android-devguide/15 §2 四 Tab 自左到右。
public enum MainTab: String, CaseIterable, Identifiable {
    case tasks
    case scheduled
    case discover
    case me

    public var id: String { rawValue }

    public var title: String {
        switch self {
        case .tasks: return LengsuoStrings.tabTasks
        case .scheduled: return LengsuoStrings.tabScheduled
        case .discover: return LengsuoStrings.tabDiscover
        case .me: return LengsuoStrings.tabMe
        }
    }

    /// SF Symbols 近似占位图标名（差异声明见任务书 §1.4）。
    public var systemImage: String {
        switch self {
        case .tasks: return "house"
        case .scheduled: return "calendar"
        case .discover: return "magnifyingglass"
        case .me: return "person"
        }
    }
}
