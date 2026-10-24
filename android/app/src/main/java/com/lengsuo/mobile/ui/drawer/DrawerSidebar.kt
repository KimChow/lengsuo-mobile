package com.lengsuo.mobile.ui.drawer

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.lengsuo.mobile.R
import com.lengsuo.mobile.ui.theme.LocalLengsuoColors

// 示例数据（写死；后续卡接真实会话流）
internal val DRAWER_SAMPLE_CONVERSATIONS = listOf("hello", "周报整理", "旅行计划")

// 实测抽屉宽 343dp（1201px ÷ 3.5），取整 340dp
internal val DRAWER_WIDTH_DP = 340

// 选中态谓词：当前打开的对话 = 浅灰选中底
internal fun isSelected(index: Int, selectedIndex: Int?): Boolean = selectedIndex == index

// 空态谓词：列表空 ⇒ 不渲染分组与条目，直接渲染空态文案
internal fun isEmptyState(conversations: List<String>): Boolean = conversations.isEmpty()

/**
 * 对话列表侧栏抽屉（静态 + 本地交互）。
 * 实现 = Box + 条件渲染（不用 ModalNavigationDrawer，开关动画省略为直接显隐）。
 * 占位声明：分组下拉（☁ 行）不做真实切换（点击无操作）、「+」胶囊点击仅关抽屉（无跳转）、
 * 分组标题右侧列表管理图标材料未给，均不做。
 */
@Composable
fun DrawerSidebar(
    visible: Boolean,
    conversations: List<String>,
    selectedIndex: Int?,
    onDismiss: () -> Unit,
    onSelectConversation: (Int) -> Unit,
    onNewTask: () -> Unit,
    // mb-chat-page 接线：条目点击打开对话页；默认空实现 ⇒ 既有调用零破坏（宿主页传入属后续卡）
    onOpenConversation: (Int) -> Unit = {},
) {
    if (!visible) return
    val palette = LocalLengsuoColors.current
    Box(modifier = Modifier.fillMaxSize()) {
        Row(modifier = Modifier.fillMaxSize()) {
            Column(
                modifier = Modifier
                    .width(DRAWER_WIDTH_DP.dp)
                    .fillMaxHeight()
                    .background(palette.pageBackground)
                    .padding(vertical = 12.dp)
                    .padding(horizontal = 12.dp),
            ) {
                // 分组下拉（占位，点击无操作）
                Text(
                    text = stringResource(R.string.drawer_cloud_group),
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                    modifier = Modifier.padding(start = 12.dp, top = 8.dp, bottom = 8.dp),
                )
                // 「+」胶囊：点击关抽屉（无跳转，占位）
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(44.dp)
                        .background(palette.odcButtonBackground, RoundedCornerShape(16.dp))
                        .clickable(onClick = onNewTask),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.Center,
                ) {
                    Icon(
                        Icons.Filled.Add,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onSurface,
                        modifier = Modifier.size(18.dp),
                    )
                    Text(
                        text = stringResource(R.string.drawer_new_task),
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurface,
                        modifier = Modifier.padding(start = 4.dp),
                    )
                }
                if (isEmptyState(conversations)) {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .weight(1f),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(
                            text = stringResource(R.string.drawer_empty_cloud),
                            style = MaterialTheme.typography.bodyMedium,
                            color = palette.mainThirdText,
                        )
                    }
                } else {
                    Text(
                        text = stringResource(R.string.drawer_task_group),
                        style = MaterialTheme.typography.labelMedium,
                        color = palette.mainSecondText,
                        modifier = Modifier.padding(top = 24.dp, bottom = 4.dp),
                    )
                    Column(modifier = Modifier.fillMaxWidth()) {
                        conversations.forEachIndexed { index, title ->
                            Box(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .height(44.dp)
                                    .background(
                                        if (isSelected(index, selectedIndex)) {
                                            palette.odcButtonBackground
                                        } else {
                                            Color.Transparent
                                        },
                                    )
                                    .clickable {
                                        onSelectConversation(index)
                                        onOpenConversation(index)
                                    }
                                    .padding(horizontal = 12.dp),
                                contentAlignment = Alignment.CenterStart,
                            ) {
                                Text(
                                    text = title,
                                    style = MaterialTheme.typography.bodyMedium,
                                    color = MaterialTheme.colorScheme.onSurface,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                )
                            }
                        }
                    }
                }
            }
            // 半透明遮罩：点遮罩关抽屉
            Box(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxHeight()
                    .background(Color.Black.copy(alpha = 0.4f))
                    .clickable(onClick = onDismiss),
            )
        }
    }
}
