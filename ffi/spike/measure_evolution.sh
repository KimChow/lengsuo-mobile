#!/usr/bin/env bash
# FFI spike 的 **API 演进成本**测量（测项 7）。
#
# # 测什么
#
# 改**一处** Rust 函数签名，看两种绑定方案下：
#   - Rust 侧要改几行；
#   - 移动端（Swift/Kotlin）要改几行；
#   - 移动端**能不能自动跟上**（uniffi 重新生成 ⇒ 不用改；手写 C ABI ⇒ 必须手改）。
#
# # 怎么测
#
# 选一个**有代表性的**签名改动：给 `open_session` 增加一个**必填参数**
# （`timeout_ms: u64`）。这是协议演进里最常见的一类改动。
#
# 做法：
#   1. 记录「改动前」各文件的行数基线；
#   2. 用脚本把签名改掉（并同步修好编译错误）；
#   3. 重新生成 uniffi 绑定，**用编译器**（swiftc / kotlinc）看移动端**是否要改**：
#      - uniffi：重新生成绑定后，移动端**源码不用改**（只是参数位置变了），
#        但**调用点**要加一个实参 ⇒ 统计调用点改动行数；
#      - 手写 C ABI：**头文件 + 两侧手写声明 + 调用点**都要改。
#   4. 记录「改动后」行数，输出差值。
#   5. ⛔ **还原**（trap 保证一定还原）。
#
# ⛔ 本脚本**只做测量**，最后一定把源码还原成原样。
set -u

export PATH="$HOME/.cargo/bin:$PATH"
SPIKE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${SPIKE_SPIKE_WORK:-/tmp/ffi-spike-run}"
SWIFTC="${SWIFTC:-$(command -v swiftc || true)}"
KOTLINC="${KOTLINC:-$HOME/tools/kotlin/kotlinc/bin/kotlinc}"
JNA_JAR="${JNA_JAR:-/tmp/jna/jna.jar}"
UNIFFI_BINDGEN="${UNIFFI_BINDGEN:-$HOME/tools/cargo-tools/bin/uniffi-bindgen}"
TARGET_DIR="${CARGO_TARGET_DIR:-/home/ubuntu/lengsuo-mobile-dev/targets/mb-ffi-spike}"

# 要改的 Rust 侧文件（一次改动 = 一个签名）。
RUST_CORE="$SPIKE_DIR/core/core.rs"
RUST_CABI="$SPIKE_DIR/cabi/src/lib.rs"
RUST_UNIFFI_API="$SPIKE_DIR/uniffi/src/api.rs"
RUST_UNIFFI_LIB="$SPIKE_DIR/uniffi/src/lib.rs"
C_HEADER="$SPIKE_DIR/cabi/include/lengsuo_spike_cabi.h"

# 备份目录
BAK="$WORK/evo-backup"
rm -rf "$BAK"; mkdir -p "$BAK"
# ⚠️ 备份名必须**唯一**（两个 crate 都叫 lib.rs！）⇒ 用序号。
i=0
for f in "$RUST_CORE" "$RUST_CABI" "$RUST_UNIFFI_API" "$RUST_UNIFFI_LIB" "$C_HEADER"; do
    cp "$f" "$BAK/$i.bak"
    i=$((i + 1))
done
# ⛔ 一定还原
restore() {
    local j=0
    for f in "$RUST_CORE" "$RUST_CABI" "$RUST_UNIFFI_API" "$RUST_UNIFFI_LIB" "$C_HEADER"; do
        [ -f "$BAK/$j.bak" ] && cp "$BAK/$j.bak" "$f"
        j=$((j + 1))
    done
    echo "（已还原源码）"
}
trap restore EXIT

echo "=== 0. 基线行数 ==="
baseline() {
    for f in "$RUST_CORE" "$RUST_CABI" "$RUST_UNIFFI_API" "$RUST_UNIFFI_LIB" "$C_HEADER" \
             "$SPIKE_DIR/swift/SpikeCabiSample.swift" "$SPIKE_DIR/swift/SpikeUniffiSample.swift" \
             "$SPIKE_DIR/kotlin/SpikeCabiSample.kt" "$SPIKE_DIR/kotlin/SpikeUniffiSample.kt"; do
        printf "BASE|%s|%s\n" "$(basename "$f")" "$(wc -l < "$f")"
    done
}
baseline

echo "=== 1. 改签名：open_session 增加必填参数 timeout_ms: u64 ==="

