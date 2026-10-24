package com.lengsuo.mobile.ui.home

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.Menu
import androidx.compose.material3.AssistChip
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.lengsuo.mobile.R
import com.lengsuo.mobile.ui.theme.LocalLengsuoColors

// 首页 4 条快捷建议（截屏实证文案，见 strings.xml home_capsule_*）
internal val HOME_CAPSULES = listOf(
    R.string.home_capsule_data_analysis,
    R.string.home_capsule_doc_process,
    R.string.home_capsule_slides,
    R.string.home_capsule_workspace,
)

@Composable
fun TasksHomePage() {
    var bannerVisible by remember { mutableStateOf(true) }
    var refreshCount by remember { mutableStateOf(0) }
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(horizontal = 16.dp),
    ) {
        // 顶栏：抽屉钮 + WorkBuddy + 工作模式
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                Icons.Filled.Menu,
                contentDescription = stringResource(R.string.home_top_drawer),
                tint = MaterialTheme.colorScheme.onSurface,
            )
            Spacer(Modifier.width(12.dp))
            Text(
                text = "WorkBuddy",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Spacer(Modifier.weight(1f))
            Text(
                text = stringResource(R.string.home_top_work_mode),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
        }
        Spacer(Modifier.height(12.dp))
        if (bannerVisible) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(
                        LocalLengsuoColors.current.odcButtonBackground,
                        RoundedCornerShape(12.dp),
                    )
                    .padding(12.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = stringResource(R.string.home_banner_placeholder),
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                    modifier = Modifier.weight(1f),
                )
                IconButton(onClick = { bannerVisible = false }) {
                    Text(
                        text = "×",
                        style = MaterialTheme.typography.titleMedium,
                        color = MaterialTheme.colorScheme.onSurface,
                    )
                }
            }
            Spacer(Modifier.height(12.dp))
        }
        // 吉祥物静态占位（Rive 待后续卡）
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(48.dp)
                .background(
                    LocalLengsuoColors.current.odcButtonBackground,
                    RoundedCornerShape(12.dp),
                ),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = stringResource(R.string.home_mascot_placeholder),
                style = MaterialTheme.typography.labelMedium,
                color = LocalLengsuoColors.current.mainSecondText,
            )
        }
        Spacer(Modifier.height(24.dp))
        // 欢迎语
        Text(
            text = stringResource(R.string.home_header_title),
            style = MaterialTheme.typography.headlineMedium,
            color = MaterialTheme.colorScheme.onSurface,
        )
        Text(
            text = stringResource(R.string.home_header_subtitle),
            style = MaterialTheme.typography.bodyLarge,
            color = LocalLengsuoColors.current.mainSecondText,
        )
        Spacer(Modifier.height(16.dp))
        // 4 条快捷胶囊 + 换一换
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Spacer(Modifier.weight(1f))
            TextButton(onClick = { refreshCount += 1 }) {
                Text(stringResource(R.string.home_capsule_refresh))
            }
        }
        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            items(HOME_CAPSULES.cycle(refreshCount)) { res ->
                AssistChip(
                    onClick = { /* 预填发送属会话卡，本卡不做 */ },
                    label = { Text(stringResource(res)) },
                )
            }
        }
        Spacer(Modifier.height(24.dp))
        // 输入栏占位
        OutlinedTextField(
            value = "",
            onValueChange = {},
            modifier = Modifier.fillMaxWidth(),
            enabled = false,
            placeholder = { Text(stringResource(R.string.home_input_hint)) },
            leadingIcon = {
                Icon(
                    Icons.Filled.Add,
                    contentDescription = stringResource(R.string.home_input_attachment),
                )
            },
            trailingIcon = {
                // 麦克风占位：core 集合无 Mic/KeyboardVoice，待扩展图标库批准后替换
                Icon(
                    Icons.Filled.PlayArrow,
                    contentDescription = stringResource(R.string.home_input_voice),
                )
            },
        )
        Spacer(Modifier.height(16.dp))
    }
}

// 换一换：按 refreshCount 轮转建议顺序（静态建议集只有一组 4 条）
internal fun List<Int>.cycle(times: Int): List<Int> {
    val n = times.mod(size)
    return drop(n) + take(n)
}
