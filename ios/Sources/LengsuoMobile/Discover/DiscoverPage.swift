import SwiftUI

/// 发现 Tab：搜索 + 五入口（资料库/专家/技能/连接器/项目）。
/// 对齐 android-devguide/15 §2「搜索 + 资料库/专家/技能/连接器/项目」；子页与数据属后续卡。
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
        List {
            Section {
                HStack {
                    Image(systemName: "magnifyingglass").foregroundStyle(.secondary)
                    TextField("搜索", text: $query)
                }
            }
            Section {
                ForEach(Self.entries, id: \.title) { e in
                    HStack {
                        Image(systemName: e.icon).frame(width: 24)
                        Text(e.title)
                        Spacer()
                        Image(systemName: "chevron.right")
                            .font(.footnote).foregroundStyle(.tertiary)
                    }
                }
            }
        }
    }
}
