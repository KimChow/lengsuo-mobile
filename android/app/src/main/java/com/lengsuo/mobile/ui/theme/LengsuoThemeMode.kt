package com.lengsuo.mobile.ui.theme

/**
 * 主题三态（参考产品「我的页」三段控件：系统 / 浅色 / 深色）。
 * ⚠️ 持久化键沿用参考侧实测名 `settings_theme_mode`（Dart 侧 libapp.so 字符串实证）。
 */
enum class LengsuoThemeMode {
    SYSTEM,
    LIGHT,
    DARK;

    companion object {
        const val PREF_KEY = "settings_theme_mode"

        fun fromStored(raw: String?): LengsuoThemeMode =
            when (raw?.uppercase()) {
                "LIGHT" -> LIGHT
                "DARK" -> DARK
                else -> SYSTEM
            }
    }
}
