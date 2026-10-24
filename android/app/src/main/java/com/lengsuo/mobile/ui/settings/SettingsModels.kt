package com.lengsuo.mobile.ui.settings

import androidx.annotation.StringRes
import com.lengsuo.mobile.R

/** 设置中心分组（4 组，组头与 PC 同源）。 */
enum class SettingsGroup(@StringRes val titleRes: Int) {
    GENERAL(R.string.settings_nav_group_general),
    FEATURE(R.string.settings_nav_group_feature),
    DATA_SECURITY(R.string.settings_nav_group_data_security),
    ABOUT(R.string.settings_nav_group_about),
}

/** 面板行的形态。 */
enum class SettingsRowKind { NAV }

/**
 * 设置中心一行面板项。纯数据，无副作用（便于后续单测与预览）。
 * @param key 导航键（settings.nav.*，后续接路由用）
 * @param group 所属分组
 * @param labelRes 行标题
 */
data class SettingsEntry(
    val key: String,
    val group: SettingsGroup,
    @StringRes val labelRes: Int,
)

/**
 * 设置中心全部面板项（顺序即渲染顺序，组内顺序对齐卡面 §二.2 表）。
 * ⚠️ 数量固定 19 项（设置 5 + 功能 5 + 数据与安全 5 + 关于我们 4），⛔ 不要增删。
 */
fun settingsEntries(): List<SettingsEntry> = listOf(
    SettingsEntry("settings.nav.settings", SettingsGroup.GENERAL, R.string.settings_nav_general),
    SettingsEntry("settings.nav.account", SettingsGroup.GENERAL, R.string.settings_nav_account),
    SettingsEntry("settings.nav.appearance", SettingsGroup.GENERAL, R.string.settings_nav_appearance),
    SettingsEntry("settings.nav.softwareConfig", SettingsGroup.GENERAL, R.string.settings_nav_software_config),
    SettingsEntry("settings.nav.keyboardShortcuts", SettingsGroup.GENERAL, R.string.settings_nav_keyboard_shortcuts),
    SettingsEntry("settings.nav.models", SettingsGroup.FEATURE, R.string.settings_nav_models),
    SettingsEntry("settings.nav.connectors", SettingsGroup.FEATURE, R.string.settings_nav_connectors),
    SettingsEntry("settings.nav.extensions", SettingsGroup.FEATURE, R.string.settings_nav_extensions),
    SettingsEntry("settings.nav.buddyApps", SettingsGroup.FEATURE, R.string.settings_nav_buddy_apps),
    SettingsEntry("settings.nav.claw", SettingsGroup.FEATURE, R.string.settings_nav_claw),
    SettingsEntry("settings.nav.securityCenter", SettingsGroup.DATA_SECURITY, R.string.settings_nav_security_center),
    SettingsEntry("settings.nav.memory", SettingsGroup.DATA_SECURITY, R.string.settings_nav_memory),
    SettingsEntry("settings.nav.dataManagement", SettingsGroup.DATA_SECURITY, R.string.settings_nav_data_management),
    SettingsEntry("settings.nav.subscriptionUsage", SettingsGroup.DATA_SECURITY, R.string.settings_nav_subscription_usage),
    SettingsEntry("settings.nav.systemPermissions", SettingsGroup.DATA_SECURITY, R.string.settings_nav_system_permissions),
    SettingsEntry("settings.nav.getHelp", SettingsGroup.ABOUT, R.string.settings_nav_get_help),
    SettingsEntry("settings.nav.helpFeedback", SettingsGroup.ABOUT, R.string.settings_nav_help_feedback),
    SettingsEntry("settings.nav.laboratory", SettingsGroup.ABOUT, R.string.settings_nav_laboratory),
    SettingsEntry("settings.nav.about", SettingsGroup.ABOUT, R.string.settings_nav_about),
)
