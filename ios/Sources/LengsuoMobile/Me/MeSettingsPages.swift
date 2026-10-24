import SwiftUI

// MARK: - 字号五档 → dynamicTypeSize 映射（15§5.2）

/// 档位 → SwiftUI `DynamicTypeSize`。映射单调递增且以系统默认 `.large` 为「标准」档；
/// 由 `MePage` 施加在 `NavigationStack` 上 ⇒ 切换档位即时生效、不重建页面（15§5.2）。
extension MeFontSize {
    public var dynamicTypeSize: DynamicTypeSize {
        switch self {
        case .small: .small
        case .standard: .large
        case .large: .xLarge
        case .xLarge: .xxLarge
        case .xxl: .xxxLarge
        }
    }
}

// MARK: - 字体大小

/// 字号子页：五档 Picker，切换即时生效（父级 `MePage` 的 `.dynamicTypeSize` 环境值）。
/// `Form` 承载 `.inline` Picker 以获得正确的分组渲染（任务书 §2.2 片段为骨架示意）。
public struct FontSizePage: View {
    @ObservedObject var store: MeSettingsStore
    public init(store: MeSettingsStore) { self.store = store }
    public var body: some View {
        Form {
            Picker("字体大小", selection: $store.fontSizeIndex) {
                ForEach(MeFontSize.allCases) { f in Text(f.title).tag(f.rawValue) }
            }
            .pickerStyle(.inline)
        }
        .navigationTitle("字体大小")
    }
}

// MARK: - 朗读与触感

/// 朗读/触感 子页：两个 Toggle（15§2 入口「朗读设置」「触感反馈（开）」）。
public struct ReadingHapticsPage: View {
    @ObservedObject var store: MeSettingsStore
    public init(store: MeSettingsStore) { self.store = store }
    public var body: some View {
        Form {
            Toggle("朗读设置", isOn: $store.readingEnabled)
            Toggle("触感反馈", isOn: $store.hapticsEnabled)
        }
        .navigationTitle("朗读与触感")
    }
}

// MARK: - 存储空间

/// 存储空间子页：占位「计算中…」，⛔ 假数字（不落任何具体容量）。
/// ⚠️ 真实占用统计属 P2；本卡不接文件系统统计，恒为占位文案。
public struct StoragePage: View {
    public init() {}
    public var body: some View {
        List { LabeledContent("已用空间", value: "计算中…") }
            .navigationTitle("存储空间")
    }
}

// MARK: - 消息通知设置

/// 通知子页：11§7 四粒度（任务结果 / 定时任务结果 / 待回复待审批 / 退后台总开关）。
/// ⚠️ 本卡只做本地偏好 UI + 持久化；R11-3 的服务端下发过滤属 P2。
public struct NotificationSettingsPage: View {
    @ObservedObject var store: MeSettingsStore
    public init(store: MeSettingsStore) { self.store = store }
    public var body: some View {
        Form {
            Toggle("任务结果通知", isOn: $store.notifyTaskResult)
            Toggle("定时任务结果", isOn: $store.notifyScheduledResult)
            Toggle("待回复/待审批提醒", isOn: $store.notifyPendingApproval)
            Toggle("退后台后接收全部通知", isOn: $store.notifyGlobal)
        }
        .navigationTitle("消息通知设置")
    }
}