import SwiftUI

/// 主壳：底部四 Tab（任务/定时任务/发现/我的）。
/// 对齐 android-devguide/15 §2；除 `.scheduled`（ScheduledPage，mb-scheduled-ios）外均为占位页，实页由后续卡逐个接入。
public struct MainShell: View {
    @State private var selection: MainTab = .tasks

    public init() {}

    public var body: some View {
        TabView(selection: $selection) {
            ForEach(MainTab.allCases) { tab in
                page(for: tab)
                    .tabItem { Label(tab.title, systemImage: tab.systemImage) }
                    .tag(tab)
            }
        }
    }

    /// 本卡只接 `.scheduled` 实页，其余 Tab 仍为占位。
    @ViewBuilder
    private func page(for tab: MainTab) -> some View {
        if tab == .scheduled {
            ScheduledPage()
        } else {
            PlaceholderPage(title: tab.title)
        }
    }
}

/// 占位页：居中「<Tab 名> · 敬请期待」。禁假数据。
struct PlaceholderPage: View {
    let title: String

    var body: some View {
        VStack(spacing: 8) {
            Text(title)
                .font(.headline)
            Text(LengsuoStrings.comingSoon)
                .font(.subheadline)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
