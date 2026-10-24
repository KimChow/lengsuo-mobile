import Foundation

/// 对话页文案：逐字取自本仓 android strings.xml chat_*，⛔ 不得改动。
public enum ChatCopy {
    public static let aiBanner = "内容由 AI 生成"
    public static let statusDone = "已完成"
    public static let statusThinking = "正在思考 ⋯"
    public static let toolbarCopy = "复制"
    public static let toolbarLike = "赞"
    public static let toolbarDislike = "踩"
    public static let toolbarReadAloud = "朗读"
    public static let toolbarShare = "分享"
    public static let topNewChat = "新建对话"
    public static let topMore = "更多操作"
    public static let subtitleMode = "☁ 工作模式"
    public static let inputPlaceholderIdle = "发消息或按住说话..."
    public static let inputPlaceholderSending = "发消息..."
    public static let modelTierIdle = "auto"
    public static let modelTierSending = "⚡ 快速 ▾"
    public static let inputVoice = "语音"
    public static let inputPlus = "+"
}

/// 两态状态机（ChatPage.kt ChatPhase 同语义）。
public enum ChatPhase { case sending, done }

public extension ChatPhase {
    var statusLabel: String {
        self == .sending ? ChatCopy.statusThinking : ChatCopy.statusDone
    }
    var modelTierLabel: String {
        self == .sending ? ChatCopy.modelTierSending : ChatCopy.modelTierIdle
    }
    var inputPlaceholder: String {
        self == .sending ? ChatCopy.inputPlaceholderSending : ChatCopy.inputPlaceholderIdle
    }
}