import SwiftUI

/// 「我的」页：五段分组（android-devguide/15 §6）。
/// 除主题三段控件外全部为占位行；⛔ 假数据。
/// ⚠️ 接线偏差：前置卡 mb-ios-theme（ThemeManager）未合入本分支，
/// 主题三段控件暂绑定页内状态（真实交互、不落全局生效）；
/// ThemeManager 合入后应由接线卡替换为 theme.preference 双向绑定。
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

    public init() {}

    public var body: some View {
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
                row("语言", value: "跟随系统")
                row("字体大小")
                row("朗读设置")
                row("触感反馈", value: "开")
                row("存储空间")
                row("消息通知设置")
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
    }

    private func row(_ title: String, value: String? = nil) -> some View {
        HStack {
            Text(title)
            Spacer()
            if let value { Text(value).foregroundStyle(.secondary) }
            Image(systemName: "chevron.right").font(.footnote).foregroundStyle(.tertiary)
        }
    }
}
