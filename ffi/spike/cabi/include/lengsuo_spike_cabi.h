/* FFI spike：**手写 C ABI** 方案的头文件（**对照**）。
 *
 * ⛔ 本文件是**样例**，不是生产代码。
 *
 * ⚠️ 这就是手写 C ABI 的成本之一：**头文件必须人工维护**，
 *    且与 Rust 侧的 `#[repr(C)]` 结构体**逐字段对齐**。
 *    Rust 侧改了结构体而忘了改这里 ⇒ **静默错位**（编译器不报错）。
 *    uniffi 那边是**自动生成**头文件（`lengsuo_spike_uniffiFFI.h`）。
 */
#ifndef LENGSUO_SPIKE_CABI_H
#define LENGSUO_SPIKE_CABI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/** 错误码：与 Rust 侧 `SpikeErrorCode` 逐值对应。 */
typedef int32_t SpikeErrorCode;

#define SPIKE_ERROR_UNKNOWN 0
#define SPIKE_ERROR_KEY_LENGTH 1
#define SPIKE_ERROR_NOT_HANDSHAKED 2
#define SPIKE_ERROR_INVALID_CAPABILITY 3
#define SPIKE_ERROR_INVALID_PEER_LABEL 4

/** 会话信息：与 Rust 侧 `#[repr(C)] struct SpikeSessionInfo` 逐字段对齐。 */
typedef struct SpikeSessionInfo {
    uint64_t epoch;
    uint32_t capabilities;
    uint64_t fingerprint;
    /** 由库分配的 NUL 结尾字符串；用 `spike_free_session_info` 释放。 */
    char *session_id;
    /** 由库分配的 NUL 结尾字符串；用 `spike_free_session_info` 释放。 */
    char *peer_label;
} SpikeSessionInfo;

/** 错误载荷：错误码 + 可为 NULL 的消息。 */
typedef struct SpikeError {
    SpikeErrorCode code;
    /** 可为 NULL；用 `spike_free_string` 释放。 */
    char *message;
} SpikeError;

/** 事件回调：Rust 调移动端。 */
typedef void (*SpikeEventCallback)(uint64_t seq, const char *kind, const char *payload);

/* ---- 生命周期 ---- */

/** 释放一个由库分配的字符串。 */
void spike_free_string(char *ptr);

/** 释放 `SpikeSessionInfo` 内部的两个字符串（**不**释放结构体本身）。 */
void spike_free_session_info(SpikeSessionInfo *ptr);

/** 释放能力名数组。 */
void spike_free_string_array(char **names, size_t count);

/* ---- 业务入口 ---- */

/**
 * 打开样例会话。
 * @return 0 = 成功；非 0 = 失败（`out_error` 被填充）。
 */
int32_t spike_open_session(const uint8_t *key, size_t key_len, const char *peer_label,
                            SpikeSessionInfo *out_info, SpikeError *out_error);

/** 能力名列表（两段式：先问数量，再取名字）。 */
int32_t spike_capability_names(uint32_t mask, char **out_names, size_t *out_count,
                               SpikeError *out_error);

/** 能力名数量。 */
size_t spike_capability_name_count(uint32_t mask);

/** 触发样例事件（同步回调，在调用线程上）。 */
int32_t spike_emit_events(uint32_t count, SpikeEventCallback callback);

/** 绑定版本号（库分配，用 `spike_free_string` 释放）。 */
char *spike_binding_version(void);

#ifdef __cplusplus
}
#endif

#endif /* LENGSUO_SPIKE_CABI_H */
