import SwiftUI

/// 发现 Tab：搜索 + 五入口（资料库/专家/技能/连接器/项目）。
/// 对齐 android-devguide/15 §2「搜索 + 资料库/专家/技能/连接器/项目」；
/// 五入口全部接入 `MarketListPage` 子页（mb-ios-discover-market）；市场数据接口属 P2。
/// ⚠️ 接线偏差（本卡声明，非本卡职责）：`Shell/MainShell.swift` 未提供 `NavigationStack`
/// （`Shell/` 为本卡禁区）⇒ 本页自持一层 `NavigationStack`，与 `Me/MePage.swift` 同处理；
/// 若后续壳层补上导航容器，应删除本页这层以免嵌套。
public struct DiscoverPage: View {
    @State private var query = ""

    static let entries: [(title: String, icon: String)] = [
        ("资料库", "folder"),
        ("专家", "person.2"),
        ("技能", "sparkles"),
        ("连接器", "cable.connector"),
        ("项目", "square.stack.3d.up"),
    ]

    public init() {}

    public var body: some View {
        NavigationStack {
            List {
                Section {
                    HStack {
                        Image(systemName: "magnifyingglass").foregroundStyle(.secondary)
                        TextField("搜索", text: $query)
                    }
                }
                Section {
                    ForEach(Self.entries, id: \.title) { e in
                        // 整行包进 NavigationLink：`List` 行内自带 chevron ⇒ 不再自绘，避免双箭头。
                        NavigationLink {
                            MarketListPage(category: e.title, query: query)
                        } label: {
                            HStack {
                                Image(systemName: e.icon).frame(width: 24)
                                Text(e.title)
                                Spacer()
                            }
                        }
                    }
                }
            }
            .navigationTitle(LengsuoStrings.tabDiscover)
        }
    }
}