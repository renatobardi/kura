// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "KuraPushKit",
    platforms: [.iOS(.v15), .macOS(.v12)],
    products: [
        .library(name: "KuraPushKit", targets: ["KuraPushKit"])
    ],
    dependencies: [
        .package(url: "https://github.com/21-DOT-DEV/swift-secp256k1.git", exact: "0.21.1")
    ],
    targets: [
        .target(
            name: "KuraPushKit",
            dependencies: [.product(name: "P256K", package: "swift-secp256k1")]
        ),
        .testTarget(
            name: "KuraPushKitTests",
            dependencies: [
                "KuraPushKit",
                .product(name: "P256K", package: "swift-secp256k1"),
            ],
            resources: [.copy("Fixtures/app_attest_transcripts.json")]
        ),
    ]
)
