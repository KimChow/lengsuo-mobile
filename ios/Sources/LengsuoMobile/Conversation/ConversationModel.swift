import Foundation

/// 会话轨道：04§2.2 双轨前缀（local_ / cloud_ / local_bga_），逐字对齐。
public enum ConvTrack: String, CaseIterable {
    case local
    case cloud
    case bga   // local_bga_：后台 Agent 会话

    /// 前缀字面量（04§2.2 原文）。
    public var prefix: String {
        switch self {
        case .local: return "/chats/local_"
        case .cloud: return "/chats/cloud_"
        case .bga: return "/chats/local_bga_"
        }
    }
}

public struct Conversation: Identifiable, Equatable {
    public var id: String          // 会话 id（含前缀）
    public var title: String

    public init(id: String, title: String) {
        self.id = id; self.title = title
    }

    /// 按前缀推断轨道。
    public var track: ConvTrack {
        if id.hasPrefix(ConvTrack.bga.prefix) { return .bga }
        if id.hasPrefix(ConvTrack.cloud.prefix) { return .cloud }
        return .local
    }
}

/// 分组纯函数：本地组（含 bga）/ 云端组。
public func groupConversations(_ items: [Conversation])
    -> (local: [Conversation], cloud: [Conversation]) {
    (
        local: items.filter { $0.track == .local || $0.track == .bga },
        cloud: items.filter { $0.track == .cloud }
    )
}