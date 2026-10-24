import SwiftUI

/// 定时任务 Tab 页：标题居中加粗 + 空态/列表两态 + FAB + 添加弹层。
/// 对齐 mb-scheduled-android 卡：仅本地内存两态，无真实调度（BGTaskScheduler）/持久化/网络。
struct ScheduledPage: View {
    @StateObject private var store = ScheduledStore()
    @State private var showAddDialog = false
    @State private var showCustomForm = false

    var body: some View {
        VStack(spacing: 0) {
            Text(LengsuoStrings.Sched.title)
                .font(.headline)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 12)

            if store.tasks.isEmpty {
                emptyState
            } else {
                taskList
            }
        }
        .overlay(alignment: .bottomTrailing) { floatingAddButton }
        .confirmationDialog(
            LengsuoStrings.Sched.fabAdd,
            isPresented: $showAddDialog,
            titleVisibility: .visible
        ) {
            Button(LengsuoStrings.Sched.sheetCustom) { showCustomForm = true }
            // 「以对话方式添加」本卡为关闭弹层占位（差异声明见报告）
            Button(LengsuoStrings.Sched.sheetConversation) {}
            Button(LengsuoStrings.Sched.sheetCancel, role: .cancel) {}
        }
        .sheet(isPresented: $showCustomForm) {
            ScheduledFormView { name, prompt, frequency in
                store.add(name: name, prompt: prompt, frequency: frequency)
            }
        }
    }

    /// 空态：吉祥物资源未入仓，暂以 SF Symbols `calendar.badge.clock` 占位（差异声明见报告）。
    private var emptyState: some View {
        VStack(spacing: 12) {
            Image(systemName: "calendar.badge.clock")
                .font(.system(size: 48))
                .foregroundStyle(.secondary)
            Text(LengsuoStrings.Sched.empty)
                .font(.subheadline)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private var taskList: some View {
        List {
            ForEach(store.tasks) { task in
                ScheduledTaskCard(task: task)
            }
            .onDelete { store.remove(atOffsets: $0) }
        }
        .listStyle(.plain)
    }

    /// 浮动按钮：56pt 圆形右下角（Android 56dp / trailing 20；底部间距按 iOS 惯例取 20，差异声明见报告）。
    private var floatingAddButton: some View {
        Button {
            showAddDialog = true
        } label: {
            Image(systemName: "plus")
                .font(.system(size: 24, weight: .semibold))
                .foregroundStyle(.white)
                .frame(width: 56, height: 56)
                .background(Color.accentColor, in: Circle())
        }
        .accessibilityLabel(Text(LengsuoStrings.Sched.fabAdd))
        .padding(.trailing, 20)
        .padding(.bottom, 20)
    }
}

/// 列表卡片：名称/内容 + 频率·有效期 + 「运行」「更多操作」（两者均为无操作占位）。
private struct ScheduledTaskCard: View {
    let task: ScheduledTask

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(task.name)
                .font(.body.weight(.semibold))
            if !task.prompt.isEmpty {
                Text(task.prompt)
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
            }
            HStack(spacing: 6) {
                Image(systemName: "clock")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Text(task.frequency.title)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Spacer()
                Text(LengsuoStrings.Sched.validityLongTerm)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            HStack(spacing: 16) {
                Button(LengsuoStrings.Sched.cardRun) {}
                    .font(.subheadline)
                Spacer()
                Button(LengsuoStrings.Sched.actionMore) {}
                    .font(.subheadline)
            }
            .buttonStyle(.borderless)
        }
        .padding(.vertical, 4)
    }
}
