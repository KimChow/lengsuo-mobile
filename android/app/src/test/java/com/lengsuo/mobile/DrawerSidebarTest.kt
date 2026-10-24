package com.lengsuo.mobile

import com.lengsuo.mobile.ui.drawer.DRAWER_SAMPLE_CONVERSATIONS
import com.lengsuo.mobile.ui.drawer.DRAWER_WIDTH_DP
import com.lengsuo.mobile.ui.drawer.isEmptyState
import com.lengsuo.mobile.ui.drawer.isSelected

/**
 * 侧栏抽屉纯数据断言。
 * 注：app 模块没有任何 testImplementation（本卡禁增 Gradle 依赖），故不引 junit；
 * 断言以纯 Kotlin 写成，main() 逐条执行，全部通过时退出码 0，否则退出码 1。
 */
class DrawerSidebarTest {
    private val failures = mutableListOf<String>()

    private fun check(name: String, condition: Boolean) {
        if (!condition) failures += name
    }

    fun sampleConversations_threeEntries() {
        check(
            "DRAWER_SAMPLE_CONVERSATIONS.size == 3, actual ${DRAWER_SAMPLE_CONVERSATIONS.size}",
            DRAWER_SAMPLE_CONVERSATIONS.size == 3,
        )
    }

    fun sampleConversations_titlesMatchSpec() {
        check(
            "标题逐字 = hello/周报整理/旅行计划",
            DRAWER_SAMPLE_CONVERSATIONS == listOf("hello", "周报整理", "旅行计划"),
        )
    }

    fun isSelected_truthTable() {
        check("isSelected(0, 0) = true", isSelected(0, 0))
        check("isSelected(1, 0) = false", !isSelected(1, 0))
        check("isSelected(5, null) = false", !isSelected(5, null))
    }

    fun isEmptyState_truthTable() {
        check(
            "isEmptyState(emptyList()) = true",
            isEmptyState(emptyList()),
        )
        check(
            "isEmptyState(DRAWER_SAMPLE_CONVERSATIONS) = false",
            !isEmptyState(DRAWER_SAMPLE_CONVERSATIONS),
        )
    }

    fun drawerWidth_is340dp() {
        check(
            "DRAWER_WIDTH_DP == 340, actual $DRAWER_WIDTH_DP",
            DRAWER_WIDTH_DP == 340,
        )
    }

    companion object {
        @JvmStatic
        fun main(args: Array<String>) {
            val test = DrawerSidebarTest()
            test.sampleConversations_threeEntries()
            test.sampleConversations_titlesMatchSpec()
            test.isSelected_truthTable()
            test.isEmptyState_truthTable()
            test.drawerWidth_is340dp()
            if (test.failures.isEmpty()) {
                println("DrawerSidebarTest: all checks passed")
            } else {
                test.failures.forEach { println("FAILED: $it") }
                kotlin.system.exitProcess(1)
            }
        }
    }
}
