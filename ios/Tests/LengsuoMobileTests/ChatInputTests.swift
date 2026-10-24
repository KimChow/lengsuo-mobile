import XCTest
import SwiftUI
@testable import LengsuoMobile

final class ChatInputTests: XCTestCase {
    // MARK: 卡面 §2.3 三条

    func testAttachmentLimitVerbatim() {
        XCTAssertEqual(InputValidator.attachmentLimitError(current: 5),
                       "Attachment limit reached. You can add up to 5.")
        XCTAssertNil(InputValidator.attachmentLimitError(current: 4))
    }

    func testCopyVerbatim() {
        XCTAssertEqual(ChatInputCopy.stillUploading, "Attachments are still uploading. Please wait.")
        XCTAssertEqual(ChatInputCopy.voiceMaxLength,
                       "Maximum voice recording length reached. Recognized text will appear in the input box.")
    }

    func testCanSend() {
        XCTAssertFalse(InputValidator.canSend(text: "  ", attachmentsUploading: false))
        XCTAssertTrue(InputValidator.canSend(text: "hi", attachmentsUploading: false))
        XCTAssertFalse(InputValidator.canSend(text: "hi", attachmentsUploading: true))
    }

    // MARK: 04§5 原文串逐字（英文原文保留，⛔ 翻译）

    /// 四条 04§5 原文串一字不差（改一个字即红）。
    func testAllFourCopiesVerbatim() {
        XCTAssertEqual(ChatInputCopy.attachmentLimit, "Attachment limit reached. You can add up to 5.")
        XCTAssertEqual(ChatInputCopy.stillUploading, "Attachments are still uploading. Please wait.")
        XCTAssertEqual(ChatInputCopy.forwardLimit, "Attachment count exceeds the limit of 50.")
        XCTAssertEqual(ChatInputCopy.voiceMaxLength,
                       "Maximum voice recording length reached. Recognized text will appear in the input box.")
        let all = [ChatInputCopy.attachmentLimit, ChatInputCopy.stillUploading,
                   ChatInputCopy.forwardLimit, ChatInputCopy.voiceMaxLength]
        XCTAssertEqual(all.count, 4)
        XCTAssertTrue(all.allSatisfy { !$0.isEmpty })
    }

    /// 三条英文原文保留英文（未翻译成中文）。
    func testEnglishOriginalsNotTranslated() {
        let english = [ChatInputCopy.attachmentLimit, ChatInputCopy.stillUploading,
                       ChatInputCopy.forwardLimit, ChatInputCopy.voiceMaxLength]
        XCTAssertTrue(english.allSatisfy { $0.hasPrefix("Attachment") || $0.hasPrefix("Maximum") })
        XCTAssertTrue(english.allSatisfy { $0.allSatisfy { $0.isASCII } })
    }

    /// 附件上限边界：4 允许 / 5 触发 / 6 也触发；`>` vs `>=` 锁死在 5。
    func testAttachmentLimitBoundary() {
        XCTAssertNil(InputValidator.attachmentLimitError(current: 0))
        XCTAssertNil(InputValidator.attachmentLimitError(current: 4))
        XCTAssertNotNil(InputValidator.attachmentLimitError(current: 5))
        XCTAssertNotNil(InputValidator.attachmentLimitError(current: 6))
        XCTAssertEqual(InputValidator.maxAttachments, 5)
    }

    func testCanAddAttachment() {
        XCTAssertTrue(InputValidator.canAddAttachment(current: 4))
        XCTAssertFalse(InputValidator.canAddAttachment(current: 5))
    }

    /// 转发/上传计数上限 50：文案原文串 "exceeds the limit" ⇒ 严格大于才触发。
    func testForwardLimitBoundary() {
        XCTAssertNil(InputValidator.forwardLimitError(count: 49))
        XCTAssertNil(InputValidator.forwardLimitError(count: 50))
        XCTAssertEqual(InputValidator.forwardLimitError(count: 51),
                       "Attachment count exceeds the limit of 50.")
        XCTAssertEqual(InputValidator.maxForwardCount, 50)
    }

    // MARK: 输入状态机 InputPhase

    func testPhaseIsFourStateAndEquatable() {
        let all: [InputPhase] = [.idle, .ready, .uploading, .recording]
        XCTAssertEqual(all.count, 4)
        XCTAssertEqual(InputPhase.idle, .idle)
        XCTAssertNotEqual(InputPhase.idle, .ready)
        XCTAssertNotEqual(InputPhase.uploading, .recording)
    }

    /// idle ⇄ ready 两态迁移：有文本 ⇒ ready；清空 ⇒ idle。
    func testAfterTextChangeIdleReady() {
        XCTAssertEqual(InputPhase.afterTextChange(current: .idle, text: "hi"), .ready)
        XCTAssertEqual(InputPhase.afterTextChange(current: .ready, text: "hi"), .ready)
        XCTAssertEqual(InputPhase.afterTextChange(current: .ready, text: ""), .idle)
        XCTAssertEqual(InputPhase.afterTextChange(current: .idle, text: "   "), .idle)
    }

