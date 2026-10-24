import XCTest
@testable import LengsuoMobile

final class ThemeTests: XCTestCase {
    func testThreeStatesVerbatim() {
        // android-devguide/15 §5.2：系统/浅色/深色
        XCTAssertEqual(ColorSchemePreference.allCases.map(\.title), ["系统", "浅色", "深色"])
    }

    func testOrderMatchesSpec() {
        XCTAssertEqual(ColorSchemePreference.allCases, [.system, .light, .dark])
    }

    func testPersistenceRoundTrip() {
        let key = "lengsuo.theme.preference"
        UserDefaults.standard.removeObject(forKey: key)
        let m1 = ThemeManager()
        XCTAssertEqual(m1.preference, .system)
        m1.set(.dark)
        let m2 = ThemeManager()
        XCTAssertEqual(m2.preference, .dark)
        UserDefaults.standard.removeObject(forKey: key)
    }
}
