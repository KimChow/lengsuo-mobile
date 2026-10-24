package com.lengsuo.mobile.ui.settings

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
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
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.lengsuo.mobile.R

/** 设置中心页：4 组头 + 19 面板导航行（本卡只做导航，面板内容另立卡）。 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsHubPage(
    onOpenPanel: (String) -> Unit,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Scaffold(
        modifier = modifier.fillMaxSize(),
        topBar = {
            TopAppBar(
                title = { Text(stringResource(R.string.settings_hub_title)) },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Text(text = "‹", style = MaterialTheme.typography.titleLarge)
                    }
                },
            )
        },
    ) { inner ->
        LazyColumn(
            modifier = Modifier.fillMaxSize().padding(inner),
            contentPadding = PaddingValues(vertical = 8.dp),
        ) {
            val rows = settingsEntries()
            SettingsGroup.entries.forEach { group ->
                item(key = "hdr_${group.name}") { GroupHeader(stringResource(group.titleRes)) }
                items(rows.filter { it.group == group }, key = { it.key }) { entry ->
                    SettingsRow(entry = entry, onClick = { onOpenPanel(entry.key) })
                }
            }
        }
    }
}

/** 分组标题。 */
@Composable
private fun GroupHeader(title: String) {
    Text(
        text = title,
        style = MaterialTheme.typography.titleSmall,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = Modifier.padding(start = 16.dp, end = 16.dp, top = 16.dp, bottom = 8.dp),
    )
}

/** 一行面板项：整行可点，行高 ≥ 48dp，行尾「›」，带无障碍描述。 */
@Composable
private fun SettingsRow(entry: SettingsEntry, onClick: () -> Unit) {
    val label = stringResource(entry.labelRes)
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(min = 48.dp)
            .clickable(onClick = onClick)
            .semantics { contentDescription = label }
            .padding(horizontal = 16.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.weight(1f),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        when (entry.kind) {
            SettingsRowKind.NAV ->
                Text(
                    text = "›",
                    style = MaterialTheme.typography.titleLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
        }
    }
}
