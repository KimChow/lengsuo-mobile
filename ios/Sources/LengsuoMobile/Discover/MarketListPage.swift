import SwiftUI

/// 五入口统一子页：空态列表。
/// 市场域 = 技能市场（skill-market/{categories,list}）+ 专家（expert）+ 连接器（connector）+ 项目（project）
/// （android-devguide/12 册概览要点）；真实市场数据属 P2 通道层，本卡 ⛔ 假商品/假专家数据
/// ⇒ 列表恒为「暂无内容」通用空态占位。
public struct MarketListPage: View {
    let category: String
    let query: String

    public init(category: String, query: String = "") {
        self.category = category; self.query = query
    }

    public var body: some View {
        List {
            // 搜索框 query 由发现页带入；仅展示 @ 前缀，不触发检索（12 册接口属 P2）。
            if !query.isEmpty {
                Section {
                    Text("搜索：\(query)").font(.footnote).foregroundStyle(.secondary)
                }
            }
            Section {
                Text("暂无内容")
                    .font(.footnote).foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .center)
            }
        }
        .navigationTitle(category)
    }
}