import Foundation

/// lengsuo-mobile iOS companion 壳的根模块。
/// 当前仅骨架：版本信息与基本自检入口，UI 与 FFI 接线由后续任务卡逐块填入。
public enum LengsuoMobile {
    public static let version = "0.1.0"

    public static func greeting() -> String {
        "LengsuoMobile iOS companion v\(version)"
    }
}
