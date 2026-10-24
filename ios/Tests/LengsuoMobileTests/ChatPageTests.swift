import XCTest
@testable import LengsuoMobile

final class ChatPageTests: XCTestCase {
    func testBannerVerbatim() {
        XCTAssertEqual(ChatCopy.aiBanner, "内容由 AI 生成")
    }

    func testStatusMapping() {
        // ChatPage.kt chatStatusLabel 同语义
        XCTAssertEqual(ChatPhase.sending.statusLabel, "正在思考 ⋯")
        XCTAssertEqual(ChatPhase.done.statusLabel, "已完成")
    }

    func testPlaceholderMapping() {
        // 注意半角三点 vs ⋯ 的差异：占位符是 "..."，状态行是 "⋯"
        XCTAssertEqual(ChatPhase.sending.inputPlaceholder, "发消息...")
        XCTAssertEqual(ChatPhase.done.inputPlaceholder, "发消息或按住说话...")
    }

    func testModelTierMapping() {
        XCTAssertEqual(ChatPhase.sending.modelTierLabel, "⚡ 快速 ▾")
        XCTAssertEqual(ChatPhase.done.modelTierLabel, "auto")
    }

    func testAllSeventeenCopies() {
        let all = [ChatCopy.aiBanner, ChatCopy.statusDone, ChatCopy.statusThinking,
                   ChatCopy.toolbarCopy, ChatCopy.toolbarLike, ChatCopy.toolbarDislike,
                   ChatCopy.toolbarReadAloud, ChatCopy.toolbarShare, ChatCopy.topNewChat,
                   ChatCopy.topMore, ChatCopy.subtitleMode, ChatCopy.inputPlaceholderIdle,
                   ChatCopy.inputPlaceholderSending, ChatCopy.modelTierIdle,
                   ChatCopy.modelTierSending, ChatCopy.inputVoice, ChatCopy.inputPlus]
        XCTAssertEqual(all.count, 17)
        XCTAssertTrue(all.allSatisfy { !$0.isEmpty })
    }

    /// 17 条与 android strings.xml chat_* 逐字全等（不止非空）。
    func testSeventeenCopiesMatchAndroidStringsVerbatim() {
        XCTAssertEqual(ChatCopy.aiBanner, "内容由 AI 生成")
        XCTAssertEqual(ChatCopy.statusDone, "已完成")
        XCTAssertEqual(ChatCopy.statusThinking, "正在思考 ⋯")
        XCTAssertEqual(ChatCopy.toolbarCopy, "复制")
        XCTAssertEqual(ChatCopy.toolbarLike, "赞")
        XCTAssertEqual(ChatCopy.toolbarDislike, "踩")
        XCTAssertEqual(ChatCopy.toolbarReadAloud, "朗读")
        XCTAssertEqual(ChatCopy.toolbarShare, "分享")
        XCTAssertEqual(ChatCopy.topNewChat, "新建对话")
        XCTAssertEqual(ChatCopy.topMore, "更多操作")
        XCTAssertEqual(ChatCopy.subtitleMode, "☁ 工作模式")
        XCTAssertEqual(ChatCopy.inputPlaceholderIdle, "发消息或按住说话...")
        XCTAssertEqual(ChatCopy.inputPlaceholderSending, "发消息...")
        XCTAssertEqual(ChatCopy.modelTierIdle, "auto")
        XCTAssertEqual(ChatCopy.modelTierSending, "⚡ 快速 ▾")
        XCTAssertEqual(ChatCopy.inputVoice, "语音")
        XCTAssertEqual(ChatCopy.inputPlus, "+")
    }

    /// 页面可实例化（两态各一），静态版不依赖网络。
    func testViewInstantiatesBothPhases() {
        _ = ChatPageView(phase: .sending)
        _ = ChatPageView(phase: .done)
        let custom = ChatPageView(phase: .done, userMessage: "u", aiMessage: "a")
        XCTAssertEqual(custom.userMessage, "u")
        XCTAssertEqual(custom.aiMessage, "a")
    }
}