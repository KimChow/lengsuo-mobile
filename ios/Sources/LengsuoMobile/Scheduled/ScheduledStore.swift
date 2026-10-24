import Foundation

/// 定时任务本地持久化（UserDefaults JSON）。真实云端 scheduler 属 P2，本卡只做本地。
public struct StoredTask: Codable, Identifiable, Equatable {
    public var id: String
    public var name: String
    public var cronExpr: String
    public var frequency: String
    public var createdAt: Date

    public init(id: String = UUID().uuidString, name: String,
                cronExpr: String, frequency: String, createdAt: Date = Date()) {
        self.id = id; self.name = name; self.cronExpr = cronExpr
        self.frequency = frequency; self.createdAt = createdAt
    }
}

public enum ScheduledTaskStore {
    static let key = "lengsuo.scheduled.tasks"

    public static func load() -> [StoredTask] {
        guard let data = UserDefaults.standard.data(forKey: key) else { return [] }
        return (try? JSONDecoder().decode([StoredTask].self, from: data)) ?? []
    }

    public static func save(_ tasks: [StoredTask]) {
        if let data = try? JSONEncoder().encode(tasks) {
            UserDefaults.standard.set(data, forKey: key)
        }
    }

    @discardableResult
    public static func add(name: String, draft: ScheduleDraft) -> StoredTask {
        var tasks = load()
        let t = StoredTask(name: name, cronExpr: draft.cronExpression(),
                           frequency: draft.frequency.rawValue)
        tasks.insert(t, at: 0)
        save(tasks)
        return t
    }

    public static func remove(id: String) {
        save(load().filter { $0.id != id })
    }
}
