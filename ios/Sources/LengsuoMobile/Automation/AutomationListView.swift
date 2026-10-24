import SwiftUI

/// 自动化任务总览（/automation-task-manager 的 iOS 形态）。
/// 数据源 = ScheduledStore 本地仓；网络版属 P2 通道层。
public struct AutomationListView: View {
    @State private var tasks: [StoredTask] = ScheduledTaskStore.load()

    public init() {}

    /// 项目维度空态：文案取自 `AutomationCopy.emptyProject`（devguide 11 §3/§8 原文逐字）。
    ///
    /// ⚠️ 接线偏差：Package.swift 平台下限为 macOS 13，而 `ContentUnavailableView`
    /// 仅 macOS 14+/iOS 17+ 可用（Package.swift 属共享区，本卡不得改动）。
    /// 故此处按可用性分支：新系统用 `ContentUnavailableView`，旧系统用等价 VStack，
    /// 两者渲染同一逐字文案。
    @ViewBuilder private var emptyState: some View {
        if #available(iOS 17.0, macOS 14.0, *) {
            ContentUnavailableView(
                "Automation",
                systemImage: "clock.badge.questionmark",
                description: Text(AutomationCopy.emptyProject)
            )
        } else {
            VStack(spacing: 8) {
                Image(systemName: "clock.badge.questionmark")
                    .font(.largeTitle).foregroundStyle(.secondary)
                Text("Automation").font(.headline)
                Text(AutomationCopy.emptyProject)
                    .font(.subheadline).foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
            }
            .padding()
        }
    }

    public var body: some View {
        Group {
            if tasks.isEmpty {
                emptyState
            } else {
                List(tasks) { t in
                    NavigationLink {
                        AutomationDetailView(task: t)
                    } label: {
                        VStack(alignment: .leading, spacing: 4) {
                            Text(t.name).font(.body)
                            Text(t.cronExpr).font(.caption).foregroundStyle(.secondary)
                        }
                    }
                }
            }
        }
        .navigationTitle("Automation")
        .onAppear { tasks = ScheduledTaskStore.load() }
    }
}

/// 详情页：状态/时间/产物 三元素（§8）。
public struct AutomationDetailView: View {
    let task: AutomationTask

    public init(task: StoredTask) {
        self.task = AutomationTask.from(task)
    }

    public var body: some View {
        List {
            LabeledContent("Status", value: task.status)
            LabeledContent("Schedule", value: task.scheduleText)
            LabeledContent("Artifact", value: task.artifact)
        }
        .navigationTitle(task.name)
    }
}