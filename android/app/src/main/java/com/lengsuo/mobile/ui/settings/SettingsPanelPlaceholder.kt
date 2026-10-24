package com.lengsuo.mobile.ui.settings

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.lengsuo.mobile.R

/** 面板占位页：返回 + 居中「敬请期待」（15 个面板的占位路由都落这里，面板内容另立卡）。 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsPanelPlaceholder(
    panelKey: String,
    title: String,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Scaffold(
        modifier = modifier.fillMaxSize(),
        topBar = {
            TopAppBar(
                title = { Text(title) },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Text(text = "‹", style = MaterialTheme.typography.titleLarge)
                    }
                },
            )
        },
    ) { inner ->
        Box(
            modifier = Modifier.fillMaxSize().padding(inner),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = stringResource(R.string.me_coming_soon),
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}
