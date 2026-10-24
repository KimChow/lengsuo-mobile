import SwiftUI

#if canImport(UIKit)
import UIKit
#elseif canImport(AppKit)
import AppKit
#endif

/// 对话页静态版：顶栏/提示条/消息区/工具条5键(DONE)/输入区两态。
/// ⛔ 本卡不发送、不流式、不录音——phase 由调用方给定。
public struct ChatPageView: View {
    let phase: ChatPhase
    let userMessage: String
    let aiMessage: String

    public init(phase: ChatPhase = .done,
                userMessage: String = "帮我整理这周的工作",
                aiMessage: String = "好的，以下是本周工作整理…") {
        self.phase = phase; self.userMessage = userMessage; self.aiMessage = aiMessage
    }

    public var body: some View {
        VStack(spacing: 0) {
            topBar
            Text(ChatCopy.aiBanner)
                .font(.caption2).foregroundStyle(.secondary)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 4).background(.bar)
            ScrollView {
                VStack(alignment: .trailing, spacing: 12) {
                    Bubble(text: userMessage, isUser: true)
                    VStack(alignment: .leading, spacing: 8) {
                        Text(aiMessage).font(.body)
                        if phase == .done { toolbar }
                        if phase == .sending {
                            Text(phase.statusLabel)
                                .font(.footnote).foregroundStyle(.secondary)
                        }
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                }.padding(16)
            }
            inputBar
        }
    }

    private var topBar: some View {
        HStack {
            Button {} label: { Label(ChatCopy.topNewChat, systemImage: "plus.bubble").labelStyle(.titleAndIcon).font(.subheadline) }
            Spacer()
            VStack(spacing: 0) {
                Text("WorkBuddy").font(.headline)
                Text(ChatCopy.subtitleMode).font(.caption2).foregroundStyle(.secondary)
            }
            Spacer()
            Button {} label: { Image(systemName: "ellipsis") }
        }.padding(.horizontal, 16).padding(.vertical, 8)
    }

    private var toolbar: some View {
        HStack(spacing: 20) {
            ForEach([(ChatCopy.toolbarCopy, "doc.on.doc"),
                     (ChatCopy.toolbarLike, "hand.thumbsup"),
                     (ChatCopy.toolbarDislike, "hand.thumbsdown"),
                     (ChatCopy.toolbarReadAloud, "speaker.wave.2"),
                     (ChatCopy.toolbarShare, "square.and.arrow.up")], id: \.0) { item in
                VStack(spacing: 2) {
                    Image(systemName: item.1).font(.footnote)
                    Text(item.0).font(.caption2)
                }.foregroundStyle(.secondary)
            }
        }
    }

    private var inputBar: some View {
        HStack(spacing: 10) {
            Text(ChatCopy.inputPlus).font(.title3)
            Text(phase.inputPlaceholder)
                .font(.subheadline).foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.vertical, 8).padding(.horizontal, 12)
                .background(.quaternary, in: Capsule())
            Text(ChatCopy.inputVoice).font(.footnote)
        }
        .padding(.horizontal, 16).padding(.vertical, 10).background(.bar)
    }
}

/// AI 正文气泡底色：卡面给的是 UIColor 专有的 secondarySystemBackground，
/// 宿主 swift build 跑在 macOS 上，故加平台守卫（AppKit 侧退化为 windowBackgroundColor）。
/// ⛔ 其余视图代码逐字照卡。
private var aiBubbleBackground: Color {
    #if canImport(UIKit)
    return Color(.secondarySystemBackground)
    #else
    return Color(nsColor: .windowBackgroundColor)
    #endif
}

struct Bubble: View {
    let text: String
    let isUser: Bool
    var body: some View {
        Text(text)
            .font(.body)
            .padding(12)
            .background(isUser ? Color.accentColor.opacity(0.18) : aiBubbleBackground,
                        in: RoundedRectangle(cornerRadius: 14))
            .frame(maxWidth: 280, alignment: isUser ? .trailing : .leading)
    }
}