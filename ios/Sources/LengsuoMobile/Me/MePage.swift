import SwiftUI

/// 「我的」页：五段分组（android-devguide/15 §6）。
/// 子页可导航：字体大小 / 朗读设置 / 触感反馈 / 存储空间 / 消息通知设置（15§2 我的行）；
/// 身份卡/套餐卡/资产组/关于/语言行仍为占位（⛔ 假数据）。
/// ⚠️ 接线偏差（本卡声明，非本卡职责）：
/// 1. 主题三段控件仍绑定页内状态，未接 `Theme/ThemeManager`（现已合入）；
///    应由独立接线卡替换为 `theme.preference` 双向绑定。
/// 2. `Shell/MainShell.swift` 未提供 `NavigationStack`（Shell/ 为本卡禁区）⇒ 本页自持
///    一层 `NavigationStack`；若后续壳层补上导航容器，应删除本页这层以免嵌套。
public struct MePage: View {
    /// 主题三段选项（android-devguide/15 §2：「主题（系统/浅色/深色 三段选择）」）。
    /// 嵌套于 MePage，避免与 mb-ios-theme 产物的顶层类型冲突。
    enum ThemeOption: String, CaseIterable, Identifiable {
        case system
        case light
        case dark

        var title: String {
            switch self {
            case .system: "系统"
            case .light: "浅色"
            case .dark: "深色"
            }
        }

        var id: String { rawValue }
    }

    @State private var themeOption: ThemeOption = .system
    @StateObject private var settings = MeSettingsStore()

    public init() {}

    public var body: some View {
        NavigationStack {
            List {
                // 身份卡（头像/昵称/箭头）
                Section {
                    HStack(spacing: 12) {
                        Image(systemName: "person.circle").font(.system(size: 44))
                        Text("未登录").font(.headline)
                        Spacer()
                        Image(systemName: "chevron.right")
                            .font(.footnote).foregroundStyle(.secondary)
                    }.padding(.vertical, 4)
                }
                // 套餐卡（档位 + 升级按钮 + 积分行）
                Section {
                    HStack {
                        Text("体验版").font(.subheadline)
                        Spacer()
                        Button("升级套餐") {}.buttonStyle(.borderedProminent).controlSize(.small)
                    }
                    HStack {
                        Text("我的积分").font(.subheadline)
                        Spacer()
                        Text("—").foregroundStyle(.secondary)
                    }
                }
                // 通用设置组（主题/语言/字号/朗读/触感/存储/通知）
                Section {
                    Picker("主题", selection: $themeOption) {
                        ForEach(ThemeOption.allCases) { p in
                            Text(p.title).tag(p)
                        }
                    }
                    // ⛔ 语言保持静态（跟随系统），不导航
                    row("语言", value: "跟随系统")
                    NavigationLink { FontSizePage(store: settings) } label: { navRow("字体大小") }
                    NavigationLink { ReadingHapticsPage(store: settings) } label: { navRow("朗读设置") }
                    NavigationLink { ReadingHapticsPage(store: settings) } label: {
                        navRow("触感反馈", value: settings.hapticsEnabled ? "开" : "关")
                    }
                    NavigationLink { StoragePage() } label: { navRow("存储空间") }
                    NavigationLink { NotificationSettingsPage(store: settings) } label: {
                        navRow("消息通知设置")
                    }
                }
                // 资产组（记忆库/成长计划）
                Section {
                    row("记忆库")
                    row("成长计划")
                }
                // 关于
                Section {
                    row("关于 WorkBuddy")
                }
            }
            .navigationTitle(LengsuoStrings.tabMe)
        }
        // 15§5.2 字号档：环境值下发 ⇒ 档位切换即时生效、不重建页面
        .dynamicTypeSize(currentFontSize.dynamicTypeSize)
    }

    private var currentFontSize: MeFontSize {
        MeFontSize(rawValue: settings.fontSizeIndex) ?? .standard
    }

    /// 静态占位行（自绘 chevron）。
    private func row(_ title: String, value: String? = nil) -> some View {
        HStack {
            Text(title)
            Spacer()
            if let value { Text(value).foregroundStyle(.secondary) }
            Image(systemName: "chevron.right").font(.footnote).foregroundStyle(.tertiary)
        }
    }

    /// 可导航行的 label：不画 chevron，由 `NavigationLink` 自带。
    private func navRow(_ title: String, value: String? = nil) -> some View {
        HStack {
            Text(title)
            Spacer()
            if let value { Text(value).foregroundStyle(.secondary) }
        }
    }
}