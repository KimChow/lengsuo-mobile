package com.lengsuo.mobile.ui.me

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.lengsuo.mobile.R

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MePage(modifier: Modifier = Modifier) {
    // 占位页状态：null = 我的页；非 null = 某一行的占位面板
    var opened by remember { mutableStateOf<MeEntry?>(null) }
    val current = opened
    if (current != null) {
        BackHandler { opened = null }
        PanelPlaceholder(
            title = stringResource(current.labelRes),
            onBack = { opened = null },
            modifier = modifier,
        )
        return
    }
    Scaffold(
        modifier = modifier.fillMaxSize(),
        topBar = { TopAppBar(title = { Text(stringResource(R.string.me_title)) }) },
    ) { inner ->
        LazyColumn(
            modifier = Modifier.fillMaxSize().padding(inner),
            contentPadding = PaddingValues(vertical = 8.dp),
        ) {
            item { IdentityCard() }
            item { PlanCard() }
            val rows = meEntries()
            MeGroup.entries.forEach { group ->
                item(key = "hdr_${group.name}") { GroupHeader(stringResource(group.titleRes)) }
                items(rows.filter { it.group == group }, key = { it.key }) { entry ->
                    MeRow(entry = entry, onClick = { opened = entry })
                }
            }
        }
    }
}

/** 身份卡：圆形头像占位 + 昵称占位 + 箭头。 */
@Composable
private fun IdentityCard() {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(min = 56.dp)
            .padding(horizontal = 16.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier = Modifier
                .size(48.dp)
                .background(MaterialTheme.colorScheme.primaryContainer, CircleShape),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = "我",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onPrimaryContainer,
            )
        }
        Spacer(Modifier.width(16.dp))
        Text(
            text = stringResource(R.string.me_placeholder_value),
            style = MaterialTheme.typography.titleMedium,
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.weight(1f),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Text(
            text = "›",
            style = MaterialTheme.typography.titleLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

/** 套餐卡：体验版 + 升级按钮 + 积分行（占位值，不写任何真实数字）。 */
@Composable
private fun PlanCard() {
    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 8.dp),
        color = MaterialTheme.colorScheme.surfaceVariant,
        shape = MaterialTheme.shapes.medium,
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = stringResource(R.string.me_plan_free),
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                    modifier = Modifier.weight(1f),
                )
                OutlinedButton(onClick = { /* 占位：套餐页另立卡 */ }) {
                    Text(stringResource(R.string.me_plan_upgrade))
                }
            }
            Spacer(Modifier.height(8.dp))
            Text(
                text = stringResource(R.string.me_points) +
                    " " + stringResource(R.string.me_placeholder_value),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

/** 分组标题。 */
@Composable
private fun GroupHeader(title: String) {
    Text(
        text = title,
        style = MaterialTheme.typography.labelLarge,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = Modifier.padding(start = 16.dp, end = 16.dp, top = 16.dp, bottom = 8.dp),
    )
}

/** 一行设置项：整行可点，行高 ≥ 56dp，带无障碍描述。 */
@Composable
private fun MeRow(entry: MeEntry, onClick: () -> Unit) {
    val label = stringResource(entry.labelRes)
    var switchOn by remember { mutableStateOf(entry.switchDefaultOn) }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(min = 56.dp)
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
            MeRowKind.VALUE -> {
                entry.valueRes?.let { valueRes ->
                    Text(
                        text = stringResource(valueRes),
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                    )
                }
                Text(
                    text = "›",
                    style = MaterialTheme.typography.titleLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(start = 4.dp),
                )
            }
            MeRowKind.SWITCH -> {
                // 只保留本地状态，不做持久化（持久化另立卡）
                Switch(checked = switchOn, onCheckedChange = { switchOn = it })
            }
            MeRowKind.NAV -> {
                Text(
                    text = "›",
                    style = MaterialTheme.typography.titleLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

/** 面板占位页：返回 + 居中「敬请期待」。 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun PanelPlaceholder(title: String, onBack: () -> Unit, modifier: Modifier = Modifier) {
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
