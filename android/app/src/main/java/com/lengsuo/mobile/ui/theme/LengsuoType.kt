package com.lengsuo.mobile.ui.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

/**
 * 排版：正文用系统字体（参考实测：正文=系统字体）。
 * ⚠️ 展示字 AlimamaShuHeiTi-Bold 属字体资产，**本卡不引入**（另立卡），TODO 标注。
 * 字号档（参考 settings.fontSize: default/small/large）见 LengsuoFontScale。
 */
// TODO(字体资产卡): 展示字改用 AlimamaShuHeiTi-Bold（需 font 文件 + 授权）
private val DisplayFamily = FontFamily.SansSerif
private val BodyFamily = FontFamily.SansSerif

val LengsuoTypography = Typography(
    displayLarge = TextStyle(fontFamily = DisplayFamily, fontWeight = FontWeight.Bold, fontSize = 36.sp, lineHeight = 44.sp),
    titleLarge = TextStyle(fontFamily = BodyFamily, fontWeight = FontWeight.Medium, fontSize = 20.sp, lineHeight = 28.sp),
    bodyLarge = TextStyle(fontFamily = BodyFamily, fontWeight = FontWeight.Normal, fontSize = 16.sp, lineHeight = 24.sp),
    bodyMedium = TextStyle(fontFamily = BodyFamily, fontWeight = FontWeight.Normal, fontSize = 14.sp, lineHeight = 20.sp),
    labelSmall = TextStyle(fontFamily = BodyFamily, fontWeight = FontWeight.Normal, fontSize = 12.sp, lineHeight = 16.sp),
)

/** 字号档（参考 settings.fontSize：default / small / large 三档）。 */
enum class LengsuoFontScale(val factor: Float) {
    SMALL(0.9f),
    DEFAULT(1.0f),
    LARGE(1.15f),
}
