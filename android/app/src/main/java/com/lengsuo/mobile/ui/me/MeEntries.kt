package com.lengsuo.mobile.ui.me

import androidx.annotation.StringRes
import com.lengsuo.mobile.R

/** 我的页分组。 */
enum class MeGroup(@StringRes val titleRes: Int) {
    GENERAL(R.string.me_group_general),
    ASSET(R.string.me_group_asset),
}

/** 行的形态。 */
enum class MeRowKind { NAV, VALUE, SWITCH }

/**
 * 一行设置项。纯数据，无副作用（便于后续单测与预览）。
 * @param key 稳定标识（后续接持久化/路由用）
 * @param labelRes 行标题
 * @param valueRes 行尾值文案（NAV 行常为 null）
 * @param kind 行形态
 */
data class MeEntry(
    val key: String,
    val group: MeGroup,
    @StringRes val labelRes: Int,
    @StringRes val valueRes: Int? = null,
    val kind: MeRowKind = MeRowKind.NAV,
    val switchDefaultOn: Boolean = false,
)

/**
 * 我的页全部条目（顺序即渲染顺序，逐字对齐 android-devguide/15 §6）。
 * ⚠️ 数量固定 10 行（通用 7 + 资产 2 + 关于 1），⛔ 不要增删。
 */
fun meEntries(): List<MeEntry> = listOf(
    MeEntry("theme", MeGroup.GENERAL, R.string.me_theme, kind = MeRowKind.VALUE),
    MeEntry("language", MeGroup.GENERAL, R.string.me_language, R.string.me_language_value, MeRowKind.VALUE),
    MeEntry("font_size", MeGroup.GENERAL, R.string.me_font_size, R.string.me_font_size_value, MeRowKind.VALUE),
    MeEntry("read_aloud", MeGroup.GENERAL, R.string.me_read_aloud),
    MeEntry("haptic", MeGroup.GENERAL, R.string.me_haptic, kind = MeRowKind.SWITCH, switchDefaultOn = true),
    MeEntry("storage", MeGroup.GENERAL, R.string.me_storage, R.string.me_placeholder_value, MeRowKind.VALUE),
    MeEntry("notifications", MeGroup.GENERAL, R.string.me_notifications),
    MeEntry("memory", MeGroup.ASSET, R.string.me_memory),
    MeEntry("growth", MeGroup.ASSET, R.string.me_growth),
    MeEntry("about", MeGroup.ASSET, R.string.me_about),
)