# ---- 1a. 核心层：加参数并透传 ----
python3 - "$RUST_CORE" <<'PY'
import pathlib, sys
p = pathlib.Path(sys.argv[1]); s = p.read_text()
s = s.replace(
    "pub fn open_session(key: &[u8], peer_label: &str) -> Result<SessionInfo, SampleError> {",
    "pub fn open_session(key: &[u8], peer_label: &str, timeout_ms: u64) -> Result<SessionInfo, SampleError> {\n    let _ = timeout_ms;")
p.write_text(s)
PY

# ---- 1b. 手写 C ABI：结构体加字段 + 函数签名加参数 ----
python3 - "$RUST_CABI" <<'PY'
import pathlib, sys
p = pathlib.Path(sys.argv[1]); s = p.read_text()
s = s.replace(
    "pub struct SpikeSessionInfo {\n    epoch: u64,",
    "pub struct SpikeSessionInfo {\n    epoch: u64,\n    timeout_ms: u64,")
s = s.replace(
    "pub fn spike_open_session(",
    "pub fn spike_open_session(")
# 在参数列表尾部加 timeout_ms
s = s.replace(
    "    peer_label: *const c_char,\n    out_info: *mut SpikeSessionInfo,",
    "    peer_label: *const c_char,\n    timeout_ms: u64,\n    out_info: *mut SpikeSessionInfo,")
s = s.replace(
    "core_api::open_session(key_bytes, &label)",
    "core_api::open_session(key_bytes, &label, timeout_ms)")
p.write_text(s)
PY

# ---- 1c. uniffi 语义层 + 宏层：加参数 ----
python3 - "$RUST_UNIFFI_API" "$RUST_UNIFFI_LIB" <<'PY'
import pathlib, sys
api = pathlib.Path(sys.argv[1]); s = api.read_text()
s = s.replace(
    "pub fn open_session(key: Vec<u8>, peer_label: String) -> Result<SessionInfo, SpikeErrorKind> {",
    "pub fn open_session(\n    key: Vec<u8>,\n    peer_label: String,\n    timeout_ms: u64,\n) -> Result<SessionInfo, SpikeErrorKind> {")
s = s.replace("core_api::open_session(&key, &peer_label)",
              "core_api::open_session(&key, &peer_label, timeout_ms)")
api.write_text(s)

lib = pathlib.Path(sys.argv[2]); s = lib.read_text()
s = s.replace(
    "pub fn open_session(key: Vec<u8>, peer_label: String) -> Result<SpikeSessionInfo, SpikeError> {",
    "pub fn open_session(\n    key: Vec<u8>,\n    peer_label: String,\n    timeout_ms: u64,\n) -> Result<SpikeSessionInfo, SpikeError> {")
s = s.replace("api::open_session(key, peer_label)", "api::open_session(key, peer_label, timeout_ms)")
lib.write_text(s)
PY

# ---- 1d. 手写 C 头文件：同步加参数 ----
python3 - "$C_HEADER" <<'PY'
import pathlib, sys
p = pathlib.Path(sys.argv[1]); s = p.read_text()
s = s.replace(
    "int32_t spike_open_session(const uint8_t *key, size_t key_len, const char *peer_label,",
    "int32_t spike_open_session(const uint8_t *key, size_t key_len, const char *peer_label,\n                            uint64_t timeout_ms,")
p.write_text(s)
PY

echo "=== 2. 改动后行数 ==="
baseline

echo "=== 3. Rust 侧还能编过吗 ==="
( cd "$SPIKE_DIR" && cargo build --release -p lengsuo-spike-cabi -p lengsuo-spike-uniffi 2>&1 | tail -3 )

echo "=== 4. 移动端：uniffi 重新生成绑定后，**调用点**要改几行 ==="
GEN="$WORK/evo-gen"; rm -rf "$GEN"; mkdir -p "$GEN"
( cd "$SPIKE_DIR" && cargo build --release -p lengsuo-spike-uniffi ) >/dev/null 2>&1
( cd "$SPIKE_DIR" && "$UNIFFI_BINDGEN" generate --library --language swift --language kotlin \
    --out-dir "$GEN" "$TARGET_DIR/release/liblengsuo_spike_uniffi.so" ) >/dev/null 2>&1
echo "生成的 Swift 签名："
grep -n "public func openSession" "$GEN/spike.swift" 2>/dev/null || echo "（生成失败）"
echo "生成的 Kotlin 签名："
grep -n "fun \`openSession\`" "$GEN/uniffi/spike/spike.kt" 2>/dev/null || echo "（生成失败）"

