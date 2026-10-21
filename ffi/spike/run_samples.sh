#!/usr/bin/env bash
# FFI spike 的**端到端样例运行器**：生成绑定 → 编译 → 真跑一次自检。
#
# ⛔ 只跑样例，不改任何源码。
# ⛔ 产物落在 $WORK（默认 /tmp/ffi-spike-run），**不入库**。
#
# 覆盖 4 条链路：
#   1. Swift  × uniffi
#   2. Swift  × 手写 C ABI
#   3. Kotlin × uniffi
#   4. Kotlin × 手写 C ABI
#
# ⚠️ 全部是 **Linux 主机上的静态/动态链接自检**，
#    **不是** iOS 设备 / Android 设备上的真机验证。
#
# 用法：
#   export PATH=$HOME/.cargo/bin:$PATH
#   bash ffi/spike/run_samples.sh
set -u

export PATH="$HOME/.cargo/bin:$PATH"
SPIKE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${SPIKE_SPIKE_WORK:-/tmp/ffi-spike-run}"
# 产物目录：与 measure.sh 用同一个（CARGO_TARGET_DIR 由环境给）
TARGET_DIR="${CARGO_TARGET_DIR:-/home/ubuntu/lengsuo-mobile-dev/targets/mb-ffi-spike}"
LIB_DIR="$TARGET_DIR/release"

# ---- 外部工具（缺了就如实报缺失，不伪造） ----
SWIFTC="${SWIFTC:-$(command -v swiftc || true)}"
KOTLINC="${KOTLINC:-$HOME/tools/kotlin/kotlinc/bin/kotlinc}"
JNA_JAR="${JNA_JAR:-/tmp/jna/jna.jar}"
UNIFFI_BINDGEN="${UNIFFI_BINDGEN:-$HOME/tools/cargo-tools/bin/uniffi-bindgen}"

rm -rf "$WORK"
mkdir -p "$WORK"

echo "=== 工具链 ==="
echo "swiftc  : ${SWIFTC:-缺失}"
echo "kotlinc : $([ -x "$KOTLINC" ] && echo "$KOTLINC" || echo 缺失)"
echo "jna     : $([ -f "$JNA_JAR" ] && echo "$JNA_JAR" || echo 缺失)"
echo "bindgen : $([ -x "$UNIFFI_BINDGEN" ] && echo "$UNIFFI_BINDGEN" || echo 缺失)"

# ================================================================ 1. uniffi 绑定
# 生成 Swift / Kotlin 绑定（从 cdylib 的元数据里读）
GEN="$WORK/uniffi-gen"
mkdir -p "$GEN"

if [ -x "$UNIFFI_BINDGEN" ]; then
    ( cd "$SPIKE_DIR" && cargo build --release -p lengsuo-spike-uniffi ) >/dev/null 2>&1
    UNIFFI_CDYLIB="$LIB_DIR/liblengsuo_spike_uniffi.so"
    echo "=== 生成 uniffi 绑定（cdylib: $UNIFFI_CDYLIB）==="
    # ⚠️ uniffi-bindgen 会在 **当前目录** 跑 `cargo metadata`
    #    ⇒ 必须在 spike workspace 里执行。
    ( cd "$SPIKE_DIR" && "$UNIFFI_BINDGEN" generate --library \
        --language swift --language kotlin \
        --out-dir "$GEN" "$UNIFFI_CDYLIB" ) 2>&1 | tail -5
    ls -l "$GEN" 2>/dev/null
else
    echo "=== 跳过 uniffi 绑定生成：uniffi-bindgen 缺失 ==="
fi

# ================================================================ 2. Swift × uniffi
SW_DIR="$WORK/swift-uniffi"
mkdir -p "$SW_DIR"
if [ -f "$GEN/spike.swift" ] && [ -n "$SWIFTC" ]; then
    cp "$GEN/spike.swift" "$GEN/spikeFFI.h" "$GEN/spikeFFI.modulemap" \
       "$SPIKE_DIR/swift/SpikeUniffiSample.swift" "$SW_DIR/"
    # 生成物里的 module 名是 spikeFFI（namespace 决定）。
    # ⚠️ Swift 在 **Linux** 上找的是 `module.modulemap`（不是 `<平台>FFI.modulemap`），
    #    且 header 需要显式 `use` Darwin/builtin 头。
    cat > "$SW_DIR/module.modulemap" <<'EOF'
module spikeFFI {
    header "spikeFFI.h"
    export *
    use "Darwin"
    use "_Builtin_stdbool"
    use "_Builtin_stdint"
}
EOF
    cat > "$SW_DIR/main.swift" <<'EOF'
import Glibc
let ok = SpikeSelfCheck.run()
print("SWIFT_UNIFFI_SELFCHECK_OK=\(ok)")
exit(ok ? 0 : 1)
EOF
    echo "=== Swift × uniffi ==="
    ( cd "$SW_DIR" && "$SWIFTC" -o spikecheck \
        main.swift SpikeUniffiSample.swift spike.swift \
        -I . -L . -Xlinker -rpath -Xlinker . \
        -L "$LIB_DIR" -llengsuo_spike_uniffi 2>&1 | tail -20 )
    if [ -x "$SW_DIR/spikecheck" ]; then
        ( cd "$SW_DIR" && LD_LIBRARY_PATH="$LIB_DIR" ./spikecheck )
        echo "Swift×uniffi 退出码=$?"
    else
        echo "Swift×uniffi 构建失败"
    fi
