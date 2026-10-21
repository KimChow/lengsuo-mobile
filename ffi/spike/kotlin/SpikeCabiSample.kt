// FFI spike：**手写 C ABI** 方案的 Kotlin 侧调用样例（**对照**）。
//
// ⛔ 本文件是**样例**，不是生产代码。
// ⚠️ 与 `SpikeUniffiSample.kt` 不同，本文件**必须**：
//   1. 手工声明结构体布局（`JNA Structure`）；
//   2. 手工管理**每一次**内存释放；
//   3. 手工把错误码翻译成异常。
//
// ⚠️ 本环境**无 Android SDK/NDK** ⇒ 没有在 Android 上跑过；
//    只在 **JVM 上编译 + 运行**（host 版 `.so` 代替 Android `.so`）。
//
// # 手写 C ABI 在 Kotlin 侧的**真实成本**（实测）
//
// - 布局要**逐字段**写进 `JNA Structure`，写错**不报错**、静默错位；
// - 字符串是 `Pointer` ⇒ 要自己 `getString` + 自己释放；
// - 错误是 `Int` 返回码 ⇒ 要自己翻译成异常；
// - 回调要实现 `com.sun.jna.Callback` 接口；因为 JNA 的 `Callback` 是
//   **接口**（不是类），可以用**匿名对象**携带状态
//   —— 比 Swift 侧的「必须用全局变量」干净一点，但**仍要**自己处理线程。

package spike.cabi

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Memory
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

/** 错误码：与 Rust 侧 `SpikeErrorCode` 逐值对应。 */
object SpikeErrorCode {
    const val UNKNOWN = 0
    const val KEY_LENGTH = 1
    const val NOT_HANDSHAKED = 2
    const val INVALID_CAPABILITY = 3
    const val INVALID_PEER_LABEL = 4
}

/** 错误载荷：`code` + 可为 null 的 `message`。 */
class SpikeError : Structure() {
    @JvmField
    var code: Int = 0

    @JvmField
    var message: Pointer? = null

    override fun toString(): String = "SpikeError(code=$code, message=${getMessage()})"

    /**
     * ⚠️ JNA 要求**显式**给出字段顺序（与 C 结构体一致）。
     * 这就是手写 C ABI 在 Kotlin 侧的又一处**手工活**：
     * 漏写 ⇒ 运行期报错（还算好）；写错顺序 ⇒ **静默错位**。
     */
    override fun getFieldOrder(): MutableList<String> =
        mutableListOf("code", "message")

    /** 读取消息（**不**释放）。 */
    fun getMessage(): String? = message?.getString(0)
}

/** 会话信息：与 Rust 侧 `#[repr(C)] struct SpikeSessionInfo` 逐字段对齐。 */
class SpikeSessionInfo : Structure() {
    @JvmField
    var epoch: Long = 0

    @JvmField
    var capabilities: Int = 0

    @JvmField
    var fingerprint: Long = 0

    @JvmField
    var sessionId: Pointer? = null

    @JvmField
    var peerLabel: Pointer? = null

    override fun toString(): String =
        "SpikeSessionInfo(epoch=$epoch, capabilities=$capabilities, " +
            "fingerprint=$fingerprint, sessionId=${getSessionId()}, peerLabel=${getPeerLabel()})"

    /**
     * ⚠️ JNA 要求**显式**给出字段顺序（与 C 结构体一致）。
     * ⇒ 与 Rust 侧 `#[repr(C)] struct SpikeSessionInfo` 逐字对应。
     */
    override fun getFieldOrder(): MutableList<String> =
        mutableListOf("epoch", "capabilities", "fingerprint", "sessionId", "peerLabel")

    /** 读取字符串（**不**释放）。 */
    fun getSessionId(): String? = sessionId?.getString(0)

    /** 读取字符串（**不**释放）。 */
    fun getPeerLabel(): String? = peerLabel?.getString(0)
}

/**
 * 事件回调。
 *
 * ⚠️ JNA 的 `Callback` 是**接口** ⇒ 用**匿名对象**实现即可携带状态
 *   （Swift 侧做不到，必须用全局变量 —— 两侧的差异见 `docs/ffi-spike.md` 测项 9）。
 */
interface SpikeEventCallback : Callback {
    fun onEvent(seq: Long, kind: String, payload: String)
}

/** 手写 C ABI 的导入层。 */
interface SpikeCabiLib : Library {
    fun spike_free_string(ptr: Pointer?)

    fun spike_free_session_info(ptr: SpikeSessionInfo?)

    fun spike_free_string_array(names: Pointer?, count: Int)

    fun spike_open_session(
        key: ByteArray?,
        keyLen: Int,
        peerLabel: String?,
        outInfo: SpikeSessionInfo?,
        outError: SpikeError?,
    ): Int

    fun spike_capability_names(
        mask: Int,
        outNames: Pointer?,
        outCount: IntArray?,
        outError: SpikeError?,
    ): Int

    fun spike_capability_name_count(mask: Int): Int

    fun spike_emit_events(count: Int, callback: SpikeEventCallback?): Int

    fun spike_binding_version(): Pointer?