    /// recording / uploading 不被文本事件覆盖（04§5「发送前等待上传完成」）。
    func testAfterTextChangeDoesNotClobberRecordingOrUploading() {
        XCTAssertEqual(InputPhase.afterTextChange(current: .recording, text: "hi"), .recording)
        XCTAssertEqual(InputPhase.afterTextChange(current: .uploading, text: "hi"), .uploading)
        XCTAssertEqual(InputPhase.afterTextChange(current: .uploading, text: ""), .uploading)
    }

    /// 上传中不可发送，即使有文本（04§5 并发等待文案的语义）。
    func testUploadingBlocksSend() {
        XCTAssertFalse(InputValidator.canSend(text: "hi", attachmentsUploading: true))
        XCTAssertTrue(InputValidator.canSend(text: "hi", attachmentsUploading: false))
    }

    /// `+` 选附件后的态迁移：上传未完成 ⇒ uploading（等待文案）；完成 ⇒ 按文本 ready/idle。
    func testAfterAttachmentAdded() {
        XCTAssertEqual(InputPhase.afterAttachmentAdded(current: .idle, uploadStalled: true, text: ""), .uploading)
        XCTAssertEqual(InputPhase.afterAttachmentAdded(current: .idle, uploadStalled: false, text: ""), .idle)
        XCTAssertEqual(InputPhase.afterAttachmentAdded(current: .idle, uploadStalled: false, text: "hi"), .ready)
        // recording 不被附件事件打断（真录音属 10 册）
        XCTAssertEqual(InputPhase.afterAttachmentAdded(current: .recording, uploadStalled: false, text: "hi"), .recording)
    }

    /// uploading 必须能退出（⛔ 不得成为死态）：上传完成 ⇒ 回 ready/idle。
    func testUploadingIsNotATerminalState() {
        XCTAssertEqual(InputPhase.afterAttachmentAdded(current: .uploading, uploadStalled: false, text: "hi"), .ready)
        XCTAssertEqual(InputPhase.afterAttachmentAdded(current: .uploading, uploadStalled: false, text: ""), .idle)
        // 仍在上传 ⇒ 留在 uploading（发送继续被挡）
        XCTAssertEqual(InputPhase.afterAttachmentAdded(current: .uploading, uploadStalled: true, text: "hi"), .uploading)
    }

    // MARK: 视图（只做可实例化 + 默认值，⛔ 不声称真机渲染）

    /// 占位符 = 04§5 原文串「发消息或按住说话...」，且与 ChatCopy 同字节。
    func testInputBarDefaultPlaceholderVerbatim() {
        let bar = ChatInputBar(text: .constant(""), attachments: .constant([]), phase: .constant(.idle))
        XCTAssertEqual(bar.placeholder, "发消息或按住说话...")
        XCTAssertEqual(bar.placeholder, ChatCopy.inputPlaceholderIdle)
    }

    /// 默认模拟上传器：附件名 = 「附件N.pdf」（卡面 §2.2 同形）。
    func testInputBarDefaultUploadSimulator() {
        let bar = ChatInputBar(text: .constant(""), attachments: .constant([]), phase: .constant(.idle))
        XCTAssertEqual(bar.uploadSimulator(1), "附件1.pdf")
        XCTAssertEqual(bar.uploadSimulator(5), "附件5.pdf")
    }

    /// 注入「返回 nil」的模拟器 ⇒ 上传未完成态可演示（真上传属后续卡）。
    func testInputBarInjectedSimulatorCanStallUploading() {
        let stalling = ChatInputBar(text: .constant(""), attachments: .constant([]),
                                    phase: .constant(.uploading), uploadSimulator: { _ in nil })
        XCTAssertNil(stalling.uploadSimulator(1))
        XCTAssertEqual(stalling.phase, .uploading)
        let done = ChatInputBar(text: .constant("hi"), attachments: .constant(["a.pdf"]),
                                phase: .constant(.ready), uploadSimulator: { _ in "b.pdf" })
        XCTAssertEqual(done.uploadSimulator(2), "b.pdf")
        XCTAssertEqual(done.attachments, ["a.pdf"])
        XCTAssertEqual(done.text, "hi")
    }

    /// 四态均可实例化（两态交互 + 上传中 + 录音高亮）。
    func testInputBarInstantiatesAllPhases() {
        for p: InputPhase in [.idle, .ready, .uploading, .recording] {
            let bar = ChatInputBar(text: .constant(p == .ready ? "hi" : ""),
                                   attachments: .constant(p == .uploading ? ["a.pdf"] : []),
                                   phase: .constant(p))
            XCTAssertEqual(bar.phase, p)
        }
    }
}