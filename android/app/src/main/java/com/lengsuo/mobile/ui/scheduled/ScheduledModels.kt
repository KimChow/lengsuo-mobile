package com.lengsuo.mobile.ui.scheduled

import com.lengsuo.mobile.R

/**
 * 定时任务页本地模型与内存仓库。
 * 本卡只做静态布局 + 本地内存增删；真实调度、持久化、详情页属后续卡。
 */
enum class ScheduleFrequency(val labelRes: Int) {
    DAILY(R.string.sched_freq_daily),
    EVERY_30_MIN(R.string.sched_freq_30m),
    EVERY_1_HOUR(R.string.sched_freq_1h),
    EVERY_2_HOUR(R.string.sched_freq_2h),
    EVERY_4_HOUR(R.string.sched_freq_4h),
    EVERY_8_HOUR(R.string.sched_freq_8h),
    BIWEEKLY(R.string.sched_freq_biweekly),
}

/** 表单 chips 的展示顺序（每天在前，间隔档升序，双周殿后）。 */
val FREQUENCY_ORDER: List<ScheduleFrequency> = ScheduleFrequency.entries.toList()

val DEFAULT_FREQUENCY: ScheduleFrequency = ScheduleFrequency.DAILY

/** 底部弹层三选项，顺序对齐参考端：自定义添加 / 以对话方式添加 / 取消。 */
enum class ScheduleAddOption(val labelRes: Int) {
    CUSTOM_ADD(R.string.sched_sheet_custom_add),
    ADD_VIA_CHAT(R.string.sched_sheet_add_via_chat),
    CANCEL(R.string.sched_sheet_cancel),
}

data class ScheduledTask(
    val id: Long,
    val name: String,
    val prompt: String,
    val frequency: ScheduleFrequency,
)

/** 表单校验：名称必填（非空白）才可创建。 */
fun isFormCreatable(name: String): Boolean = name.isNotBlank()

/** 本地内存仓库：新增任务插到列表首位，支持按 id 移除。 */
class ScheduledStore(initialTasks: List<ScheduledTask> = emptyList()) {
    private val items = initialTasks.toMutableList()
    private var nextId: Long = (initialTasks.maxOfOrNull { it.id } ?: 0L) + 1L

    val tasks: List<ScheduledTask>
        get() = items.toList()

    fun add(name: String, prompt: String, frequency: ScheduleFrequency): ScheduledTask {
        val task = ScheduledTask(
            id = nextId++,
            name = name,
            prompt = prompt,
            frequency = frequency,
        )
        items.add(0, task)
        return task
    }

    fun remove(taskId: Long) {
        items.removeAll { it.id == taskId }
    }
}
