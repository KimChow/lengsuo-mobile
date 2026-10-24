import Foundation

/// 空态与错误文案：devguide 11 册原文逐字，⛔ 不得改写。
public enum AutomationCopy {
    public static let emptyProject = "No automation tasks for this project"
    public static let detailLoadFailed = "Failed to load task detail(s)"
    public static let cacheUnavailable = "Local task list cache unavailable (storage/account scope not ready)"
}

/// 详情页三元素（§8：状态/时间/产物）。
public struct AutomationTask: Identifiable, Equatable {
    public var id: String
    public var name: String
    public var status: String
    public var scheduleText: String
    public var artifact: String   // 本卡恒为 "—"

    public init(id: String = UUID().uuidString, name: String,
                status: String, scheduleText: String, artifact: String = "—") {
        self.id = id; self.name = name; self.status = status
        self.scheduleText = scheduleText; self.artifact = artifact
    }
}

// MARK: - 本地仓 → 视图模型映射

extension AutomationTask {
    /// 由 ScheduledStore 本地仓条目映射为详情页视图模型。
    ///
    /// ⚠️ 接线偏差：卡片伪码写的是 `ScheduledStore.load()` 与 `task.artifact`，
    /// 但本分支实际 API 是 `ScheduledTaskStore.load() -> [StoredTask]`，
    /// 且 `StoredTask` 无 `artifact` 字段（Scheduled/ 为既有目录，本卡只读其 public API，
    /// 不得改动）。故此处映射：`status` 取「已排期」的语义常量，`scheduleText` 取 cron，
    /// 产物字段由 `AutomationTask` 默认值恒为 "—"（本卡无产物通道，属 P2）。
    public static func from(_ task: StoredTask) -> AutomationTask {
        AutomationTask(id: task.id, name: task.name,
                       status: "scheduled", scheduleText: task.cronExpr)
    }
}