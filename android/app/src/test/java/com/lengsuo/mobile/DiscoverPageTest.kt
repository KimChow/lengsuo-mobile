package com.lengsuo.mobile

import com.lengsuo.mobile.ui.discover.DISCOVER_BANNER_HEIGHT_DP
import com.lengsuo.mobile.ui.discover.DISCOVER_CATEGORY_ALL
import com.lengsuo.mobile.ui.discover.DISCOVER_CATEGORY_EXPERTS
import com.lengsuo.mobile.ui.discover.DISCOVER_CATEGORY_SKILLS
import com.lengsuo.mobile.ui.discover.DISCOVER_CATEGORY_VALUES
import com.lengsuo.mobile.ui.discover.DISCOVER_GROUP_SPACING_DP
import com.lengsuo.mobile.ui.discover.DISCOVER_MARKET_CARD_COUNT
import com.lengsuo.mobile.ui.discover.DISCOVER_PAGE_MARGIN_DP
import com.lengsuo.mobile.ui.discover.DISCOVER_ROW_HEIGHT_DP
import com.lengsuo.mobile.ui.discover.DiscoverCopy
import com.lengsuo.mobile.ui.discover.DiscoverEmptyState
import com.lengsuo.mobile.ui.discover.discoverCategoryLabelRes
import com.lengsuo.mobile.ui.discover.discoverEmptyHintRes
import com.lengsuo.mobile.ui.discover.discoverEmptyState
import com.lengsuo.mobile.ui.discover.discoverEmptyTitleRes
import com.lengsuo.mobile.ui.discover.isDiscoverCategorySelected
import java.io.File

/**
 * 发现页纯数据断言。
 * 注：app 模块没有任何 testImplementation（本卡禁增 Gradle 依赖），故不引 junit；
 * 断言以纯 Kotlin 写成，main() 逐条执行，全部通过时退出码 0，否则退出码 1。
 * 文案断言 = 资源 id 映射 + strings.xml 逐字文案（JVM 侧无法解析 stringResource，
 * 直接读 values/strings.xml 原文比对，保证「文案只在 strings.xml 且逐字」）。
 */
class DiscoverPageTest {
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

    fun searchPlaceholderMapping() {
        check(
            "搜索框占位 id = discover_search_placeholder",
            DiscoverCopy.searchPlaceholder == R.string.discover_search_placeholder,
        )
    }

    fun categoryChips_verbatim() {
        check(
            "分类受控值逐字 = 全部/专家/技能, actual $DISCOVER_CATEGORY_VALUES",
            DISCOVER_CATEGORY_VALUES == listOf("全部", "专家", "技能"),
        )
        check(
            "全部 chip id = discover_all_category",
            discoverCategoryLabelRes(DISCOVER_CATEGORY_ALL) == R.string.discover_all_category,
        )
        check(
            "专家 chip id = discover_section_experts",
            discoverCategoryLabelRes(DISCOVER_CATEGORY_EXPERTS) == R.string.discover_section_experts,
        )
        check(
            "技能 chip id = discover_section_skills",
            discoverCategoryLabelRes(DISCOVER_CATEGORY_SKILLS) == R.string.discover_section_skills,
        )
        check(
            "未知分类回落「全部」",
            discoverCategoryLabelRes("other") == R.string.discover_all_category,
        )
    }

    fun categorySelection_truthTable() {
        check(
            "选中态 (全部, 全部) = true",
            isDiscoverCategorySelected(DISCOVER_CATEGORY_ALL, DISCOVER_CATEGORY_ALL),
        )
        check(
            "选中态 (全部, 专家) = false",
            !isDiscoverCategorySelected(DISCOVER_CATEGORY_ALL, DISCOVER_CATEGORY_EXPERTS),
        )
        check(
            "选中态 (技能, 技能) = true",
            isDiscoverCategorySelected(DISCOVER_CATEGORY_SKILLS, DISCOVER_CATEGORY_SKILLS),
        )
    }

    fun leftGroup_threeCards() {
        check(
            "三连卡专家 id = discover_section_experts",
            DiscoverCopy.sectionExperts == R.string.discover_section_experts,
        )
        check(
            "三连卡技能 id = discover_section_skills",
            DiscoverCopy.sectionSkills == R.string.discover_section_skills,
        )
        check(
            "三连卡连接器 id = discover_mcp_detail",
            DiscoverCopy.mcpDetail == R.string.discover_mcp_detail,
        )
    }

    fun emptyState_categoryEmpty() {
        val state = discoverEmptyState(DISCOVER_CATEGORY_ALL, query = "", empty = true)
        check(
            "全部 + empty → CATEGORY_EMPTY, actual $state",
            state == DiscoverEmptyState.CATEGORY_EMPTY,
        )
        check(
            "空态标题 id = discover_empty_category",
            discoverEmptyTitleRes(state) == R.string.discover_empty_category,
        )
        check(
            "「暂无该分类的案例」无副提示",
            discoverEmptyHintRes(state) == null,
        )
    }

    fun emptyState_noCategoryResults() {
        val state = discoverEmptyState(DISCOVER_CATEGORY_EXPERTS, query = "", empty = true)
        check(
            "专家 + empty → NO_CATEGORY_RESULTS, actual $state",
            state == DiscoverEmptyState.NO_CATEGORY_RESULTS,
        )
        check(
            "标题 id = discover_no_category_results",
            discoverEmptyTitleRes(state) == R.string.discover_no_category_results,
        )
        check(
            "副提示 id = discover_no_category_hint",
            discoverEmptyHintRes(state) == R.string.discover_no_category_hint,
        )
    }

