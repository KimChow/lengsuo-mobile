import XCTest
@testable import LengsuoMobile

final class MeSettingsTests: XCTestCase {
    /// 本卡涉及的全部 UserDefaults key（后缀）；逐用例清空以免跨用例串味。
    private let suffixKeys = [
        "fontSizeIndex", "readingEnabled", "hapticsEnabled",
        "notifyTaskResult", "notifyScheduledResult", "notifyPendingApproval", "notifyGlobal",
    ]

    override func setUp() {
        super.setUp()
        clearKeys()
    }

    override func tearDown() {
        clearKeys()
        super.tearDown()
    }

    private func clearKeys() {
        for k in suffixKeys { UserDefaults.standard.removeObject(forKey: MeSettingsStore.prefix + k) }
    }

    // MARK: - §2.4 指定用例（逐字）

    func testHapticsDefaultOn() {
        // 15§2「触感反馈（开）」
        UserDefaults.standard.removeObject(forKey: "lengsuo.me.hapticsEnabled")
        XCTAssertTrue(MeSettingsStore().hapticsEnabled)
    }

    func testFontSizeFiveLevels() {
        XCTAssertEqual(MeFontSize.allCases.count, 5)
        XCTAssertEqual(MeFontSize.standard.title, "标准")
    }

    func testNotifyDefaultsAllOn() {
        for k in ["notifyTaskResult", "notifyScheduledResult", "notifyPendingApproval", "notifyGlobal"] {
            UserDefaults.standard.removeObject(forKey: "lengsuo.me." + k)
        }
        let s = MeSettingsStore()
        XCTAssertTrue(s.notifyTaskResult && s.notifyScheduledResult
                      && s.notifyPendingApproval && s.notifyGlobal)
    }

    func testFontSizePersistence() {
        let s = MeSettingsStore(); s.fontSizeIndex = 3
        XCTAssertEqual(MeSettingsStore().fontSizeIndex, 3)
        UserDefaults.standard.removeObject(forKey: "lengsuo.me.fontSizeIndex")
    }

    // MARK: - 本卡补充用例

    func testFontSizeTitlesVerbatimAndDefaultStandard() {
        // 15§5.2 五档逐字 + 默认「标准」（index 1）
        XCTAssertEqual(MeFontSize.allCases.map(\.title), ["小", "标准", "大", "特大", "最大"])
        XCTAssertEqual(MeFontSize.allCases.map(\.rawValue), [0, 1, 2, 3, 4])
        XCTAssertEqual(MeSettingsStore().fontSizeIndex, 1)
    }

    func testFontSizeIndexMapsToAscendingDynamicTypeSize() {
        // 15§5.2：档位 → dynamicTypeSize；「标准」= 系统默认 .large
        XCTAssertEqual(MeFontSize.standard.dynamicTypeSize, .large)
        XCTAssertEqual(MeFontSize.allCases.map(\.dynamicTypeSize),
                       [.small, .large, .xLarge, .xxLarge, .xxxLarge])
        // 越界 index 回退「标准」，不崩溃
        XCTAssertNil(MeFontSize(rawValue: 99))
    }

    func testReadingDefaultsOffAndRoundTrips() {
        XCTAssertFalse(MeSettingsStore().readingEnabled)
        let s = MeSettingsStore(); s.readingEnabled = true
        XCTAssertTrue(MeSettingsStore().readingEnabled)
    }

    func testHapticsRoundTrips() {
        let s = MeSettingsStore()
        s.hapticsEnabled = false
        XCTAssertFalse(MeSettingsStore().hapticsEnabled)
        s.hapticsEnabled = true
        XCTAssertTrue(MeSettingsStore().hapticsEnabled)
    }

    func testNotifyFourGranularitiesRoundTripIndependently() {
        // 11§7 四粒度互不干扰：逐个翻转并各自往返
        let s = MeSettingsStore()
        let cases: [(String, ReferenceWritableKeyPath<MeSettingsStore, Bool>)] = [
            ("任务结果", \.notifyTaskResult),
            ("定时任务结果", \.notifyScheduledResult),
            ("待回复待审批", \.notifyPendingApproval),
            ("退后台总开关", \.notifyGlobal),
        ]
        for (_, kp) in cases { s[keyPath: kp] = false }
        let reloaded = MeSettingsStore()
        for (name, kp) in cases {
            XCTAssertFalse(reloaded[keyPath: kp], "\(name) 应持久化为关闭")
        }
        s.notifyPendingApproval = true
        XCTAssertTrue(MeSettingsStore().notifyPendingApproval)
        XCTAssertFalse(MeSettingsStore().notifyTaskResult, "翻转单粒度不得影响其余粒度")
    }

    // MARK: - 文案逐字 / 红线（源码级）

    func testEntryNamesAndSubPageTitlesVerbatim() throws {
        let me = meSourceDir()
        let page = try String(contentsOf: me.appendingPathComponent("MePage.swift"), encoding: .utf8)
        let pages = try String(contentsOf: me.appendingPathComponent("MeSettingsPages.swift"), encoding: .utf8)

        // 15§2 我的行入口名逐字（5 个可导航入口 + 语言行保持静态）
        for entry in ["字体大小", "朗读设置", "触感反馈", "存储空间", "消息通知设置"] {
            XCTAssertTrue(page.contains("\"\(entry)\""), "入口名缺失：\(entry)")
        }
        XCTAssertTrue(page.contains("跟随系统"), "语言行应保持静态「跟随系统」")

        // 子页标题逐字
        for title in ["字体大小", "朗读与触感", "存储空间", "消息通知设置"] {
            XCTAssertTrue(pages.contains("\"\(title)\""), "子页标题缺失：\(title)")
        }
        // 11§7 通知四粒度文案逐字
        for granularity in ["任务结果通知", "定时任务结果", "待回复/待审批提醒", "退后台后接收全部通知"] {
            XCTAssertTrue(pages.contains("\"\(granularity)\""), "通知粒度文案缺失：\(granularity)")
        }
    }

    func testStoragePageHasNoFakeNumbers() throws {
        let pages = try String(contentsOf: meSourceDir().appendingPathComponent("MeSettingsPages.swift"),
                               encoding: .utf8)
        let start = try XCTUnwrap(pages.range(of: "public struct StoragePage"))
        let end = try XCTUnwrap(pages.range(of: "// MARK: - 消息通知设置"))
        let block = String(pages[start.lowerBound..<end.lowerBound])
        XCTAssertTrue(block.contains("计算中…"), "存储页应为「计算中…」占位")
        XCTAssertFalse(block.contains(where: \.isNumber), "存储页不得出现任何具体数字（⛔ 假数据）")
    }

    /// `ios/Tests/LengsuoMobileTests/<本文件>` → `ios/Sources/LengsuoMobile/Me`
    private func meSourceDir() -> URL {
        URL(fileURLWithPath: #filePath)          // .../ios/Tests/LengsuoMobileTests/MeSettingsTests.swift
            .deletingLastPathComponent()          // .../ios/Tests/LengsuoMobileTests
            .deletingLastPathComponent()          // .../ios/Tests
            .deletingLastPathComponent()          // .../ios
            .appendingPathComponent("Sources/LengsuoMobile/Me")
    }
}