# `ffi/spike/` —— FFI 绑定层 spike（uniffi vs 手写 C ABI）

> ⛔ **这是隔离区**：本目录是**独立 workspace**（自带 `Cargo.lock`），
> **不进最终产物**，也不参与 `ffi/` 的构建。
> `ffi/` 主体**没有**引用这里的任何 crate 依赖。
>
> 📊 实测结论见 [`docs/ffi-spike.md`](../../docs/ffi-spike.md)，
> 选型决策见 [`docs/ffi-decision.md`](../../docs/ffi-decision.md)。

---

## 1. 目录结构

```
ffi/spike/
├── Cargo.toml              # 独立 workspace（members: cabi, uniffi）
├── Cargo.lock              # 独立 lockfile ⇒ 不污染 ffi/Cargo.lock
├── core/core.rs            # ★ 两种方案**共用**的业务实现（纯 Rust、无 IO、无 FFI）
├── cabi/                   # 【对照】手写 C ABI
│   ├── src/lib.rs          #   extern "C" + 手工内存/错误/回调管理
│   ├── src/tests.rs        #   边界形状的单测
│   └── include/lengsuo_spike_cabi.h   # 人工维护的 C 头文件
├── uniffi/                 # 【候选】uniffi
│   ├── src/api.rs          #   纯 Rust 语义层（无宏）⇒ 可被 ffi/tests/ 直接对照
│   └── src/lib.rs          #   uniffi 宏层（derive/export/callback_interface）
├── vectors/                # 跨端测试向量（单一事实来源）
│   ├── golden.rs           #   Rust 常量（真值）
│   └── golden.json         #   由 golden.rs 渲染，测试断言逐字节一致
├── swift/                  # 移动端样例
│   ├── SpikeUniffiSample.swift
│   └── SpikeCabiSample.swift
├── kotlin/                 # 移动端样例
│   ├── SpikeUniffiSample.kt
│   └── SpikeCabiSample.kt
├── measure.sh              # 测量：产物体积 + 构建耗时 + 依赖增量
├── measure_evolution.sh    # 测量：API 演进成本（自动还原源码）
└── run_samples.sh          # 端到端：生成绑定 → 编译 → 真跑自检
```

**★ 关键设计**：`core/core.rs` 是**唯一**的业务实现，
两种绑定层都只是它的**薄包装** ⇒ 「同一输入 ⇒ 同一行为」可被测试断言
（`ffi/tests/spike_consistency.rs` 的 7 个断言）。

---

## 2. 环境要求

| 工具 | 版本（本卡实测） | 必需？ | 说明 |
|---|---|---|---|
| Rust | 1.98.0（`rustc --version`） | ✅ | 需 `rustup target add aarch64-apple-ios aarch64-linux-android armv7-linux-androideabi x86_64-linux-android` |
| NDK | **r27c** | Android 体积测量必需 | 装在 `$HOME/tools/ndk/android-ndk-r27c`（或设 `ANDROID_NDK_HOME`） |
| Swift | **6.1.2**（Linux） | 跨语言样例 | `/tmp/swift-6.1.2-RELEASE-ubuntu24.04/usr/bin` |
| Kotlin | **2.1.0** | 跨语言样例 | `$HOME/tools/kotlin/kotlinc/bin/kotlinc` |
| JNA | 5.x | Kotlin 样例 | `/tmp/jna/jna.jar` |
| `uniffi-bindgen` | **0.32.2** | uniffi 样例 | `$HOME/tools/cargo-tools/bin/uniffi-bindgen` |

**装 NDK**（47 上 `dl.google.com` 不通，用腾讯镜像）：

```bash
curl -L -o ndk.zip https://mirrors.cloud.tencent.com/AndroidSDK/android-ndk-r27c-linux.zip
unzip -q ndk.zip -d "$HOME/tools/ndk"
```

**装 uniffi-bindgen**（`crates.io` 直连 403，需 rsproxy 镜像）：

```bash
export PATH=$HOME/.cargo/bin:$PATH
cargo install uniffi --version 0.32.2 --features cli --root "$HOME/tools/cargo-tools"
```