    fun emptyState_noSearchResults() {
        val state = discoverEmptyState(DISCOVER_CATEGORY_ALL, query = "x", empty = false)
        check(
            "有搜索词 → NO_SEARCH_RESULTS, actual $state",
            state == DiscoverEmptyState.NO_SEARCH_RESULTS,
        )
        check(
            "标题 id = discover_no_search_results",
            discoverEmptyTitleRes(state) == R.string.discover_no_search_results,
        )
        check(
            "副提示 id = discover_no_search_hint",
            discoverEmptyHintRes(state) == R.string.discover_no_search_hint,
        )
    }

    fun emptyState_none() {
        check(
            "无搜索词 + 非 empty → NONE",
            discoverEmptyState(DISCOVER_CATEGORY_ALL, query = "", empty = false) ==
                DiscoverEmptyState.NONE,
        )
    }

    fun sizes_matchSpec() {
        check(
            "外边距 DISCOVER_PAGE_MARGIN_DP == 16, actual $DISCOVER_PAGE_MARGIN_DP",
            DISCOVER_PAGE_MARGIN_DP == 16,
        )
        check(
            "组间距 DISCOVER_GROUP_SPACING_DP == 12, actual $DISCOVER_GROUP_SPACING_DP",
            DISCOVER_GROUP_SPACING_DP == 12,
        )
        check(
            "行高 DISCOVER_ROW_HEIGHT_DP == 48, actual $DISCOVER_ROW_HEIGHT_DP",
            DISCOVER_ROW_HEIGHT_DP == 48,
        )
        check(
            "banner 高 DISCOVER_BANNER_HEIGHT_DP == 48, actual $DISCOVER_BANNER_HEIGHT_DP",
            DISCOVER_BANNER_HEIGHT_DP == 48,
        )
        check(
            "市场卡片数 DISCOVER_MARKET_CARD_COUNT == 3, actual $DISCOVER_MARKET_CARD_COUNT",
            DISCOVER_MARKET_CARD_COUNT == 3,
        )
    }

    fun stringsXml_copyIsVerbatim() {
        val xml = loadStringsXml()
        val expected = listOf(
            "discover_search_placeholder\">搜索灵感<",
            "discover_all_category\">全部<",
            "discover_section_experts\">专家<",
            "discover_section_skills\">技能<",
            "discover_featured\">精选<",
            "discover_my_favorites\">我的收藏<",
            "discover_empty_category\">暂无该分类的案例<",
            "discover_no_category_results\">当前分类下暂无内容<",
            "discover_no_category_hint\">切换其他分类看看<",
            "discover_no_search_results\">未找到匹配的结果<",
            "discover_no_search_hint\">试试其他关键词<",
            "discover_refresh\">换一换<",
            "discover_preview\">预览<",
            "discover_make_same\">做同款<",
            "discover_favorite\">收藏<",
            "discover_favorited\">已收藏<",
            "discover_official\">官方<",
            "discover_back\">返回<",
            "discover_subtitle\">常见工作流沉淀成可复用的任务起点<",
            "discover_description\">详细介绍<",
            "discover_tags_label\">标签类别<",
            "discover_library\">资料库<",
            "discover_mcp_detail\">连接器详情<",
            "discover_project\">项目<",
        )
        expected.forEach { needle ->
            check(
                "strings.xml 缺少逐字条目: $needle",
                needle in xml,
            )
        }
        check(
            "DiscoverCopy 覆盖全部 24 条 discover_* 资源 id",
            DiscoverCopy.all == listOf(
                R.string.discover_search_placeholder,
                R.string.discover_all_category,
                R.string.discover_section_experts,
                R.string.discover_section_skills,
                R.string.discover_featured,
                R.string.discover_my_favorites,
                R.string.discover_empty_category,
                R.string.discover_no_category_results,
                R.string.discover_no_category_hint,
                R.string.discover_no_search_results,
                R.string.discover_no_search_hint,
                R.string.discover_refresh,
                R.string.discover_preview,
                R.string.discover_make_same,
                R.string.discover_favorite,
                R.string.discover_favorited,
                R.string.discover_official,
                R.string.discover_back,
                R.string.discover_subtitle,
                R.string.discover_description,
                R.string.discover_tags_label,
                R.string.discover_library,
                R.string.discover_mcp_detail,
                R.string.discover_project,
            ),
        )
    }

    companion object {
        @JvmStatic
        fun main(args: Array<String>) {
            val test = DiscoverPageTest()
            test.searchPlaceholderMapping()
            test.categoryChips_verbatim()
            test.categorySelection_truthTable()
            test.leftGroup_threeCards()
            test.emptyState_categoryEmpty()
            test.emptyState_noCategoryResults()
            test.emptyState_noSearchResults()
            test.emptyState_none()
            test.sizes_matchSpec()
            test.stringsXml_copyIsVerbatim()
            if (test.failures.isEmpty()) {
                println("DiscoverPageTest: all checks passed")
            } else {
                test.failures.forEach { println("FAILED: $it") }
                kotlin.system.exitProcess(1)
            }
        }
    }
}