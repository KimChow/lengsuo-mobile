import Foundation

/// 输入区原文串：devguide 04§5 逐字（英文原文保留，⛔ 翻译/改写）。
public enum ChatInputCopy {
    /// 附件上限 5 个（04§5）。
    public static let attachmentLimit = "Attachment limit reached. You can add up to 5."
    /// 上传与发送并发 ⇒ 发送前等待上传完成（04§5）。
    public static let stillUploading = "Attachments are still uploading. Please wait."
    /// 转发/上传计数上限 50（04§5）。
    public static let forwardLimit = "Attachment count exceeds the limit of 50."
    /// 语音录制达上限，识别文本回填输入框（04§5）。
    public static let voiceMaxLength = "Maximum voice recording length reached. Recognized text will appear in the input box."
}

/// 输入区状态机（本卡 UI 态；真录音/真上传属后续卡）。
/// idle=占位符；ready=有文本、⏯ 变发送态；uploading=等待文案；recording=🎤 高亮。
public enum InputPhase: Equatable { case idle, ready, uploading, recording }

public extension InputPhase {
    /// 文本变化时的态迁移纯函数（视图 `onChange` 调它，便于单测）。
    /// recording / uploading 不被文本事件覆盖：04§5「发送前等待上传完成」。
    static func afterTextChange(current: InputPhase, text: String) -> InputPhase {
        switch current {
        case .recording, .uploading:
            return current
        case .idle, .ready:
            return InputValidator.canSend(text: text, attachmentsUploading: false) ? .ready : .idle
        }
    }

    /// 附件事件（`+` 选附件）后的态迁移纯函数。
    /// `uploadStalled`=上传未完成 ⇒ 停在 `.uploading`（真上传属后续卡，等待文案见 04§5）。
    /// 上传完成 ⇒ 从 uploading 退出，按文本回到 ready/idle（⛔ 不得成为死态）；
    /// recording 不被附件事件打断（真录音属 10 册）。
    static func afterAttachmentAdded(current: InputPhase, uploadStalled: Bool, text: String) -> InputPhase {
        if uploadStalled { return .uploading }
        switch current {
        case .recording:
            return .recording
        case .uploading, .idle, .ready:
            return afterTextChange(current: .idle, text: text)
        }
    }
}

/// 纯函数校验器（可测）。
public enum InputValidator {
    public static let maxAttachments = 5
    public static let maxForwardCount = 50

    /// 返回 nil=允许；否则为上限文案。
    public static func attachmentLimitError(current: Int) -> String? {
        current >= maxAttachments ? ChatInputCopy.attachmentLimit : nil
    }

    /// 是否还能再加附件（04§5「附件上限 5 个」）。
    public static func canAddAttachment(current: Int) -> Bool {
        attachmentLimitError(current: current) == nil
    }

    /// 转发/上传计数上限 50：超出（> 50）⇒ 返回上限文案，否则 nil。
    public static func forwardLimitError(count: Int) -> String? {
        count > maxForwardCount ? ChatInputCopy.forwardLimit : nil
    }

    /// 文本非空 ⇒ 可发送（附件上传中不可发）。
    public static func canSend(text: String, attachmentsUploading: Bool) -> Bool {
        !attachmentsUploading && !text.trimmingCharacters(in: .whitespaces).isEmpty
    }
}