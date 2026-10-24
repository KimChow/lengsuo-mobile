import XCTest
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

    }