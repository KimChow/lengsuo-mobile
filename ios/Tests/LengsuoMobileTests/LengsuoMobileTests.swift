import XCTest
@testable import LengsuoMobile

final class LengsuoMobileTests: XCTestCase {
    func testGreetingContainsVersion() {
        let text = LengsuoMobile.greeting()
        XCTAssertTrue(text.contains("iOS companion"))
        XCTAssertTrue(text.contains(LengsuoMobile.version))
    }
}
