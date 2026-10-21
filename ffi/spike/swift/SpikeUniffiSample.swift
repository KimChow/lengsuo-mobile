// FFI spike：**uniffi 方案**的 Swift 侧调用样例（**候选**）。
//
// ⛔ 本文件是**样例**，不是生产代码。
// ⛔ 本环境**无 macOS / 无 Xcode** ⇒ 本文件**从未**被 `swift build` / `swift test` 编译过。
//    它只做过**语法/类型级**的静态检查（见 `ffi/spike/README.md` 的「静态检查」一节）。
//
// # 与手写 C ABI 的对照点
//
// 手写 C ABI 的 Swift 侧需要：
//   1. 自己声明 `struct SpikeSessionInfo`（逐字段对齐 `#[repr(C)]`）；
//   2. 自己管理字符串释放（`spike_free_string`）；
//   3. 自己解析错误码 + 消息。
//
// uniffi 的 Swift 侧（本文件）：
//   1. 绑定**自动生成**（`SpikeSessionInfo` 是带具名字段的 struct）；
//   2. 内存**自动管理**；
//   3. 错误是**带类型的枚举**（`SpikeError`），`throws` 出来。
//
// ⇒ 差异见 `docs/ffi-spike.md` 测项 6/7。

import Foundation

// MARK: - 样例调用

/// 打开一个样例会话：**返回结构体** + **错误跨语言**（`throws`）。
///
/// - Throws: `SpikeError`（uniffi 的富错误枚举，字段带类型）。
func openSampleSession() throws -> SpikeSessionInfo {
    // 32 字节样例密钥（**测试数据**，非真实密钥）。
    let key = Data((0..<32).map { UInt8($0) })
    return try openSession(key: key, peerLabel: "peer-a")
}

/// 能力名列表：**返回 `Result`** 的样例。
func sampleCapabilityNames() throws -> [String] {
    return try capabilityNames(mask: 0b1111)
}

/// 错误分支：短密钥 ⇒ 抛出 `SpikeError.KeyLength(actual:expected:)`。
///
/// ⛔ 断言错误**不泄露密钥字节**（沿用 `TransportKey` 纪律）。
func expectKeyLengthError() throws {
    let shortKey = Data([0x01, 0x02, 0x03, 0x04])
    do {
        _ = try openSession(key: shortKey, peerLabel: "peer-a")
        fatalError("短密钥本应失败")
    } catch let error as SpikeError {
        switch error {
        case let .KeyLength(actual, expected):
            // 错误码与长度**可见**，密钥字节**不可见**。
            precondition(actual == 4)
            precondition(expected == 32)
            // ⛔ 错误描述里**不得**出现密钥的可打印字节。
            let text = String(describing: error)
            for byte in shortKey {
                precondition(!text.contains(Character(UnicodeScalar(byte))))
            }
        default:
            fatalError("错误类型不符：\(error)")
        }
    }
}

/// 回调样例：实现 uniffi 的 `SpikeEventSink` 协议。
///
/// ⚠️ uniffi 的回调对象是 `AnyObject`（引用类型），
/// 且**代理是线程受限的** —— 必须在**创建它的那个线程**上被回调。
///
/// ⚠️⚠️ 另一个**实测到的**约束：uniffi 生成的 `SpikeEventSink` 协议要求
/// `Sendable` 一致性 ⇒ 移动端实现方**必须**自己处理线程安全
/// （Swift 6 语言模式下「`Sendable` 类里的可变存储」直接是**错误**）。
/// ⇒ 这是 uniffi 回调的**真实成本**，已记入 `docs/ffi-spike.md` 测项 9。
final class SampleEventSink: SpikeEventSink {
    /// 可变状态装进一个显式 `@unchecked Sendable` 的盒子里：
    /// 这样 `SampleEventSink` 自身只剩 `let` 存储 ⇒ 满足 `Sendable`。
    private final class Box: @unchecked Sendable {
        private let lock = NSLock()
        private var items: [(seq: UInt64, kind: String, payload: String)] = []

        func append(_ item: (seq: UInt64, kind: String, payload: String)) {
            lock.lock()
            defer { lock.unlock() }
            items.append(item)
        }

        var snapshot: [(seq: UInt64, kind: String, payload: String)] {
            lock.lock()
            defer { lock.unlock() }
            return items
        }
    }

    private let box = Box()

    /// 收到的回调（线程安全读取）。
    var received: [(seq: UInt64, kind: String, payload: String)] {
        box.snapshot
    }

    /// uniffi 生成的协议要求**带标签**的参数名 `onEvent(event:)`。
    func onEvent(event: SpikeEvent) {
        box.append((event.seq, event.kind, event.payload))
    }
}

/// 跑一遍回调样例，返回收到的条数。
func sampleCallback() throws -> Int {
    let sink = SampleEventSink()
    let delivered = emitEvents(count: 3, sink: sink)
    precondition(delivered == 3)
    return sink.received.count
}

// MARK: - 自检（无 Xcode 环境下只能靠人工/静态检查）

/// 一个不依赖测试框架的自检入口：可在真机/模拟器上直接跑。
enum SpikeSelfCheck {
    static func run() -> Bool {
        do {
            let info = try openSampleSession()
            precondition(info.peerLabel == "peer-a")
            precondition(info.epoch == 1)
            let names = try sampleCapabilityNames()
            precondition(names.count == 4)
            try expectKeyLengthError()
            let callbackHits = try sampleCallback()
            precondition(callbackHits == 3)
            return true
        } catch {
            return false
        }
    }
}
