// swift-tools-version:5.10
import PackageDescription

let package = Package(
    name: "LengsuoMobile",
    platforms: [.iOS(.v17)],
    products: [
        .library(name: "LengsuoMobile", targets: ["LengsuoMobile"]),
    ],
    dependencies: [],
    targets: [
        .target(name: "LengsuoMobile", dependencies: []),
        .testTarget(name: "LengsuoMobileTests", dependencies: ["LengsuoMobile"]),
    ]
)
