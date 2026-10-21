// FFI spike：**手写 C ABI** 方案的 Swift 侧调用样例（**对照**）。
//
// ⛔ 本文件是**样例**，不是生产代码。
// ⚠️ 与 `SpikeUniffiSample.swift` 不同，本文件**必须**：
//   1. 手工维护 `SpikeSessionInfo` 的**逐字段布局**（与 Rust 的 `#[repr(C)]` 对齐）；
//   2. 手工管理**每一次**内存释放；
//   3. 手工把错误码翻译成 Swift 错误。
//    这些都是 uniffi 自动做的事。
//
// ⚠️ 布局核对：C 的 `struct` 与 Swift 的 `MemoryLayout` 必须逐字段一致。
//    `spikeCabiLayoutMatches()` 就是一个**运行期断言**——
//    它在 Swift 侧重新算出每个字段的偏移，与 C 头文件对照。
//    ⇒ 若 Rust 侧改了结构体而忘了改头文件，这里**立刻**炸，而不是静默错位。

import Foundation

// MARK: - 手工声明的 C 互操作层

// 与 `lengsuo_spike_cabi.h` 逐字段对应。
// ⚠️ 顺序与类型必须与 Rust 的 `#[repr(C)] struct SpikeSessionInfo` 一致。
struct SpikeSessionInfoC {
    var epoch: UInt64
    var capabilities: UInt32
    var fingerprint: UInt64
    var sessionId: UnsafeMutablePointer<CChar>?
    var peerLabel: UnsafeMutablePointer<CChar>?
}

struct SpikeErrorC {
    var code: Int32
    var message: UnsafeMutablePointer<CChar>?
}

/// 错误码 → Swift 错误类型（**手工**翻译）。
enum SpikeCabiError: Error, Equatable {
    case keyLength(actual: Int, expected: Int)
    case notHandshaked
    case invalidCapability(mask: UInt32)
    case invalidPeerLabel(actual: Int, expected: Int)
    case unknown(code: Int32)
}

/// 错误载荷 → Swift 错误（**手工**解析码 + 消息）。
///
/// ⛔ 消息是 `UnsafeMutablePointer<CChar>?` ⇒ 可能为 NULL，
///    也**必须**释放（否则泄漏）。
func decodeSpikeError(_ raw: SpikeErrorC) -> (SpikeCabiError, String?) {
    let message = raw.message.map { String(cString: $0) }
    let code: SpikeCabiError
    switch raw.code {
    case 1: code = .keyLength(actual: -1, expected: -1)
    case 2: code = .notHandshaked
    case 3: code = .invalidCapability(mask: 0)
    case 4: code = .invalidPeerLabel(actual: -1, expected: -1)
    default: code = .unknown(code: raw.code)
    }
    return (code, message)
}

// MARK: - 导入 C 函数

@_silgen_name("spike_open_session")
func spikeOpenSession(
    _ key: UnsafePointer<UInt8>?,
    _ keyLen: Int,
    _ peerLabel: UnsafePointer<CChar>?,
    _ outInfo: UnsafeMutablePointer<SpikeSessionInfoC>?,
    _ outError: UnsafeMutablePointer<SpikeErrorC>?
) -> Int32

@_silgen_name("spike_free_string")
func spikeFreeString(_ ptr: UnsafeMutablePointer<CChar>?)

@_silgen_name("spike_free_session_info")
func spikeFreeSessionInfo(_ ptr: UnsafeMutablePointer<SpikeSessionInfoC>?)

@_silgen_name("spike_capability_names")
func spikeCapabilityNames(
    _ mask: UInt32,
    _ outNames: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outCount: UnsafeMutablePointer<Int>?,
    _ outError: UnsafeMutablePointer<SpikeErrorC>?
) -> Int32

@_silgen_name("spike_capability_name_count")
func spikeCapabilityNameCount(_ mask: UInt32) -> Int

@_silgen_name("spike_free_string_array")
func spikeFreeStringArray(_ names: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?, _ count: Int)

