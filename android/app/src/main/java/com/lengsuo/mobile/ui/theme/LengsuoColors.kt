package com.lengsuo.mobile.ui.theme

import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.Color

/**
 * 品牌与语义色板。
 * ⚠️ 全部色值来自参考 APK 资源实测（res/values + res/values-night 双套），⛔ 不可自造。
 * 证据：docs/android-theme-rea-evidence.md 附录。
 */
@Immutable
data class LengsuoColors(
    val pageBackground: Color,
    val mainText: Color,
    val mainSecondText: Color,
    val mainThirdText: Color,
    val primary: Color,
    val dialogBackground: Color,
    val splashBackground: Color,
    val lightButton: Color,
    val lightButtonPressed: Color,
    val odcButtonBackground: Color,
    val tencentButtonBackground: Color,
    val borderColor: Color,
    val actionBorderColor: Color,
)

// 品牌色：仅亮色定义 ⇒ 明暗共用（实测无 night 覆盖）
val BrandMain = Color(0xFF005ED9)        // #005ed9
val BrandMainLight = Color(0xFF0070E0)   // #0070e0
val BrandButtonText = Color(0xFFFFFFFF)  // #ffffff
val BrandFontBlue = Color(0xFF066FFF)    // #066fff
val BrandReadDot = Color(0xFFF74C31)     // #f74c31
val BrandColorBackground = Color(0xFFF4F8FE) // #f4f8fe

val LightLengsuoColors = LengsuoColors(
    pageBackground = Color(0xFFFFFFFF),        // #ffffff
    mainText = Color(0xFF333333),              // #333333
    mainSecondText = Color(0xFF666666),        // #666666
    mainThirdText = Color(0xFF999999),         // #999999
    primary = Color(0xFF1A1A1A),               // #1a1a1a
    dialogBackground = Color(0xFFFFFFFF),      // #ffffff
    splashBackground = Color(0xFFFFFFFF),      // #ffffff
    lightButton = Color(0xFFFFFFFF),           // #ffffff
    lightButtonPressed = Color(0xFFF3F3F3),    // #f3f3f3
    odcButtonBackground = Color(0xFFEBF0F7),   // #ebf0f7
    tencentButtonBackground = Color(0xFFF2F7FF), // #f2f7ff
    borderColor = Color(0xFF333333),           // #333333
    actionBorderColor = Color(0xFFEEEEEE),     // #eeeeee
)

val DarkLengsuoColors = LightLengsuoColors.copy(
    pageBackground = Color(0xFF0F1011),        // #0f1011
    mainText = Color(0xFFFFFFFF),              // #ffffff
    mainSecondText = Color(0xFFABABAB),        // #ababab
    mainThirdText = Color(0xFF888888),         // #888888
    primary = Color(0xFFFFFFFF),               // #ffffff
    dialogBackground = Color(0xFF333333),      // #333333
    splashBackground = Color(0xFF1A1A1A),      // #1a1a1a
    lightButton = Color(0xFFCCCCCC),           // #cccccc
    lightButtonPressed = Color(0xFFC3C3C3),    // #c3c3c3
    odcButtonBackground = Color(0xFF4D5768),   // #4d5768
    tencentButtonBackground = Color(0xFF32537E), // #32537e
    borderColor = Color(0xFFAAAAAA),           // #aaaaaa
    actionBorderColor = Color(0xFF666666),     // #666666
)
