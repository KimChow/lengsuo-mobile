package com.lengsuo.mobile.ui.chat

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.Menu
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.Share
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.ThumbUp
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.lengsuo.mobile.R
import com.lengsuo.mobile.ui.theme.LocalLengsuoColors

// 本地交互两态（不做真实网络 / AI 回复 / 持久化，消息一律来自写死示例）
// 注：public（ChatPage 公开签名含此类型，internal 会被编译器拒绝）
enum class ChatPhase { SENDING, DONE }

// 示例数据（写死；标题 = 首条消息内容，即自动命名规则）
internal val CHAT_SAMPLE_TITLE = "hello"
internal val CHAT_SAMPLE_USER_MESSAGE = "hello"
internal val CHAT_SAMPLE_AI_REPLY = "这是静态示例回复：占位内容，展示 AI 正文左对齐无气泡的排版。"

// 回复工具条 5 键实测尺寸（84px ÷ 3.5 = 24dp，键间距 42px ÷ 3.5 = 12dp）
internal const val TOOLBAR_ICON_DP = 24
internal const val TOOLBAR_GAP_DP = 12

// 工具条 5 键 desc 资源 id（顺序 = 实拍从左到右）；文案逐字在 strings.xml
internal val CHAT_TOOLBAR_ITEMS = listOf(
    R.string.chat_toolbar_copy,
    R.string.chat_toolbar_like,
    R.string.chat_toolbar_dislike,
    R.string.chat_toolbar_read_aloud,
    R.string.chat_toolbar_share,
)

// 两态映射纯函数（返回 strings.xml 资源 id，JVM 测试断言 id + 逐字文案双保险）
internal fun chatStatusLabel(phase: ChatPhase): Int =
    if (phase == ChatPhase.SENDING) R.string.chat_status_thinking else R.string.chat_status_done

internal fun chatModelTierLabel(phase: ChatPhase): Int =
    if (phase == ChatPhase.SENDING) R.string.chat_model_tier_sending else R.string.chat_model_tier_idle

internal fun chatPlaceholder(phase: ChatPhase): Int =
    if (phase == ChatPhase.SENDING) {
        R.string.chat_input_placeholder_sending
    } else {
        R.string.chat_input_placeholder_idle
    }

/**
 * 对话页（静态 + 本地交互两态）。实现 = Column 通栏，无导航库（返回 = 调用方本地 state 切换）。
 * 占位声明：顶栏 ☰（抽屉钮）/ ⊕ 新建对话 / ⋯、语音、+、工具条 5 键均点击无操作；
 * ⏹ 停止键点击 = 无操作占位（两态由 phase 参数驱动，本页无内部状态）。
 */
