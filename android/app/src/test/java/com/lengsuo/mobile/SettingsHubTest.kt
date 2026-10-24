package com.lengsuo.mobile

import com.lengsuo.mobile.ui.me.meEntries
import com.lengsuo.mobile.ui.settings.SettingsGroup
import com.lengsuo.mobile.ui.settings.settingsEntries
import java.io.File

/**
 * 设置中心纯数据断言（对齐卡面 §3.7 T1–T4）。
 * 注：app 模块没有任何 testImplementation（本卡禁增 Gradle 依赖），故不引 junit；
 * 断言以纯 Kotlin 写成，main() 逐条执行，全部通过时退出码 0，否则退出码 1。
 * 文案逐字断言直接读 values/strings.xml 原文比对（JVM 侧无法解析 stringResource）。
 */
class SettingsHubTest {
    private val failures = mutableListOf<String>()

    private fun check(name: String, condition: Boolean) {
        if (!condition) failures += name
    }

    private fun loadStringsXml(): String {
        val file = listOf(
            "app/src/main/res/values/strings.xml",
            "src/main/res/values/strings.xml",
        ).mapNotNull { File(it).takeIf { f -> f.exists() } }.firstOrNull()
            ?: error("strings.xml not found; run from android/ or app/ dir")
        return file.readText()
    }

    /** 卡面 §二.2 的 19 项 + 4 组固定面，键名白名单同时守住 A8（无伪造文案键）。 */
    private val whitelistedKeys = listOf(
        "settings_hub_title",
        "settings_nav_group_general",
        "settings_nav_group_feature",
        "settings_nav_group_data_security",
        "settings_nav_group_about",
        "settings_nav_general",
        "settings_nav_account",
        "settings_nav_appearance",
        "settings_nav_software_config",
        "settings_nav_keyboard_shortcuts",
        "settings_nav_models",
        "settings_nav_connectors",
        "settings_nav_extensions",
        "settings_nav_buddy_apps",
        "settings_nav_claw",
        "settings_nav_security_center",
        "settings_nav_memory",
        "settings_nav_data_management",
        "settings_nav_subscription_usage",
        "settings_nav_system_permissions",
        "settings_nav_get_help",
        "settings_nav_help_feedback",
        "settings_nav_laboratory",
        "settings_nav_about",
    )

    fun t1_entriesCount19AndGroupCounts() {
        val entries = settingsEntries()
        check("T1 settingsEntries().size == 19, actual ${entries.size}", entries.size == 19)
        val counts = SettingsGroup.entries.associateWith { g -> entries.count { it.group == g } }
        check("T1 GENERAL=5, actual ${counts[SettingsGroup.GENERAL]}", counts[SettingsGroup.GENERAL] == 5)
        check("T1 FEATURE=5, actual ${counts[SettingsGroup.FEATURE]}", counts[SettingsGroup.FEATURE] == 5)
        check(
            "T1 DATA_SECURITY=5, actual ${counts[SettingsGroup.DATA_SECURITY]}",
            counts[SettingsGroup.DATA_SECURITY] == 5,
        )
        check("T1 ABOUT=4, actual ${counts[SettingsGroup.ABOUT]}", counts[SettingsGroup.ABOUT] == 4)
    }

    fun t2_labelResAreValidResourceIds() {
        val entries = settingsEntries()
        check("T2 全部 labelRes 为非零 R.string id", entries.all { it.labelRes != 0 })
        check("T2 全部组头 titleRes 为非零 R.string id", SettingsGroup.entries.all { it.titleRes != 0 })
        check(
            "T2 导航键互不重复, actual ${entries.size} vs ${entries.map { it.key }.toSet().size}",
            entries.map { it.key }.toSet().size == entries.size,
        )
    }

    fun t3_stringsXmlCopyIsVerbatim() {
        val xml = loadStringsXml()
        val expected = listOf(
            "settings_hub_title\">设置<",
            "settings_nav_group_general\">设置<",
            "settings_nav_group_feature\">功能<",
            "settings_nav_group_data_security\">数据与安全<",
            "settings_nav_group_about\">关于我们<",
            "settings_nav_general\">通用<",
            "settings_nav_account\">个人主页<",
            "settings_nav_appearance\">外观<",
            "settings_nav_models\">模型<",
            "settings_nav_claw\">频道推送<",
            "settings_nav_security_center\">安全中心<",
            "settings_nav_memory\">记忆与进化<",
            "settings_nav_data_management\">数据管理<",
            "settings_nav_subscription_usage\">套餐与积分<",
            "settings_nav_get_help\">获取帮助<",
            "settings_nav_help_feedback\">关于 WorkBuddy<",
            "settings_nav_laboratory\">实验室<",
            "settings_nav_about\">关于<",
        )
        expected.forEach { needle ->
            check("T3 strings.xml 缺少逐字条目: $needle", needle in xml)
        }
    }

    fun t3b_settingsKeysCount24AndWhitelisted() {
        val xml = loadStringsXml()
        val names = Regex("name=\"(settings_[a-z_]+)\"").findAll(xml)
            .map { it.groupValues[1] }.toList()
        check("T3 settings_* 键数 == 24, actual ${names.size}", names.size == 24)
        check(
            "T3 settings_* 键与卡面白名单一致, diff=${names.toSet() - whitelistedKeys.toSet()} / ${whitelistedKeys.toSet() - names.toSet()}",
            names.toSet() == whitelistedKeys.toSet(),
        )
    }

    fun t4_meEntriesNotRegressed() {
        check("T4 meEntries().size == 10, actual ${meEntries().size}", meEntries().size == 10)
        check("T4 meEntries 仍含 about 行", meEntries().any { it.key == "about" })
    }

    companion object {
        @JvmStatic
        fun main(args: Array<String>) {
            val test = SettingsHubTest()
            test.t1_entriesCount19AndGroupCounts()
            test.t2_labelResAreValidResourceIds()
            test.t3_stringsXmlCopyIsVerbatim()
            test.t3b_settingsKeysCount24AndWhitelisted()
            test.t4_meEntriesNotRegressed()
            if (test.failures.isEmpty()) {
                println("SettingsHubTest: all checks passed")
            } else {
                test.failures.forEach { println("FAILED: $it") }
                kotlin.system.exitProcess(1)
            }
        }
    }
}
