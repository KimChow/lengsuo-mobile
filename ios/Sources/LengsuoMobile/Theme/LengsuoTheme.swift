import SwiftUI

#if canImport(UIKit)
import UIKit
#elseif canImport(AppKit)
import AppKit
#endif

/// 主题三态：对齐 android-devguide/15 §5.2「系统/浅色/深色 三段选择」。
public enum ColorSchemePreference: String, CaseIterable, Identifiable {
    case system
    case light
    case dark

    public var id: String { rawValue }

    /// 我的页三段控件文案（逐字）。
    public var title: String {
        switch self {
        case .system: return "系统"
        case .light: return "浅色"
        case .dark: return "深色"
        }
    }

    public var scheme: ColorScheme? {
        switch self {
        case .system: return nil
        case .light: return .light
        case .dark: return .dark
        }
    }
}

/// 语义色板：视图层唯一取色入口，⛔ 视图不得自造颜色。
/// 槽位对齐 android 线 LengsuoColors.kt；色值逐项取自参考 APK 实测
/// （res/values + res/values-night 双套），⛔ 不可自造。
/// 实现方式：UIColor/NSColor 动态提供者（Package.swift 未声明资源，不走 assets catalog），
/// 浅/深随 trait 即时解析，主题切换不重建视图。
public enum LengsuoTheme {
    /// 页面背景（android: pageBackground）。
    public static let background = dynamicColor(light: 0xFFFFFF, dark: 0x0F1011)

    /// 卡片/弹层表面（android: dialogBackground）。
    public static let surface = dynamicColor(light: 0xFFFFFF, dark: 0x333333)

    /// 主要前景（android: primary）。
    public static let primary = dynamicColor(light: 0x1A1A1A, dark: 0xFFFFFF)

    /// 次要/弱化前景（android: mainThirdText）。
    public static let secondary = dynamicColor(light: 0x999999, dark: 0x888888)

    /// 正文主文本（android: mainText）。
    public static let textPrimary = dynamicColor(light: 0x333333, dark: 0xFFFFFF)

    /// 次级文本（android: mainSecondText）。
    public static let textSecondary = dynamicColor(light: 0x666666, dark: 0xABABAB)

    /// 品牌强调（android: BrandMain，实测仅亮色定义 ⇒ 明暗共用）。
    public static let accent = dynamicColor(light: 0x005ED9, dark: 0x005ED9)
}

// MARK: - 动态色（iOS 17 / macOS 13 双宿主）

private func dynamicColor(light: UInt32, dark: UInt32) -> Color {
#if canImport(UIKit)
    Color(UIColor { trait in
        trait.userInterfaceStyle == .dark ? uiColor(dark) : uiColor(light)
    })
#elseif canImport(AppKit)
    Color(NSColor(name: nil) { appearance in
        appearance.bestMatch(from: [.darkAqua, .vibrantDark]) != nil ? nsColor(dark) : nsColor(light)
    })
#endif
}

#if canImport(UIKit)
private func uiColor(_ rgb: UInt32) -> UIColor {
    UIColor(red: CGFloat((rgb >> 16) & 0xFF) / 255,
            green: CGFloat((rgb >> 8) & 0xFF) / 255,
            blue: CGFloat(rgb & 0xFF) / 255,
            alpha: 1)
}
#elseif canImport(AppKit)
private func nsColor(_ rgb: UInt32) -> NSColor {
    NSColor(red: CGFloat((rgb >> 16) & 0xFF) / 255,
            green: CGFloat((rgb >> 8) & 0xFF) / 255,
            blue: CGFloat(rgb & 0xFF) / 255,
            alpha: 1)
}
#endif