> ⚠️ **cargo 槽位由 `bin/cargo` 的 `flock` 串行化** ⇒ 不要在同一 slot 并发跑 cargo。

---

## 3. 复现

### 3.1 全部测量（体积 + 耗时 + 依赖增量）

```bash
export PATH=$HOME/.cargo/bin:$PATH
bash ffi/spike/measure.sh all      # 或 sizes / times / deps
```

输出是 `|` 分隔的机器可读行：

```
iOS|aarch64-apple-ios|staticlib .a|lengsuo-spike-uniffi|5942568
Android|aarch64-linux-android|shared .so|lengsuo-spike-uniffi|7477912
TIMING|lengsuo-spike-uniffi|cold|8.72
DEPS|lengsuo-spike-uniffi|runtime_crates=42|direct_deps=2
```

### 3.2 四条跨语言链路真跑一遍

```bash
export PATH=/tmp/swift-6.1.2-RELEASE-ubuntu24.04/usr/bin:$HOME/tools/cargo-tools/bin:$HOME/.cargo/bin:$PATH
bash ffi/spike/run_samples.sh
```

**期望输出**（本卡实测全部退出码 0）：

```
SWIFT_UNIFFI_SELFCHECK_OK=true    Swift×uniffi 退出码=0
SWIFT_CABI_SELFCHECK_OK=true      Swift×CABI 退出码=0
KOTLIN_UNIFFI_SELFCHECK_OK=true   Kotlin×uniffi 退出码=0
KOTLIN_CABI_SELFCHECK_OK=true     Kotlin×CABI 退出码=0
```

脚本会：生成 uniffi 绑定 → 编译 Swift/Kotlin → **真跑**一遍自检 → 打印退出码。

### 3.3 API 演进成本

```bash
bash ffi/spike/measure_evolution.sh
```

自动给 `open_session` 加一个必填参数 → 重新生成绑定 → 统计两侧改动行数 → **自动还原源码**。

### 3.4 Rust 侧测试

```bash
export PATH=$HOME/.cargo/bin:$PATH
cargo test --manifest-path ffi/spike/Cargo.toml --workspace --locked   # 12 + 12 passed
cargo test --manifest-path ffi/Cargo.toml --locked                     # 3 + 19 + 3 passed
#   ↑ 19 = 7 个绑定一致性断言 + 7 个 C ABI 边界单测 + 5 个核心单测
```

---

## 4. 静态检查 vs 真跑

| 目标 | 做到哪一步 | 说明 |
|---|---|---|
| **Swift × uniffi** | ✅ **编译 + 真跑**（Linux host） | `swiftc` 链接 host `.so` 后执行自检 |
| **Swift × 手写 C ABI** | ✅ **编译 + 真跑**（Linux host） | `@_silgen_name` 直连 Rust 导出的符号 |
| **Kotlin × uniffi** | ✅ **编译 + 真跑**（JVM） | host `.so` 代替 Android `.so` |
| **Kotlin × 手写 C ABI** | ✅ **编译 + 真跑**（JVM） | JNA `Native.load` |
| **iOS 真机/模拟器** | ⛔ **无真机验证** | 47 上无 macOS / 无 Xcode |
| **Android 真机/模拟器** | ⛔ **无真机验证** | 只测了 Android `.so` 的**体积**，没跑过 |

⚠️ 上表的「真跑」全部是 **Linux 主机 / JVM** 上的运行，
**不等于** iOS/Android 设备上的验证。

---

## 5. 踩坑记录（省得后人重踩）

### 5.1 uniffi 依赖配置

```toml
# ✅ 正确
uniffi = { version = "0.32.2", default-features = false }

# ❌ 错误：会把整个 codegen 栈（askama/serde_json/toml…）拉进**运行时**依赖
uniffi = { version = "0.32.2", features = ["build"] }
```

⇒ 开了 `"build"` 之后，Android `.so` 从 **7.5 MB 涨到 20.5 MB**（×2.7）。
`"build"` 只有走 **UDL 文件**时才需要；本 spike 用 **proc-macro 模式**，用不到。

### 5.2 uniffi 的 codegen

