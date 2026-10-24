import SwiftUI

/// 输入栏增强版：附件 chips + TextField 两态 + 语音/发送钮。
/// 对齐 04§5 布局「[+] 附件 | 文本框 | 🎤 | ⏯」。
struct ChatInputBar: View {
    @Binding var text: String
    @Binding var attachments: [String]      // 附件名（真上传属后续卡）
    @Binding var phase: InputPhase
    var onSend: () -> Void = {}

    /// 占位符：默认即 04§5 原文串（复用已逐字锁死的 `ChatCopy.inputPlaceholderIdle`，同字节）。
    /// 宿主可注入 `ChatPhase.inputPlaceholder` 以保留「发消息...」发送中态文案。
    var placeholder: String = ChatCopy.inputPlaceholderIdle

    /// 附件上传的**模拟**注入点（真上传属后续卡）：返回该附件的最终名；
    /// 返回 nil ⇒ 停在 `.uploading`，演示 04§5「发送前等待上传完成」。
    var uploadSimulator: (Int) -> String? = { "附件\($0).pdf" }

    /// 上限文案提示槽（逐字 04§5；与 `.uploading` 的等待文案共用一行）。
    @State private var hint: String?

    var body: some View {
        VStack(spacing: 6) {
            if !attachments.isEmpty {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 8) {
                        ForEach(attachments, id: \.self) { a in
                            HStack(spacing: 4) {
                                Image(systemName: "paperclip").font(.caption2)
                                Text(a).font(.caption).lineLimit(1)
                                Button { attachments.removeAll { $0 == a } } label: {
                                    Image(systemName: "xmark").font(.caption2)
                                }
                            }
                            .padding(.horizontal, 8).padding(.vertical, 4)
                            .background(.quaternary, in: Capsule())
                        }
                    }.padding(.horizontal, 16)
                }
            }
            if let notice = hint ?? (phase == .uploading ? ChatInputCopy.stillUploading : nil) {
                Text(notice)
                    .font(.caption2).foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading).padding(.horizontal, 16)
            }
            HStack(spacing: 10) {
                Button {
                    if let e = InputValidator.attachmentLimitError(current: attachments.count) {
                        hint = e                // 上限文案逐字显示；真交互属后续卡
                    } else {
                        let name = uploadSimulator(attachments.count + 1)
                        attachments.append(name ?? "附件\(attachments.count + 1).pdf")
                        hint = nil
                        // 模拟器返回 nil ⇒ 上传未完成：停在 uploading，发送被 canSend 挡住
                        phase = InputPhase.afterAttachmentAdded(current: phase,
                                                               uploadStalled: name == nil,
                                                               text: text)
                    }
                } label: { Image(systemName: "plus") }
                TextField(placeholder, text: $text)
                    .font(.subheadline)
                    .padding(.vertical, 8).padding(.horizontal, 12)
                    .background(.quaternary, in: Capsule())
                    // 卡面 §2.2 给的是 `{ _, v in }` 两参式（macOS 14+）；
                    // 本包 platforms 为 macOS(.v13)，故用 13 可用的单参式（语义等价，见报告 §五-1）。
                    .onChange(of: text) { v in
                        phase = InputPhase.afterTextChange(current: phase, text: v)
                        hint = nil
                    }
                Image(systemName: phase == .recording ? "mic.fill" : "mic")
                Button {
                    if InputValidator.canSend(text: text, attachmentsUploading: phase == .uploading) {
                        onSend(); text = ""; phase = .idle
                    }
                } label: { Image(systemName: phase == .idle ? "play.fill" : "arrow.up.circle.fill") }
            }
            .padding(.horizontal, 16).padding(.vertical, 10)
        }
    }
}