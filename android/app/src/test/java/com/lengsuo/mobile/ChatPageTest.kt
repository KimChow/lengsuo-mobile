package com.lengsuo.mobile

import com.lengsuo.mobile.ui.chat.CHAT_SAMPLE_AI_REPLY
import com.lengsuo.mobile.ui.chat.CHAT_SAMPLE_TITLE
import com.lengsuo.mobile.ui.chat.CHAT_SAMPLE_USER_MESSAGE
import com.lengsuo.mobile.ui.chat.CHAT_TOOLBAR_ITEMS
import com.lengsuo.mobile.ui.chat.ChatPhase
import com.lengsuo.mobile.ui.chat.TOOLBAR_GAP_DP
import com.lengsuo.mobile.ui.chat.TOOLBAR_ICON_DP
import com.lengsuo.mobile.ui.chat.chatModelTierLabel
import com.lengsuo.mobile.ui.chat.chatPlaceholder
import com.lengsuo.mobile.ui.chat.chatStatusLabel
import java.io.File

/**
 * 对话页纯数据断言。
 * 注：app 模块没有任何 testImplementation（本卡禁增 Gradle 依赖），故不引 junit；
 * 断言以纯 Kotlin 写成，main() 逐条执行，全部通过时退出码 0，否则退出码 1。
 * 两态文案映射断言 = 资源 id 映射 + strings.xml 逐字文案（JVM 侧无法解析 stringResource，
 * 直接读 values/strings.xml 原文比对，保证「文案只在 strings.xml 且逐字」）。
 */
class ChatPageTest {
    private val failures = mutableListOf<String>()

    private fun check(name: String, condition: Boolean) {
        if (!condition) failures += name
    }

    private fun loadStringsXml(): String {
        val file = listOf(
            "app/src/main/res/values/strings.xml",
            "src/main/res/values/strings.xml",
        ).mapNotNull { File(it).takeIf { f -> f.exists() } }.firstOrNull()
            ?: error("strings.xml not found; run from android/ or app/ dir")
        return file.readText()
    }

    fun b1_titleEqualsFirstUserMessage() {
        check(
            "B1 标题 == 首条用户消息, actual $CHAT_SAMPLE_TITLE vs $CHAT_SAMPLE_USER_MESSAGE",
            CHAT_SAMPLE_TITLE == CHAT_SAMPLE_USER_MESSAGE,
        )
    }

    fun b3_statusLineMapping() {
        check(
            "B3 发送态状态行 id = chat_status_thinking",
            chatStatusLabel(ChatPhase.SENDING) == R.string.chat_status_thinking,
        )
        check(
            "B3 完成态状态行 id = chat_status_done",
            chatStatusLabel(ChatPhase.DONE) == R.string.chat_status_done,
        )
    }

    fun b3_placeholderMapping() {
        check(
            "发送态输入占位 id = chat_input_placeholder_sending",
            chatPlaceholder(ChatPhase.SENDING) == R.string.chat_input_placeholder_sending,
        )
        check(
            "完成态输入占位 id = chat_input_placeholder_idle",
            chatPlaceholder(ChatPhase.DONE) == R.string.chat_input_placeholder_idle,
        )
    }

    fun b7_modelTierMapping() {
        check(
            "B7 发送态档位 id = chat_model_tier_sending",
            chatModelTierLabel(ChatPhase.SENDING) == R.string.chat_model_tier_sending,
        )
        check(
            "B7 完成态档位 id = chat_model_tier_idle",
            chatModelTierLabel(ChatPhase.DONE) == R.string.chat_model_tier_idle,
        )
    }

    fun b4_toolbarFiveItemsInOrder() {
        check(
            "B4 工具条 5 键 id 列表逐字（复制/赞/踩/朗读/分享 顺序）",
            CHAT_TOOLBAR_ITEMS == listOf(
                R.string.chat_toolbar_copy,
                R.string.chat_toolbar_like,
                R.string.chat_toolbar_dislike,
                R.string.chat_toolbar_read_aloud,
                R.string.chat_toolbar_share,
            ),
        )
        check(
            "B4 工具条键数 == 5, actual ${CHAT_TOOLBAR_ITEMS.size}",
            CHAT_TOOLBAR_ITEMS.size == 5,
        )
    }

    fun b4_toolbarIconSizeAndGap() {
        check(
            "B4 键尺寸 TOOLBAR_ICON_DP == 24, actual $TOOLBAR_ICON_DP",
            TOOLBAR_ICON_DP == 24,
        )
        check(
            "B4 键间距 TOOLBAR_GAP_DP == 12, actual $TOOLBAR_GAP_DP",
            TOOLBAR_GAP_DP == 12,
        )
    }

    fun stringsXml_copyIsVerbatim() {
        val xml = loadStringsXml()
        val expected = listOf(
            "chat_ai_banner\">内容由 AI 生成<",
            "chat_status_done\">已完成<",
            "chat_status_thinking\">正在思考 ⋯<",
            "chat_toolbar_copy\">复制<",
            "chat_toolbar_like\">赞<",
            "chat_toolbar_dislike\">踩<",
            "chat_toolbar_read_aloud\">朗读<",
            "chat_toolbar_share\">分享<",
            "chat_top_new_chat\">新建对话<",
            "chat_top_more\">更多操作<",
            "chat_subtitle_mode\">☁ 工作模式<",
            "chat_input_placeholder_idle\">发消息或按住说话...<",
            "chat_input_placeholder_sending\">发消息...<",
            "chat_model_tier_idle\">auto<",
            "chat_model_tier_sending\">⚡ 快速 ▾<",
            "chat_input_voice\">语音<",
            "chat_input_plus\">+<",
        )
        expected.forEach { needle ->
            check(
                "strings.xml 缺少逐字条目: $needle",
                needle in xml,
            )
        }
    }

    fun sampleData_sanity() {
        check(
            "示例消息非空",
            CHAT_SAMPLE_USER_MESSAGE.isNotBlank() && CHAT_SAMPLE_AI_REPLY.isNotBlank(),
        )
        check(
            "示例标题与消息一致（自动命名）",
            CHAT_SAMPLE_TITLE == CHAT_SAMPLE_USER_MESSAGE,
        )
    }

    companion object {
        @JvmStatic
        fun main(args: Array<String>) {
            val test = ChatPageTest()
            test.b1_titleEqualsFirstUserMessage()
            test.b3_statusLineMapping()
            test.b3_placeholderMapping()
            test.b7_modelTierMapping()
            test.b4_toolbarFiveItemsInOrder()
            test.b4_toolbarIconSizeAndGap()
            test.stringsXml_copyIsVerbatim()
            test.sampleData_sanity()
            if (test.failures.isEmpty()) {
                println("ChatPageTest: all checks passed")
            } else {
                test.failures.forEach { println("FAILED: $it") }
                kotlin.system.exitProcess(1)
            }
        }
    }
}
