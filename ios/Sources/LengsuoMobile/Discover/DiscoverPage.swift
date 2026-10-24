import SwiftUI

/// 发现 Tab 页（骨架由主理人一次性落盘）。
/// 实页内容由 `mb-ios-discover` 卡填入。
public struct DiscoverPage: View {
    public init() {}

    public var body: some View {
        PlaceholderPage(title: LengsuoStrings.tabDiscover)
    }
}