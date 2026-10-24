import SwiftUI

/// 主壳：底部四 Tab（任务/定时任务/发现/我的）。
/// 对齐 android-devguide/15 §2。
/// ⭐ 四 Tab 分派骨架由**主理人一次性落盘**：每张页面卡只改自己的目录
/// （`Home/` `Scheduled/` `Discover/` `Me/`），互不触碰本文件 ⇒ 可并行派卡。
/// 接线状态：.scheduled = 实页（mb-scheduled-ios）；其余三 Tab 为骨架占位，由后续卡逐个填入。
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

    @ViewBuilder
    private func page(for tab: MainTab) -> some View {
        switch tab {
        case .tasks: TasksHomePage()
        case .scheduled: ScheduledPage()
        case .discover: DiscoverPage()
        case .me: MePage()
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