    companion object {
        /** ⚠️ 手写 C ABI 的**手工步骤**：库名与加载方式要自己写。 */
        val INSTANCE: SpikeCabiLib = Native.load("lengsuo_spike_cabi", SpikeCabiLib::class.java)
    }
}

/** 错误码 → Kotlin 异常（**手工**翻译）。 */
class SpikeCabiException(val code: Int, val detail: String?) :
    RuntimeException("spike error code=$code detail=$detail")

/** 打开一个样例会话：**返回结构体** + **错误跨语言**。 */
fun openCabiSampleSession(): SpikeSessionInfo {
    val key = ByteArray(32) { it.toByte() }
    val info = SpikeSessionInfo()
    val error = SpikeError()
    val rc = SpikeCabiLib.INSTANCE.spike_open_session(key, key.size, "peer-a", info, error)
    if (rc != 0) {
        val detail = error.getMessage()
        SpikeCabiLib.INSTANCE.spike_free_string(error.message)
        throw SpikeCabiException(error.code, detail)
    }
    return info
}

/** 能力名列表：**两段式调用**（先问数量，再取名字）。 */
fun cabiSampleCapabilityNames(): List<String> {
    val mask = 0b1111
    val count = SpikeCabiLib.INSTANCE.spike_capability_name_count(mask)
    if (count <= 0) return emptyList()

    val outCount = IntArray(1)
    val error = SpikeError()
    // ⚠️⚠️ 指针数组要**手工**在 JNA 里搭出来，而且**有两个致命坑**（均已实测）：
    //
    //   坑 1：`Pointer(long)` 的参数是**地址**，不是**字节数**！
    //         写 `Pointer(8 * count)` 等于「往地址 0x40 写」⇒ **SIGSEGV**。
    //         要分配内存必须用 `Memory(size)`。
    //   坑 2：JNA 的 `setPointer(i, null)` 会对 null 指针**解引用** ⇒
    //         **JVM 直接 SIGSEGV**（`Java_com_sun_jna_Native_setPointer`），
    //         **不是**可捕获的异常 ⇒ 整个进程崩。
    //
    //   ⇒ 正确做法：`Memory` 分配**未初始化**内存，让 native 侧把
    //     `out_count` 个槽位**全部**写满，之后才读。
    //
    //   ⚠️ 这正是手写 C ABI 的**真实风险**：内存错误表现为**进程崩溃**，
    //      而不是可捕获的异常。uniffi 那边是自动的，永远走不到这一步。
    val namesPtr = Memory(Native.POINTER_SIZE * count.toLong())
    val rc = SpikeCabiLib.INSTANCE.spike_capability_names(mask, namesPtr, outCount, error)
    if (rc != 0) {
        val detail = error.getMessage()
        SpikeCabiLib.INSTANCE.spike_free_string(error.message)
        throw SpikeCabiException(error.code, detail)
    }
    val result = (0 until outCount[0]).map {
        namesPtr.getPointer(it.toLong() * Native.POINTER_SIZE).getString(0)
    }
    SpikeCabiLib.INSTANCE.spike_free_string_array(namesPtr, outCount[0])
    return result
}

/** 错误分支：短密钥 ⇒ 错误码 1。 */
fun cabiExpectKeyLengthError() {
    val key = byteArrayOf(0x01, 0x02, 0x03, 0x04)
    val info = SpikeSessionInfo()
    val error = SpikeError()
    val rc = SpikeCabiLib.INSTANCE.spike_open_session(key, key.size, "peer-a", info, error)
    check(rc != 0) { "短密钥本应失败" }
    check(error.code == SpikeErrorCode.KEY_LENGTH) { "错误码应为 1，实际 ${error.code}" }
    val detail = error.getMessage()
    if (detail != null) {
        check(!detail.contains('\u0001')) { "错误消息不得包含密钥字节" }
        SpikeCabiLib.INSTANCE.spike_free_string(error.message)
    }
}

/** 回调样例：用**匿名对象**实现 `Callback` 接口（携带状态）。 */
class CabiEventSink : SpikeEventCallback {
    private val lock = Any()
    private val items = mutableListOf<Triple<Long, String, String>>()

    val received: List<Triple<Long, String, String>>
        get() = synchronized(lock) { items.toList() }

    override fun onEvent(seq: Long, kind: String, payload: String) {
        synchronized(lock) {
            items.add(Triple(seq, kind, payload))
        }
    }
}

/** 跑一遍回调样例。 */
fun cabiSampleCallback(): Int {
    val sink = CabiEventSink()
    val delivered = SpikeCabiLib.INSTANCE.spike_emit_events(3, sink)
    check(delivered == 3) { "应回调 3 次" }
    return sink.received.size
}

/** 一个不依赖测试框架的自检入口。 */
object SpikeCabiSelfCheck {
    @JvmStatic
    fun run(): Boolean {
        return try {
            val info = openCabiSampleSession()
            check(info.getPeerLabel() == "peer-a")
            check(info.epoch == 1L)
            check(cabiSampleCapabilityNames().size == 4)
            cabiExpectKeyLengthError()
            check(cabiSampleCallback() == 3)
            true
        } catch (e: Throwable) {
            println("SPIKE_CABI_ERROR=$e")
            false
        }
    }
}
