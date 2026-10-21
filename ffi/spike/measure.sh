#!/usr/bin/env bash
# FFI spike 的**测量脚本**：产物体积 + 构建耗时（冷/增量）+ 依赖增量。
#
# ⛔ 本脚本只做**测量**，不改任何源码。
# ⛔ 产物落在 $CARGO_TARGET_DIR 下，**不入库**。
#
# 用法：
#   export PATH=$HOME/.cargo/bin:$PATH
#   bash ffi/spike/measure.sh            # 全部
#   bash ffi/spike/measure.sh sizes      # 只测体积
#   bash ffi/spike/measure.sh times      # 只测耗时
#   bash ffi/spike/measure.sh deps       # 只测依赖增量
set -u

export PATH="$HOME/.cargo/bin:$PATH"
SPIKE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASE_TARGET_DIR="${CARGO_TARGET_DIR:-/home/ubuntu/lengsuo-mobile-dev/targets/mb-ffi-spike}"
MEASURE_DIR="$BASE_TARGET_DIR/spike-measure"

# NDK（Android 交叉编译需要 clang；没装则 Android 部分如实报缺失）
NDK_ROOT="${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-$HOME/tools/ndk/android-ndk-r27c}}"
NDK_BIN="$NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin"
# Android 最低 API level（ndk 的 clang 包装脚本按这个数字找 sysroot）。
ANDROID_API="${ANDROID_API:-21}"

WHAT="${1:-all}"

# ---------------------------------------------------------------- 公共函数

# lib_name <package>：crate 的 lib 名（产物文件名用这个，下划线形式）
lib_name() {
    case "$1" in
        lengsuo-spike-uniffi) echo "lengsuo_spike_uniffi" ;;
        lengsuo-spike-cabi)   echo "lengsuo_spike_cabi" ;;
        *) echo "$1" ;;
    esac
}

# size_of <文件>：返回字节数（文件不存在则返回 "MISSING"）
size_of() {
    if [ -f "$1" ]; then
        stat -c %s "$1"
    else
        echo "MISSING"
    fi
}

# android_env <abi>：该 ABI 的 Rust 目标三元组
android_env() {
    case "$1" in
        aarch64) echo "aarch64-linux-android" ;;
        armv7)  echo "armv7-linux-androideabi" ;;
        x86_64) echo "x86_64-linux-android" ;;
        *)      return 1 ;;
    esac
}

# ndk_clang <triple>：该三元组对应的 NDK clang 包装脚本
# ⚠️ NDK 的 armv7 驱动叫 `armv7a-...`（带 a），Rust 三元组叫 `armv7-...`（不带 a）。
ndk_clang() {
    local triple="$1"
    case "$triple" in
        armv7-linux-androideabi) triple="armv7a-linux-androideabi" ;;
    esac
    echo "$NDK_BIN/${triple}${ANDROID_API}-clang"
}

# build_android_lib <package> <abi> <out_dir>
#   用 NDK 的 clang 交叉编译出 Android 静态库（.a）。
#   ⚠️ Rust **不支持**为 Android 产出 cdylib（`warning: dropping unsupported crate type`），
#      所以 Android 的最终产物 .so 由 NDK clang 把 .a 链出来（见 link_android_so）。
build_android_lib() {
    local pkg="$1" abi="$2" out="$3"
    local triple clang env_var
    triple="$(android_env "$abi")"
    clang="$(ndk_clang "$triple")"
    if [ ! -x "$clang" ]; then
        echo "NDK_MISSING"
        return 1
    fi
    env_var="CARGO_TARGET_$(echo "$triple" | tr 'a-z-' 'A-Z_')_LINKER"
    ( cd "$SPIKE_DIR" && env \
        RUSTFLAGS="-C target-feature=+crt-static" \
        "$env_var=$clang" \
        CARGO_TARGET_DIR="$out" \
        cargo build --release --target "$triple" -p "$pkg" ) >/dev/null 2>&1
}

# link_android_so <triple> <staticlib> <out_so>
#   用 NDK clang 把 Rust 静态库链成 Android 共享库（.so）。
#   —— 这正是 Gradle/ndk-build 在生产里做的事（Rust .a → .so）。
link_android_so() {
    local triple="$1" lib="$2" out="$3"
    local clang shim
    clang="$(ndk_clang "$triple")"
    [ -x "$clang" ] || { echo "NDK_MISSING"; return 1; }
    shim="$(dirname "$out")/shim_$triple.c"
    # 极简 shim：引用一个 Rust 导出符号，保证 .a 不被整体裁掉。
    cat > "$shim" <<'EOF'
#include <stdint.h>
extern int32_t spike_binding_version(void);
int32_t shim_probe(void) { return spike_binding_version(); }
EOF
    "$clang" -shared -fPIC -O2 \
        -Wl,--whole-archive "$lib" -Wl,--no-whole-archive \
        -o "$out" "$shim" -llog -ldl -lm 2>/dev/null
}