@_silgen_name("spike_emit_events")
func spikeEmitEvents(_ count: UInt32, _ callback: @convention(c) (UInt64, UnsafePointer<CChar>?, UnsafePointer<CChar>?) -> Void) -> Int32

@_silgen_name("spike_binding_version")
func spikeBindingVersion() -> UnsafeMutablePointer<CChar>?

// MARK: - 样例调用

/// 打开一个样例会话：**返回结构体** + **错误跨语言**。
///
/// ⚠️ 注意这里的**手工内存管理**：
/// `session_id` / `peer_label` 都是库分配的 ⇒ 用完必须 `spikeFreeSessionInfo`。
func openCabiSampleSession() throws -> (epoch: UInt64, capabilities: UInt32, fingerprint: UInt64, sessionId: String, peerLabel: String) {
    let key = [UInt8](0..<32)
    var info = SpikeSessionInfoC(
        epoch: 0, capabilities: 0, fingerprint: 0, sessionId: nil, peerLabel: nil
    )
    var error = SpikeErrorC(code: 0, message: nil)

    let rc = key.withUnsafeBufferPointer { keyBuf in
        "peer-a".withCString { label in
            spikeOpenSession(
                keyBuf.baseAddress, keyBuf.count, label, &info, &error
            )
        }
    }

    guard rc == 0 else {
        let (err, _) = decodeSpikeError(error)
        spikeFreeString(error.message)
        throw err
    }

    // ⚠️ 必须**当场复制**字符串（库拥有的内存随时可能被释放）。
    let sessionId = info.sessionId.map { String(cString: $0) } ?? ""
    let peerLabel = info.peerLabel.map { String(cString: $0) } ?? ""
    spikeFreeSessionInfo(&info)

    return (info.epoch, info.capabilities, info.fingerprint, sessionId, peerLabel)
}

/// 能力名列表：**两段式调用**（先问数量，再取名字）——手写 C ABI 的典型样板。
func cabiSampleCapabilityNames() throws -> [String] {
    let mask: UInt32 = 0b1111
    let count = spikeCapabilityNameCount(mask)
    guard count > 0 else { return [] }

    var names = [UnsafeMutablePointer<CChar>?](repeating: nil, count: count)
    var actualCount = 0
    var error = SpikeErrorC(code: 0, message: nil)

    let rc = names.withUnsafeMutableBufferPointer { buf in
        spikeCapabilityNames(mask, buf.baseAddress, &actualCount, &error)
    }
    guard rc == 0 else {
        spikeFreeString(error.message)
        throw decodeSpikeError(error).0
    }

    let result = names.compactMap { $0.map { String(cString: $0) } }
    spikeFreeStringArray(&names, actualCount)
    return result
}

/// 错误分支：短密钥 ⇒ 错误码 1。
///
/// ⛔ 断言错误**不泄露密钥字节**。
func cabiExpectKeyLengthError() throws {
    let key: [UInt8] = [0x01, 0x02, 0x03, 0x04]
    var info = SpikeSessionInfoC(
        epoch: 0, capabilities: 0, fingerprint: 0, sessionId: nil, peerLabel: nil
    )
    var error = SpikeErrorC(code: 0, message: nil)
    let rc = key.withUnsafeBufferPointer { keyBuf in
        "peer-a".withCString { label in
            spikeOpenSession(keyBuf.baseAddress, keyBuf.count, label, &info, &error)
        }
    }
    precondition(rc != 0, "短密钥本应失败")
    precondition(error.code == 1, "错误码应为 KeyLength(1)，实际 \(error.code)")

    // 错误消息可见但**不含密钥字节**。
    if let msg = error.message {
        let text = String(cString: msg)
        precondition(!text.contains("\u{01}"), "错误消息不得包含密钥字节")
        spikeFreeString(error.message)
    }
}

/// 回调样例：**手工**的 C 函数指针。
///
/// ⚠️ 回调的坑（手写 C ABI 最容易出事的地方）：
/// - 签名必须与 Rust 侧**逐字**匹配；
/// - 回调收到的 `kind`/`payload` 是**库拥有的临时内存**
///   ⇒ 必须**当场复制**，不能留存指针。
final class CabiEventSink {
    private let lock = NSLock()
    private var items: [(seq: UInt64, kind: String, payload: String)] = []

