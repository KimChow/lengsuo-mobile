import SwiftUI

/// 我的 Tab 页（骨架由主理人一次性落盘）。
/// 实页内容由 `mb-ios-me-page` 卡填入。
public struct MePage: View {
    public init() {}

    public var body: some View {
        PlaceholderPage(title: LengsuoStrings.tabMe)
    }
}