else
    echo "=== 跳过 Swift×uniffi：绑定或 swiftc 缺失 ==="
fi

# ================================================================ 3. Swift × 手写 C ABI
CABI_SW_DIR="$WORK/swift-cabi"
mkdir -p "$CABI_SW_DIR"
if [ -n "$SWIFTC" ]; then
    ( cd "$SPIKE_DIR" && cargo build --release -p lengsuo-spike-cabi ) >/dev/null 2>&1
    cp "$SPIKE_DIR/cabi/include/lengsuo_spike_cabi.h" \
       "$SPIKE_DIR/swift/SpikeCabiSample.swift" "$CABI_SW_DIR/"
    cat > "$CABI_SW_DIR/module.modulemap" <<'EOF'
module lengsuo_spike_cabi {
    header "lengsuo_spike_cabi.h"
    export *
}
EOF
    cat > "$CABI_SW_DIR/main.swift" <<'EOF'
import Glibc
let ok = SpikeCabiSelfCheck.run()
print("SWIFT_CABI_SELFCHECK_OK=\(ok)")
exit(ok ? 0 : 1)
EOF
    echo "=== Swift × 手写 C ABI ==="
    ( cd "$CABI_SW_DIR" && "$SWIFTC" -o cabicheck \
        main.swift SpikeCabiSample.swift \
        -I . -L "$LIB_DIR" -llengsuo_spike_cabi 2>&1 | tail -20 )
    if [ -x "$CABI_SW_DIR/cabicheck" ]; then
        ( cd "$CABI_SW_DIR" && LD_LIBRARY_PATH="$LIB_DIR" ./cabicheck )
        echo "Swift×CABI 退出码=$?"
    else
        echo "Swift×CABI 构建失败"
    fi
else
    echo "=== 跳过 Swift×CABI：swiftc 缺失 ==="
fi

# ================================================================ 4. Kotlin × uniffi
KT_DIR="$WORK/kotlin-uniffi"
mkdir -p "$KT_DIR"
if [ -f "$GEN/uniffi/spike/spike.kt" ] && [ -x "$KOTLINC" ]; then
    cp "$GEN/uniffi/spike/spike.kt" "$SPIKE_DIR/kotlin/SpikeUniffiSample.kt" "$KT_DIR/"
    cat > "$KT_DIR/Main.kt" <<'EOF'
import uniffi.spike.SpikeSelfCheck
fun main() {
    val ok = SpikeSelfCheck.run()
    println("KOTLIN_UNIFFI_SELFCHECK_OK=$ok")
    if (!ok) kotlin.system.exitProcess(1)
}
EOF
    echo "=== Kotlin × uniffi ==="
    # ⚠️ uniffi 生成的 Kotlin **依赖 JNA**（`Native`/`Pointer`/`Structure`）⇒ 必须带上 JNA jar。
    ( cd "$KT_DIR" && "$KOTLINC" -nowarn -cp ".:$JNA_JAR" -include-runtime -d spike.jar \
        Main.kt SpikeUniffiSample.kt spike.kt 2>&1 | tail -20 )
    if [ -f "$KT_DIR/spike.jar" ]; then
        ( cd "$KT_DIR" && LD_LIBRARY_PATH="$LIB_DIR" \
            java -Djava.library.path="$LIB_DIR" \
                 -cp "spike.jar:$JNA_JAR" MainKt )
        echo "Kotlin×uniffi 退出码=$?"
    else
        echo "Kotlin×uniffi 构建失败"
    fi
else
    echo "=== 跳过 Kotlin×uniffi：绑定或 kotlinc 缺失 ==="
fi

# ================================================================ 5. Kotlin × 手写 C ABI
KT_CABI_DIR="$WORK/kotlin-cabi"
mkdir -p "$KT_CABI_DIR"
if [ -x "$KOTLINC" ] && [ -f "$JNA_JAR" ]; then
    cp "$SPIKE_DIR/kotlin/SpikeCabiSample.kt" "$KT_CABI_DIR/"
    cat > "$KT_CABI_DIR/Main.kt" <<'EOF'
import spike.cabi.SpikeCabiSelfCheck
fun main() {
    val ok = SpikeCabiSelfCheck.run()
    println("KOTLIN_CABI_SELFCHECK_OK=$ok")
    if (!ok) kotlin.system.exitProcess(1)
}
EOF
    echo "=== Kotlin × 手写 C ABI ==="
    ( cd "$KT_CABI_DIR" && "$KOTLINC" -nowarn -cp "$JNA_JAR" -include-runtime -d spike.jar \
        Main.kt SpikeCabiSample.kt 2>&1 | tail -20 )
    if [ -f "$KT_CABI_DIR/spike.jar" ]; then
        ( cd "$KT_CABI_DIR" && LD_LIBRARY_PATH="$LIB_DIR" \
            java -Djava.library.path="$LIB_DIR" \
                 -cp "spike.jar:$JNA_JAR" MainKt )
        echo "Kotlin×CABI 退出码=$?"
    else
        echo "Kotlin×CABI 构建失败"
    fi
else
    echo "=== 跳过 Kotlin×CABI：kotlinc 或 jna 缺失 ==="
fi