- `uniffi-bindgen` 会在**当前目录**跑 `cargo metadata` ⇒ 必须在 Rust workspace 里跑。
- 生成的文件名由 **namespace** 决定：`spike.swift` / `spikeFFI.h` / `spikeFFI.modulemap` /
  `uniffi/spike/spike.kt` —— **不是** crate 名。
- Kotlin 生成的包名是 `uniffi.<namespace>`（`uniffi.spike`）。
- 生成的 Kotlin **依赖 JNA** ⇒ `kotlinc -cp` 必须带 `jna.jar`。
- Swift 侧必须提供 **`module.modulemap`**
  （⚠️ **Linux 上文件名就是 `module.modulemap`**，不是 `<平台>FFI.modulemap`），
  且头文件里要 `use "Darwin"` / `use "_Builtin_stdbool"`。
- 生成类型名带前缀：`SpikeSessionInfo` / `SpikeEvent` / `SpikeEventSink`。
- Kotlin 把 `u32`→`UInt`、`u64`→`ULong` ⇒ 与 `Int` 不能直接比较，要显式 `.toUInt()` / `1uL`。

### 5.3 手写 C ABI

- **C 函数指针不能捕获上下文** ⇒ Swift 侧回调必须用
  `nonisolated(unsafe) var` 全局变量 + **不捕获**的蹦床闭包。
- `#[repr(C)]` 结构体加字段 ⇒ **C 头文件 + Swift 镜像 + Kotlin `Structure`** 三处都要改，
  漏改**不报错**（静默错位）。
- `spike_free_session_info` **只**释放内部两个字符串 —— 结构体本身是**调用方的内存**。

### 5.4 JNA 的两个 SIGSEGV 坑（Kotlin 侧实测）

1. `Pointer(long)` 的参数是**地址**，**不是字节数**！
   写 `Pointer(8 * count)` 等于「往地址 0x40 写」⇒ **SIGSEGV**。
   要分配内存必须用 `Memory(size)`。
2. `Structure.setPointer(i, null)` 会对 null 指针**解引用** ⇒
   **JVM 直接 SIGSEGV**（`Java_com_sun_jna_Native_setPointer`），
   **不是**可捕获的异常 ⇒ 整个进程崩。
   ⇒ 不要预先清零指针，让 native 侧填满所有槽位。
3. `Structure.getFieldOrder()` **必须**实现，否则 JNA 报
   "does not provide enough names"。

### 5.5 Android 交叉编译

- Rust **不支持**为 Android 产出 `cdylib`
  （`warning: dropping unsupported crate type 'cdylib'`）
  ⇒ Android 的最终 `.so` 要用 NDK clang 把 Rust 的 `.a` 链出来
  （这正是 Gradle/ndk-build 在生产里做的事）。
- NDK 的 armv7 驱动叫 **`armv7a-...`**（带 `a`），
  而 Rust 三元组是 **`armv7-...`**（不带 `a`）⇒ 要做映射。
- 链接要 `RUSTFLAGS="-C target-feature=+crt-static"`，
  否则报 "Relocations in generic ELF (EM: 183)"。

### 5.6 iOS 交叉编译

- `staticlib` **不需要** macOS SDK ⇒ 47 上能测 iOS `.a` 的**体积**。
- `cdylib` 需要 `xcrun`/macOS SDK ⇒ 47 上**测不了**（如实报告缺失）。

### 5.7 `panic`

`ffi/spike/Cargo.toml` 用 `panic = "unwind"`（**有意**与 `ffi/` 的 `panic = "abort"` 不同），
因为 uniffi 要在 FFI 边界**捕获 panic**。
⚠️ 这是选 uniffi 时**必须**向主理人申请改 `ffi/Cargo.toml` 的原因之一。

---

## 6. ⛔ 边界提醒

- 本目录**只**做 spike 样例与测量，**不写生产逻辑**。
- **不进最终产物**：`ffi/spike/` 是独立 workspace，不参与 `ffi/` 构建。
  `ffi/tests/` 用 `#[path]` 把源文件编进**测试二进制**（不发布）。
- 选定方案后，由**后续卡**删除另一份样例
  （本卡的结论是保留 uniffi、删除 `cabi/` 与两份 C ABI 移动端样例）。
