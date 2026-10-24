import SwiftUI
import Combine

/// 三态偏好管理：内存 + UserDefaults 持久化；变更即时生效（不重建视图）。
public final class ThemeManager: ObservableObject {
    static let storageKey = "lengsuo.theme.preference"

    @Published public private(set) var preference: ColorSchemePreference

    public init(initial: ColorSchemePreference = .system) {
        let stored = UserDefaults.standard.string(forKey: Self.storageKey)
            .flatMap(ColorSchemePreference.init(rawValue:))
        self.preference = stored ?? initial
    }

    public func set(_ p: ColorSchemePreference) {
        preference = p
        UserDefaults.standard.set(p.rawValue, forKey: Self.storageKey)
    }
}
