import SwiftUI

/// 任务 Tab 页（骨架由主理人一次性落盘）。
/// 实页内容由 `mb-ios-tasks-home` 卡填入；骨架阶段只渲染占位。
public struct TasksHomePage: View {
    public init() {}

    public var body: some View {
        PlaceholderPage(title: LengsuoStrings.tabTasks)
    }
}