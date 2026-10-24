package com.lengsuo.mobile

import com.lengsuo.mobile.ui.scheduled.DEFAULT_FREQUENCY
import com.lengsuo.mobile.ui.scheduled.FREQUENCY_ORDER
import com.lengsuo.mobile.ui.scheduled.ScheduleAddOption
import com.lengsuo.mobile.ui.scheduled.ScheduleFrequency
import com.lengsuo.mobile.ui.scheduled.ScheduledStore
import com.lengsuo.mobile.ui.scheduled.isFormCreatable

/**
 * 定时任务页纯数据断言。
 * 注：app 模块没有任何 testImplementation（本卡禁增 Gradle 依赖），故不引 junit；
 * 断言以纯 Kotlin 写成，main() 逐条执行，全部通过时退出码 0，否则退出码 1。
 */
class ScheduledPageTest {
    private val failures = mutableListOf<String>()

    private fun check(name: String, condition: Boolean) {
        if (!condition) failures += name
    }

    fun frequency_hasSevenOptionsInSpecOrder() {
        val expected = listOf(
            R.string.sched_freq_daily,
            R.string.sched_freq_30m,
            R.string.sched_freq_1h,
            R.string.sched_freq_2h,
            R.string.sched_freq_4h,
            R.string.sched_freq_8h,
            R.string.sched_freq_biweekly,
        )
        check(
            "频率共 7 档, actual ${FREQUENCY_ORDER.size}",
            FREQUENCY_ORDER.size == 7,
        )
        check(
            "FREQUENCY_ORDER 顺序 = 每天/30分/1h/2h/4h/8h/双周",
            FREQUENCY_ORDER.map { it.labelRes } == expected,
        )
    }

    fun frequency_defaultIsDaily() {
        check(
            "默认频率 = 每天",
            DEFAULT_FREQUENCY == ScheduleFrequency.DAILY,
        )
    }

    fun addSheet_optionsInSpecOrder() {
        val expected = listOf(
            R.string.sched_sheet_custom_add,
            R.string.sched_sheet_add_via_chat,
            R.string.sched_sheet_cancel,
        )
        check(
            "弹层 3 项顺序 = 自定义添加/以对话方式添加/取消",
            ScheduleAddOption.entries.map { it.labelRes } == expected,
        )
    }

    fun form_validationRequiresNonBlankName() {
        check("空名称不可创建", !isFormCreatable(""))
        check("纯空格名称不可创建", !isFormCreatable("   "))
        check("非空名称可创建", isFormCreatable("测试任务 A"))
    }

    fun store_addPutsNewTaskFirst_andRemoveWorks() {
        val store = ScheduledStore()
        val first = store.add(name = "任务一", prompt = "内容一", frequency = ScheduleFrequency.DAILY)
        val second = store.add(name = "任务二", prompt = "内容二", frequency = ScheduleFrequency.BIWEEKLY)
        check("新增两条后仓库共 2 条", store.tasks.size == 2)
        check("新任务出现在列表首位", store.tasks.first() == second)
        check("首位之外是先创建的任务", store.tasks.last() == first)
        check("任务字段完整", first.name == "任务一" && first.prompt == "内容一")
        store.remove(second.id)
        check("移除新任务后剩 1 条且为先创建的", store.tasks.size == 1 && store.tasks.single() == first)
        store.remove(first.id)
        check("全部移除后仓库为空", store.tasks.isEmpty())
        store.remove(first.id)
        check("重复移除不抛异常", store.tasks.isEmpty())
    }

    fun resources_labelResBoundToSchedStrings() {
        check(
            "频率档绑定 sched_freq_* 资源名",
            ScheduleFrequency.DAILY.labelRes == R.string.sched_freq_daily &&
                ScheduleFrequency.EVERY_30_MIN.labelRes == R.string.sched_freq_30m &&
                ScheduleFrequency.EVERY_1_HOUR.labelRes == R.string.sched_freq_1h &&
                ScheduleFrequency.EVERY_2_HOUR.labelRes == R.string.sched_freq_2h &&
                ScheduleFrequency.EVERY_4_HOUR.labelRes == R.string.sched_freq_4h &&
                ScheduleFrequency.EVERY_8_HOUR.labelRes == R.string.sched_freq_8h &&
                ScheduleFrequency.BIWEEKLY.labelRes == R.string.sched_freq_biweekly,
        )
        check(
            "弹层选项绑定 sched_sheet_* 资源名",
            ScheduleAddOption.CUSTOM_ADD.labelRes == R.string.sched_sheet_custom_add &&
                ScheduleAddOption.ADD_VIA_CHAT.labelRes == R.string.sched_sheet_add_via_chat &&
                ScheduleAddOption.CANCEL.labelRes == R.string.sched_sheet_cancel,
        )
    }

    companion object {
        @JvmStatic
        fun main(args: Array<String>) {
            val test = ScheduledPageTest()
            test.frequency_hasSevenOptionsInSpecOrder()
            test.frequency_defaultIsDaily()
            test.addSheet_optionsInSpecOrder()
            test.form_validationRequiresNonBlankName()
            test.store_addPutsNewTaskFirst_andRemoveWorks()
            test.resources_labelResBoundToSchedStrings()
            if (test.failures.isEmpty()) {
                println("ScheduledPageTest: all checks passed")
            } else {
                test.failures.forEach { println("FAILED: $it") }
                kotlin.system.exitProcess(1)
            }
        }
    }
}
