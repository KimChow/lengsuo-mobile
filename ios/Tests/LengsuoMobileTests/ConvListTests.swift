import XCTest
@testable import LengsuoMobile

final class ConvListTests: XCTestCase {
    func testPrefixesVerbatim() {
        // 04§2.2 原文前缀
        XCTAssertEqual(ConvTrack.local.prefix, "/chats/local_")
        XCTAssertEqual(ConvTrack.cloud.prefix, "/chats/cloud_")
        XCTAssertEqual(ConvTrack.bga.prefix, "/chats/local_bga_")
    }

    func testTrackInference() {
        XCTAssertEqual(Conversation(id: "/chats/local_bga_9", title: "x").track, .bga)
        XCTAssertEqual(Conversation(id: "/chats/cloud_42", title: "x").track, .cloud)
        XCTAssertEqual(Conversation(id: "/chats/local_draft_1", title: "x").track, .local)
    }

    func testGrouping() {
        let g = groupConversations(ConversationListView.sample)
        XCTAssertEqual(g.local.count, 2)   // local + bga
        XCTAssertEqual(g.cloud.count, 1)
    }

    func testBgaGoesToLocalSection() {
        let bga = Conversation(id: "/chats/local_bga_1", title: "x")
        XCTAssertTrue(groupConversations([bga]).local.contains(bga))
    }
}