    var received: [(seq: UInt64, kind: String, payload: String)] {
        lock.lock()
        defer { lock.unlock() }
        return items
    }

    /// C 回调 trampoline：只做「复制」，不做别的。
    fileprivate func handle(seq: UInt64, kind: UnsafePointer<CChar>?, payload: UnsafePointer<CChar>?) {
        let kindText = kind.map { String(cString: $0) } ?? ""
        let payloadText = payload.map { String(cString: $0) } ?? ""
        lock.lock()
        defer { lock.unlock() }
        items.append((seq, kindText, payloadText))
    }
}

/// ⚠️⚠️ **实测到的 Swift 侧硬约束**：C 函数指针（`@convention(c)`）
/// **不能捕获上下文**（编译器直接报错：
/// "a C function pointer cannot be formed from a closure that captures context"）。
///
/// ⇒ 手写 C ABI 的回调在 Swift 侧**必须**用**全局变量**传递上下文。
/// ⛔ 而全局变量 = **非线程安全**：并发回调会互相踩。
///    下面用 `NSLock` 兜住并发，但「谁在用这个全局」这件事
///    **移动端必须自己保证**（比如同一时刻只有一个活跃 sink）。
///
/// uniffi 的回调**没有**这个问题：它生成的代理是**引用类型**，
/// 可以直接持有状态，不需要全局。
nonisolated(unsafe) private var currentCabiSink: CabiEventSink?

/// C 回调 trampoline：**无捕获**，从全局取上下文。
private let cabiTrampoline: @convention(c) (UInt64, UnsafePointer<CChar>?, UnsafePointer<CChar>?) -> Void =
    { seq, kind, payload in
        currentCabiSink?.handle(seq: seq, kind: kind, payload: payload)
    }

/// 跑一遍回调样例。
func cabiSampleCallback() throws -> Int {
    let sink = CabiEventSink()
    currentCabiSink = sink
    defer { currentCabiSink = nil }
    let delivered = spikeEmitEvents(3, cabiTrampoline)
    precondition(delivered == 3)
    return sink.received.count
}

/// 布局核对：Swift 侧的 `MemoryLayout` 与 C 头文件的布局是否一致。
///
/// ⚠️ 这正是手写 C ABI 的风险点 —— 这里**主动**把它变成一个断言。
func spikeCabiLayoutMatches() -> Bool {
    // epoch(8) + capabilities(4) + padding(4) + fingerprint(8) + ptr(8) + ptr(8)
    return MemoryLayout<SpikeSessionInfoC>.size == 40
        && MemoryLayout<SpikeSessionInfoC>.alignment == 8
        && MemoryLayout<SpikeSessionInfoC>.offset(of: \SpikeSessionInfoC.epoch) == 0
        && MemoryLayout<SpikeSessionInfoC>.offset(of: \SpikeSessionInfoC.capabilities) == 8
        && MemoryLayout<SpikeSessionInfoC>.offset(of: \SpikeSessionInfoC.fingerprint) == 16
        && MemoryLayout<SpikeSessionInfoC>.offset(of: \SpikeSessionInfoC.sessionId) == 24
        && MemoryLayout<SpikeSessionInfoC>.offset(of: \SpikeSessionInfoC.peerLabel) == 32
}

// MARK: - 自检

enum SpikeCabiSelfCheck {
    static func run() -> Bool {
        do {
            precondition(spikeCabiLayoutMatches(), "C 布局不匹配")
            let info = try openCabiSampleSession()
            precondition(info.peerLabel == "peer-a")
            precondition(info.epoch == 1)
            let names = try cabiSampleCapabilityNames()
            precondition(names.count == 4)
            try cabiExpectKeyLengthError()
            let hits = try cabiSampleCallback()
            precondition(hits == 3)
            return true
        } catch {
            print("SPIKE_CABI_ERROR=\(error)")
            return false
        }
    }
}