# Swift：把样例改成新签名，看要改几行
cp "$SPIKE_DIR/swift/SpikeUniffiSample.swift" "$WORK/evo-swift-uniffi.swift"
python3 - "$WORK/evo-swift-uniffi.swift" <<'PY'
import pathlib, sys
p = pathlib.Path(sys.argv[1]); s = p.read_text()
s = s.replace("return try openSession(key: key, peerLabel: \"peer-a\")",
              "return try openSession(key: key, peerLabel: \"peer-a\", timeoutMs: 5_000)")
s = s.replace("_ = try openSession(key: shortKey, peerLabel: \"peer-a\")",
              "_ = try openSession(key: shortKey, peerLabel: \"peer-a\", timeoutMs: 5_000)")
p.write_text(s)
PY
echo "uniffi · Swift 调用点改动行数：$(diff <(cat "$SPIKE_DIR/swift/SpikeUniffiSample.swift") "$WORK/evo-swift-uniffi.swift" | grep -c '^>')"

cp "$SPIKE_DIR/kotlin/SpikeUniffiSample.kt" "$WORK/evo-kotlin-uniffi.kt"
python3 - "$WORK/evo-kotlin-uniffi.kt" <<'PY'
import pathlib, sys
p = pathlib.Path(sys.argv[1]); s = p.read_text()
s = s.replace('return openSession(key, "peer-a")', 'return openSession(key, "peer-a", 5_000uL)')
s = s.replace('openSession(shortKey, "peer-a")', 'openSession(shortKey, "peer-a", 5_000uL)')
p.write_text(s)
PY
echo "uniffi · Kotlin 调用点改动行数：$(diff <(cat "$SPIKE_DIR/kotlin/SpikeUniffiSample.kt") "$WORK/evo-kotlin-uniffi.kt" | grep -c '^>')"

echo "=== 5. 移动端：手写 C ABI 要改几行 ==="
cp "$SPIKE_DIR/swift/SpikeCabiSample.swift" "$WORK/evo-swift-cabi.swift"
python3 - "$WORK/evo-swift-cabi.swift" <<'PY'
import pathlib, sys
p = pathlib.Path(sys.argv[1]); s = p.read_text()
s = s.replace("    _ peerLabel: UnsafePointer<CChar>?,\n",
              "    _ peerLabel: UnsafePointer<CChar>?,\n    _ timeoutMs: UInt64,\n")
s = s.replace("keyBuf.baseAddress, keyBuf.count, label, &info, &error",
              "keyBuf.baseAddress, keyBuf.count, label, 5_000, &info, &error")
p.write_text(s)
PY
echo "手写 C ABI · Swift 声明改动行数：$(diff <(cat "$SPIKE_DIR/swift/SpikeCabiSample.swift") "$WORK/evo-swift-cabi.swift" | grep -c '^>')"

cp "$SPIKE_DIR/kotlin/SpikeCabiSample.kt" "$WORK/evo-kotlin-cabi.kt"
python3 - "$WORK/evo-kotlin-cabi.kt" <<'PY'
import pathlib, sys
p = pathlib.Path(sys.argv[1]); s = p.read_text()
s = s.replace("        peerLabel: String?,\n",
              "        peerLabel: String?,\n        timeoutMs: Long,\n")
s = s.replace('spike_open_session(key, key.size, "peer-a", info, error)',
              'spike_open_session(key, key.size, "peer-a", 5_000L, info, error)')
p.write_text(s)
PY
echo "手写 C ABI · Kotlin 声明改动行数：$(diff <(cat "$SPIKE_DIR/kotlin/SpikeCabiSample.kt") "$WORK/evo-kotlin-cabi.kt" | grep -c '^>')"

echo "=== 6. 结论：谁需要『手工同步』 ==="
cat <<'TXT'
uniffi      ：绑定**自动重新生成**（含新的参数、新的结构体字段）；
              移动端**只改调用点**（加一个实参）⇒ 改 1 处/语言。
              编译器会**主动报错**指出所有漏改的调用点。
手写 C ABI  ：**头文件 + 两侧手写声明 + 调用点**都要手工同步；
              结构体加字段 ⇒ 两侧的 `#[repr(C)]` 镜像都要改，
              且**改错不报错**（静默错位）⇒ 风险显著更高。
TXT
