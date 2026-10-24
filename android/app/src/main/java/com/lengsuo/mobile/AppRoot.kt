package com.lengsuo.mobile

import androidx.compose.foundation.layout.fillMaxSize
import com.lengsuo.mobile.ui.shell.MainShell
import com.lengsuo.mobile.ui.theme.LengsuoTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview

@Composable
fun AppRoot() {
    LengsuoTheme {
        Surface(modifier = Modifier.fillMaxSize()) {
            MainShell()
        }
    }
}

@Preview(showBackground = true)
@Composable
private fun AppRootPreview() {
    AppRoot()
}