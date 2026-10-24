import SwiftUI

/// 纯函数：选中态谓词（DrawerSidebar.kt isSelected 同语义）。
public func drawerIsSelected(index: Int, selectedIndex: Int?) -> Bool {
    selectedIndex == index
}

/// 纯函数：空态谓词（列表空 ⇒ 只渲染空态文案）。
public func drawerIsEmpty(_ conversations: [String]) -> Bool {
    conversations.isEmpty
}

/// 抽屉侧栏：☁ 云端 ▾ / 新建任务 / 空态 或 任务 ▾ + 对话列表。
/// 分组行点击无操作（对齐 Android 卡「不做真实切换」声明）。
public struct DrawerView: View {
    let conversations: [String]
    let selectedIndex: Int?
    let onDismiss: () -> Void
    let onSelectConversation: (Int) -> Void
    let onNewTask: () -> Void

    public init(conversations: [String] = DrawerCopy.sampleConversations,
                selectedIndex: Int? = nil,
                onDismiss: @escaping () -> Void = {},
                onSelectConversation: @escaping (Int) -> Void = { _ in },
                onNewTask: @escaping () -> Void = {}) {
        self.conversations = conversations
        self.selectedIndex = selectedIndex
        self.onDismiss = onDismiss
        self.onSelectConversation = onSelectConversation
        self.onNewTask = onNewTask
    }

    public var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text(DrawerCopy.cloudGroup)
                .font(.subheadline).foregroundStyle(.secondary)
                .padding(16).frame(maxWidth: .infinity, alignment: .leading)
                .contentShape(Rectangle()).onTapGesture {}   // 无操作（语义对齐）
            Button {
                onNewTask()
                onDismiss()
            } label: {
                Label(DrawerCopy.newTask, systemImage: "plus.circle")
                    .font(.subheadline).frame(maxWidth: .infinity, alignment: .leading)
                    .padding(12)
                    .background(.quaternary, in: Capsule())
            }
            .padding(.horizontal, 16).padding(.bottom, 8)

            if drawerIsEmpty(conversations) {
                Text(DrawerCopy.emptyCloud)
                    .font(.footnote).foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding(.top, 32)
            } else {
                Text(DrawerCopy.taskGroup)
                    .font(.footnote).foregroundStyle(.secondary)
                    .padding(.horizontal, 16).padding(.vertical, 8)
                List(0..<conversations.count, id: \.self) { i in
                    Text(conversations[i]).lineLimit(1)
                        .listRowBackground(
                            drawerIsSelected(index: i, selectedIndex: selectedIndex)
                                ? Color.gray.opacity(0.15) : Color.clear
                        )
                        .onTapGesture { onSelectConversation(i) }
                }
                .listStyle(.plain)
            }
            Spacer(minLength: 0)
        }
        .background(drawerBackground)
        .frame(width: 340)
    }
}

/// 抽屉背景：iOS = systemBackground，macOS 宿主（SwiftPM 双平台）取 windowBackgroundColor。
/// 与 Theme/LengsuoTheme.swift 的 `#if canImport(UIKit)` 惯例一致。
private var drawerBackground: Color {
#if canImport(UIKit)
    Color(uiColor: .systemBackground)
#elseif canImport(AppKit)
    Color(nsColor: .windowBackgroundColor)
#endif
}