# ---------------------------------------------------------------- 体积测量

measure_sizes() {
    echo "=== 体积测量（release：opt-level=z, lto=true, strip=debuginfo）==="

    for pkg in lengsuo-spike-uniffi lengsuo-spike-cabi; do
        label="$pkg"

        # ---- iOS aarch64 staticlib ----
        # ⚠️ 只能出 .a（cdylib/.xcframework 需要 macOS SDK，见报告）。
        ios_out="$MEASURE_DIR/ios-$pkg"
        rm -rf "$ios_out"
        ( cd "$SPIKE_DIR" && CARGO_TARGET_DIR="$ios_out" \
            cargo rustc --release --target aarch64-apple-ios -p "$pkg" \
            --crate-type staticlib ) >/dev/null 2>&1
        ios_a=$(size_of "$ios_out/aarch64-apple-ios/release/lib$(lib_name "$pkg").a")
        echo "iOS|aarch64-apple-ios|staticlib .a|$label|${ios_a}"

        # ---- Android 各 ABI ----
        for abi in aarch64 armv7 x86_64; do
            triple="$(android_env "$abi")"
            and_out="$MEASURE_DIR/and-$abi-$pkg"
            rm -rf "$and_out"
            out="$(build_android_lib "$pkg" "$abi" "$and_out")"
            if [ "$out" = "NDK_MISSING" ]; then
                echo "Android|$triple|staticlib .a|$label|NDK_MISSING"
                echo "Android|$triple|shared .so|$label|NDK_MISSING"
                continue
            fi
            a_path="$and_out/$triple/release/lib$(lib_name "$pkg").a"
            and_a=$(size_of "$a_path")
            echo "Android|$triple|staticlib .a|$label|${and_a}"
            so_path="$MEASURE_DIR/lib$(lib_name "$pkg")_$triple.so"
            if link_android_so "$triple" "$a_path" "$so_path" >/dev/null 2>&1; then
                and_so=$(size_of "$so_path")
                echo "Android|$triple|shared .so|$label|${and_so}"
            else
                echo "Android|$triple|shared .so|$label|LINK_FAILED"
            fi
        done
    done
}

# ---------------------------------------------------------------- 耗时测量

# elapsed <命令...>：打印耗时（秒）
elapsed() {
    local start end
    start=$(date +%s.%N)
    "$@" >/dev/null 2>&1
    end=$(date +%s.%N)
    python3 -c "print(f'{$end - $start:.2f}')"
}

measure_times() {
    echo "=== 构建耗时（秒，release）==="

    for pkg in lengsuo-spike-uniffi lengsuo-spike-cabi; do
        out="$MEASURE_DIR/time-$pkg"
        rm -rf "$out"
        # 冷构建：目标目录为空（在 spike workspace 里构建）
        cold=$(elapsed bash -c "cd '$SPIKE_DIR' && CARGO_TARGET_DIR='$out' cargo build --release -p '$pkg'")
        # 增量构建：同一目标目录、无源码改动
        incr=$(elapsed bash -c "cd '$SPIKE_DIR' && CARGO_TARGET_DIR='$out' cargo build --release -p '$pkg'")
        echo "TIMING|$pkg|cold|${cold}"
        echo "TIMING|$pkg|incremental|${incr}"
    done
}

# ---------------------------------------------------------------- 依赖增量

measure_deps() {
    echo "=== 依赖增量（normal 边上的第三方 crate 数 / 直接依赖数）==="
    for pkg in lengsuo-spike-uniffi lengsuo-spike-cabi; do
        # normal 边 = 会被链进**运行时**产物的依赖（build-dep 不算）。
        tree="$( cd "$SPIKE_DIR" && cargo tree -p "$pkg" --edges normal --prefix none --no-dedupe 2>/dev/null )"
        total=$(printf '%s\n' "$tree" | sed 's/ (.*//' | grep -v "^$pkg " | sort -u | grep -c . || true)
        # 直接依赖数（读 Cargo.toml 的 [dependencies]）。
        manifest="$( cd "$SPIKE_DIR" && cargo metadata --format-version 1 --no-deps 2>/dev/null \
            | python3 -c "
import json, sys
md = json.load(sys.stdin)
for p in md['packages']:
    if p['name'] == '$pkg':
        print(p['manifest_path'])
        break
" )"
        direct=$(python3 -c "
import pathlib, sys, tomllib
data = tomllib.loads(pathlib.Path(sys.argv[1]).read_text())
print(len(data.get('dependencies', {})))
" "$manifest")
        echo "DEPS|$pkg|runtime_crates=${total}|direct_deps=${direct}"
    done
}

# ---------------------------------------------------------------- 入口

case "$WHAT" in
    sizes) measure_sizes ;;
    times) measure_times ;;
    deps)  measure_deps ;;
    all)   measure_sizes; measure_times; measure_deps ;;
    *)     echo "用法: $0 [sizes|times|deps|all]" >&2; exit 2 ;;
esac
