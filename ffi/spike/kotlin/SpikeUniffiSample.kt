// FFI spike：**uniffi 方案**的 Kotlin 侧调用样例（**候选**）。
//
// ⛔ 本文件是**样例**，不是生产代码。
// ⚠️ 本环境**无 Android SDK/NDK**，所以本文件**没有**在 Android 上跑过；
//    它只做过 **JVM 上的编译 + 运行**（用 host 版 `.so` 代替 Android `.so`）。
//    ⇒ **无真机/模拟器验证**（见 `docs/ffi-spike.md` 测项 5）。
//
// # 与手写 C ABI 的对照点
//
// uniffi 的 Kotlin 侧：
//   1. 绑定**自动生成**；
//   2. 内存**自动管理**；
//   3. 错误是**带类型的异常**（`SpikeException.KeyLength` 等），字段带类型。
//
// 手写 C ABI 的 Kotlin 侧需要：
//   1. 手工声明结构体布局；
//   2. 手工管理每一次释放；
//   3. 手工把错误码翻译成异常。
//
// ⚠️⚠️ **实测到的 Kotlin 侧摩擦**（已记入 `docs/ffi-spike.md` 测项 5/6）：
// uniffi 把 Rust 的 `u32` 映射成 `kotlin.UInt`、`u64` 映射成 `kotlin.ULong`。
// ⇒ 移动端每次用这些值都要跟**无符号**类型打交道
//   （`UInt` 与 `Int` 不能直接比较，转换要显式写 `.toUInt()` / `.toLong()`）。
//   这不是错误，但**是**每个调用点的额外心智负担。

package uniffi.spike

/** 打开一个样例会话：**返回结构体** + **错误跨语言**（抛异常）。 */
fun openSampleSession(): SpikeSessionInfo {
    // 32 字节样例密钥（**测试数据**，非真实密钥）。
    val key = ByteArray(32) { it.toByte() }
    return openSession(key, "peer-a")
}

/** 能力名列表：**返回 `Result`** 的样例。 */
fun sampleCapabilityNames(): List<String> = capabilityNames(0b1111u)

/**
 * 错误分支：短密钥 ⇒ 抛出 `SpikeException.KeyLength`。
 *
 * ⛔ 断言错误**不泄露密钥字节**（沿用 `TransportKey` 纪律）。
 */
fun expectKeyLengthError() {
    val shortKey = byteArrayOf(0x01, 0x02, 0x03, 0x04)
    try {
        openSession(shortKey, "peer-a")
        error("短密钥本应失败")
    } catch (e: SpikeException.KeyLength) {
        // ⚠️ 注意：字段是 `UInt` ⇒ 与字面量比较要显式转换。
        check(e.actual == 4u) { "实际长度应为 4，实际 ${e.actual}" }
        check(e.expected == 32u) { "期望长度应为 32，实际 ${e.expected}" }
        // 错误消息可见，但**不得**包含密钥字节。
        val text = e.message ?: ""
        for (b in shortKey) {
            check(!text.contains(b.toInt().toChar())) { "错误消息不得包含密钥字节" }
        }
    }
}

/**
 * 回调样例：实现 uniffi 的 `EventSink` 接口。
 *
 * ⚠️ uniffi 的回调对象是**一个对象**（持有状态），
 * 不需要全局变量 —— 这是它相对手写 C ABI 的优势。
 */
class SampleEventSink : SpikeEventSink {
    private val lock = Any()
    private val items = mutableListOf<Triple<ULong, String, String>>()

    /** 收到的回调（线程安全读取）。 */
    val received: List<Triple<ULong, String, String>>
        get() = synchronized(lock) { items.toList() }

    override fun onEvent(event: SpikeEvent) {
        synchronized(lock) {
            items.add(Triple(event.seq, event.kind, event.payload))
        }
    }
}

/** 跑一遍回调样例，返回收到的条数。 */
fun sampleCallback(): Int {
    val sink = SampleEventSink()
    val delivered = emitEvents(3u, sink)
    check(delivered == 3u) { "应回调 3 次" }
    return sink.received.size
}

/** 一个不依赖测试框架的自检入口。 */
object SpikeSelfCheck {
    @JvmStatic
    fun run(): Boolean {
        return try {
            val info = openSampleSession()
            check(info.peerLabel == "peer-a")
            check(info.epoch == 1uL)
            check(sampleCapabilityNames().size == 4)
            expectKeyLengthError()
            check(sampleCallback() == 3)
            true
        } catch (e: Throwable) {
            println("SPIKE_ERROR=$e")
            false
        }
    }
}
