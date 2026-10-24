package com.lengsuo.mobile

import androidx.compose.foundation.layout.fillMaxSize
import com.lengsuo.mobile.ui.me.MePage
import com.lengsuo.mobile.ui.theme.LengsuoTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview

@Composable
fun AppRoot() {
    LengsuoTheme {
        Surface(modifier = Modifier.fillMaxSize()) {
            // 暂作为唯一页面；主壳卡（mb-main-shell）接入后移入底部 Tab
            MePage()
        }
    }
}

@Preview(showBackground = true)
@Composable
private fun AppRootPreview() {
    AppRoot()
}