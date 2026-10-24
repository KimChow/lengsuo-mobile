import Foundation

/// 我的页子页偏好：统一 UserDefaults 持久化（key 前缀 lengsuo.me.*）。
public final class MeSettingsStore: ObservableObject {
    static let prefix = "lengsuo.me."

    @Published public var fontSizeIndex: Int { didSet { save("fontSizeIndex", fontSizeIndex) } }
    @Published public var readingEnabled: Bool { didSet { save("readingEnabled", readingEnabled) } }
    @Published public var hapticsEnabled: Bool { didSet { save("hapticsEnabled", hapticsEnabled) } }
    @Published public var notifyTaskResult: Bool { didSet { save("notifyTaskResult", notifyTaskResult) } }
    @Published public var notifyScheduledResult: Bool { didSet { save("notifyScheduledResult", notifyScheduledResult) } }
    @Published public var notifyPendingApproval: Bool { didSet { save("notifyPendingApproval", notifyPendingApproval) } }
    @Published public var notifyGlobal: Bool { didSet { save("notifyGlobal", notifyGlobal) } }

    public init() {
        let d = UserDefaults.standard
        fontSizeIndex = d.object(forKey: Self.prefix + "fontSizeIndex") as? Int ?? 1
        readingEnabled = d.object(forKey: Self.prefix + "readingEnabled") as? Bool ?? false
        hapticsEnabled = d.object(forKey: Self.prefix + "hapticsEnabled") as? Bool ?? true   // 15§2「触感反馈（开）」
        notifyTaskResult = d.object(forKey: Self.prefix + "notifyTaskResult") as? Bool ?? true
        notifyScheduledResult = d.object(forKey: Self.prefix + "notifyScheduledResult") as? Bool ?? true
        notifyPendingApproval = d.object(forKey: Self.prefix + "notifyPendingApproval") as? Bool ?? true
        notifyGlobal = d.object(forKey: Self.prefix + "notifyGlobal") as? Bool ?? true
    }

    private func save<T>(_ k: String, _ v: T) { UserDefaults.standard.set(v, forKey: Self.prefix + k) }
}

/// 字号五档 → SwiftUI dynamicTypeSize（15§5.2 字号档，切换不重建页面）。
public enum MeFontSize: Int, CaseIterable, Identifiable {
    case small = 0, standard, large, xLarge, xxl

    public var id: Int { rawValue }
    public var title: String { ["小", "标准", "大", "特大", "最大"][rawValue] }
}