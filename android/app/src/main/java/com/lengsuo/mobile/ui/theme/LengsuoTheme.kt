package com.lengsuo.mobile.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.staticCompositionLocalOf

val LocalLengsuoColors = staticCompositionLocalOf { LightLengsuoColors }

/**
 * 三态主题：
 *  - SYSTEM ⇒ 跟随系统（isSystemInDarkTheme）
 *  - LIGHT / DARK ⇒ 强制
 * ⛔ 明暗是两套独立色板（不是反色）。
 * ⛔ 切换走 Compose 重组，**不重建 Activity**（参考侧验收口径）。
 */
@Composable
fun LengsuoTheme(
    mode: LengsuoThemeMode = LengsuoThemeMode.SYSTEM,
    content: @Composable () -> Unit,
) {
    val dark = when (mode) {
        LengsuoThemeMode.SYSTEM -> isSystemInDarkTheme()
        LengsuoThemeMode.LIGHT -> false
        LengsuoThemeMode.DARK -> true
    }
    val palette = if (dark) DarkLengsuoColors else LightLengsuoColors

    // Material3 基线：只映射我们有实测值的语义槽；其余用 M3 默认（TODO 待补实测）
    val scheme = if (dark) {
        darkColorScheme(
            primary = BrandMain,
            onPrimary = BrandButtonText,
            background = palette.pageBackground,
            onBackground = palette.mainText,
            surface = palette.pageBackground,
            onSurface = palette.mainText,
            onSurfaceVariant = palette.mainSecondText,
            outline = palette.borderColor,
        )
    } else {
        lightColorScheme(
            primary = BrandMain,
            onPrimary = BrandButtonText,
            background = palette.pageBackground,
            onBackground = palette.mainText,
            surface = palette.pageBackground,
            onSurface = palette.mainText,
            onSurfaceVariant = palette.mainSecondText,
            outline = palette.borderColor,
        )
    }

    CompositionLocalProvider(LocalLengsuoColors provides palette) {
        MaterialTheme(colorScheme = scheme, content = content)
    }
}
