package com.lengsuo.mobile

import com.lengsuo.mobile.ui.home.HOME_CAPSULES
import com.lengsuo.mobile.ui.home.cycle

/**
 * 任务首页纯数据断言。
 * 注：app 模块没有任何 testImplementation（本卡禁增 Gradle 依赖），故不引 junit；
 * 断言以纯 Kotlin 写成，main() 逐条执行，全部通过时退出码 0，否则退出码 1。
 */
class TasksHomeTest {
    private val failures = mutableListOf<String>()

    private fun check(name: String, condition: Boolean) {
        if (!condition) failures += name
    }

    fun homeCapsules_staticSetHasFourSuggestions() {
        check(
            "HOME_CAPSULES.size == 4, actual ${HOME_CAPSULES.size}",
            HOME_CAPSULES.size == 4,
        )
    }

    fun homeCapsules_orderMatchesSpec() {
        val expected = listOf(
            R.string.home_capsule_data_analysis,
            R.string.home_capsule_doc_process,
            R.string.home_capsule_slides,
            R.string.home_capsule_workspace,
        )
        check("HOME_CAPSULES 顺序 = 数据分析/文档/幻灯片/工作台", HOME_CAPSULES == expected)
    }

    fun homeCapsules_cycleShiftsLeftByOne() {
        val shifted = HOME_CAPSULES.cycle(1)
        check(
            "cycle(1) 后顺序整体左移一位",
            shifted == HOME_CAPSULES.drop(1) + HOME_CAPSULES.take(1),
        )
    }

    fun homeCapsules_cycleWrapsAround() {
        check("cycle(4) 回到原序", HOME_CAPSULES.cycle(4) == HOME_CAPSULES)
        check("cycle(0) 原序不变", HOME_CAPSULES.cycle(0) == HOME_CAPSULES)
    }

    companion object {
        @JvmStatic
        fun main(args: Array<String>) {
            val test = TasksHomeTest()
            test.homeCapsules_staticSetHasFourSuggestions()
            test.homeCapsules_orderMatchesSpec()
            test.homeCapsules_cycleShiftsLeftByOne()
            test.homeCapsules_cycleWrapsAround()
            if (test.failures.isEmpty()) {
                println("TasksHomeTest: all checks passed")
            } else {
                test.failures.forEach { println("FAILED: $it") }
                kotlin.system.exitProcess(1)
            }
        }
    }
}
