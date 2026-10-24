import SwiftUI

/// 会话列表：本地/云端双 Section。数据 = 写死样例（真实接口属 P2）。
public struct ConversationListView: View {
    let conversations: [Conversation]

    public init(conversations: [Conversation] = ConversationListView.sample) {
        self.conversations = conversations
    }

    // ⚠️ 卡面 §2.2 为 `static let sample`（internal）；因 `public init` 默认实参
    // 需可见性 ≥ public，真跑报「static property 'sample' is internal and cannot be
    // referenced from a default argument value」⇒ 最小修正为 public（语义不变）。
    public static let sample: [Conversation] = [
        Conversation(id: "/chats/local_draft_1", title: "本地草稿：周报"),
        Conversation(id: "/chats/local_bga_9", title: "后台任务：数据整理"),
        Conversation(id: "/chats/cloud_42", title: "云端：旅行计划"),
    ]

    public var body: some View {
        let g = groupConversations(conversations)
        List {
            Section("本地") {
                ForEach(g.local) { c in
                    HStack {
                        Text(c.title).lineLimit(1)
                        if c.track == .bga {
                            Text("后台").font(.caption2).foregroundStyle(.secondary)
                                .padding(.horizontal, 6).padding(.vertical, 2)
                                .background(.quaternary, in: Capsule())
                        }
                    }
                }
            }
            Section("云端") {
                ForEach(g.cloud) { c in
                    Text(c.title).lineLimit(1)
                }
            }
        }
    }
}