@Composable
fun ChatPage(
    title: String,
    phase: ChatPhase,
    userMessage: String,
    aiReply: String,
) {
    val palette = LocalLengsuoColors.current
    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(palette.pageBackground),
    ) {
        // 顶栏：☰ + 标题/副标题 + ⊕ 新建对话 + ⋯ 更多操作
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                Icons.Filled.Menu,
                contentDescription = stringResource(R.string.home_top_drawer),
                tint = MaterialTheme.colorScheme.onSurface,
            )
            Spacer(Modifier.width(12.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = stringResource(R.string.chat_subtitle_mode),
                    style = MaterialTheme.typography.bodySmall,
                    color = palette.mainSecondText,
                )
            }
            Icon(
                Icons.Filled.Add,
                contentDescription = stringResource(R.string.chat_top_new_chat),
                tint = MaterialTheme.colorScheme.onSurface,
            )
            Spacer(Modifier.width(12.dp))
            Icon(
                Icons.Filled.MoreVert,
                contentDescription = stringResource(R.string.chat_top_more),
                tint = MaterialTheme.colorScheme.onSurface,
            )
        }
        // AI 来源提示条（通栏）
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp)
                .background(palette.odcButtonBackground, RoundedCornerShape(12.dp))
                .padding(12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text = stringResource(R.string.chat_ai_banner),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
        }
        Spacer(Modifier.height(12.dp))
        // 消息区（滚动）
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .weight(1f)
                .verticalScroll(rememberScrollState())
                .padding(horizontal = 16.dp),
        ) {
            // 用户气泡：右对齐 + surfaceVariant 圆角底
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.End,
            ) {
                Text(
                    text = userMessage,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                    modifier = Modifier
                        .background(
                            MaterialTheme.colorScheme.surfaceVariant,
                            RoundedCornerShape(12.dp),
                        )
                        .padding(12.dp),
                )
            }
            Spacer(Modifier.height(8.dp))
            // 执行状态行：发送中 = 思考提示；完成态 = 已完成 + 右箭头
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = stringResource(chatStatusLabel(phase)),
                    style = MaterialTheme.typography.bodyMedium,
                    color = palette.mainSecondText,
                )
                if (phase == ChatPhase.DONE) {
                    Icon(
                        Icons.Filled.KeyboardArrowRight,
                        contentDescription = null,
                        tint = palette.mainSecondText,
                    )
                }
            }
            Spacer(Modifier.height(8.dp))
            // AI 正文：左对齐、无气泡直接排版
            Text(
                text = aiReply,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Spacer(Modifier.height(12.dp))
            // 回复工具条 5 键：24×24dp、键间距 12dp（点击均无操作占位）
            Row(horizontalArrangement = Arrangement.spacedBy(TOOLBAR_GAP_DP.dp)) {
                val toolbarIcons = listOf(
                    // core 集合无 ContentCopy / VolumeUp / ThumbDown，暂代图标（同 TasksHome 麦克风先例），desc 语义为准
                    Icons.Filled.Edit,
                    Icons.Filled.ThumbUp,
                    Icons.Filled.Close,
                    Icons.Filled.PlayArrow,
                    Icons.Filled.Share,
                )
                CHAT_TOOLBAR_ITEMS.forEachIndexed { index, res ->
                    Icon(
                        toolbarIcons[index],
                        contentDescription = stringResource(res),
                        tint = palette.mainSecondText,
                        modifier = Modifier
                            .size(TOOLBAR_ICON_DP.dp)
                            .clickable(enabled = false) { },
                    )
                }
            }
        }
        // 输入区（底部，两态由 phase 驱动）：输入框 + 工具行
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp)
                .padding(bottom = 16.dp),
        ) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(MaterialTheme.colorScheme.surfaceVariant, RoundedCornerShape(24.dp))
                    .padding(16.dp),
            ) {
                Text(
                    text = stringResource(chatPlaceholder(phase)),
                    style = MaterialTheme.typography.bodyMedium,
                    color = palette.mainThirdText,
                )
            }
            Spacer(Modifier.height(12.dp))
            Row(verticalAlignment = Alignment.CenterVertically) {
                Icon(
                    Icons.Filled.PlayArrow,
                    contentDescription = stringResource(R.string.chat_input_voice),
                    tint = palette.mainSecondText,
                )
                Spacer(Modifier.width(12.dp))
                // 档位选择器：发送中 = ⚡ 快速 ▾；完成态 = auto
                Text(
                    text = stringResource(chatModelTierLabel(phase)),
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Spacer(Modifier.width(12.dp))
                Icon(
                    Icons.Filled.Add,
                    contentDescription = stringResource(R.string.chat_input_plus),
                    tint = MaterialTheme.colorScheme.onSurface,
                )
                Spacer(Modifier.weight(1f))
                if (phase == ChatPhase.SENDING) {
                    // 停止键：36dp 圆形（点击 = 无操作占位，两态由调用方切 phase 演示）
                    Box(
                        modifier = Modifier
                            .size(36.dp)
                            .background(MaterialTheme.colorScheme.onSurface, CircleShape)
                            .clickable { },
                    )
                }
            }
        }
    }
}
