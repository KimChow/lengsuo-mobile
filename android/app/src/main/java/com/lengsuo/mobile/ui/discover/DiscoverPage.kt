package com.lengsuo.mobile.ui.discover

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Build
import androidx.compose.material.icons.filled.DateRange
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.Star
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import com.lengsuo.mobile.R

/**
 * 发现页尺寸常量（照 mobile-ui-assets.md：外边距 16dp、组间距 12dp、行高 48dp）。
 */
const val DISCOVER_PAGE_MARGIN_DP = 16
const val DISCOVER_GROUP_SPACING_DP = 12
const val DISCOVER_ROW_HEIGHT_DP = 48
const val DISCOVER_CARD_CORNER_DP = 12
const val DISCOVER_AVATAR_SIZE_DP = 48
const val DISCOVER_BANNER_HEIGHT_DP = 48
const val DISCOVER_MARKET_CARD_COUNT = 3

/**
 * 分类受控值（卡面 §3.2 指定：全部 / 专家 / 技能）。
 * 其余分类文案全表未命中，本卡不渲染（留待文案补全卡）。
 */
const val DISCOVER_CATEGORY_ALL = "全部"
const val DISCOVER_CATEGORY_EXPERTS = "专家"
const val DISCOVER_CATEGORY_SKILLS = "技能"
val DISCOVER_CATEGORY_VALUES: List<String> =
    listOf(DISCOVER_CATEGORY_ALL, DISCOVER_CATEGORY_EXPERTS, DISCOVER_CATEGORY_SKILLS)

/**
 * 文案资源 id 常量表：界面只引用这里，测试可对 id 与 strings.xml 做逐字对照。
 */
object DiscoverCopy {
    val searchPlaceholder = R.string.discover_search_placeholder
    val allCategory = R.string.discover_all_category
    val sectionExperts = R.string.discover_section_experts
    val sectionSkills = R.string.discover_section_skills
    val featured = R.string.discover_featured
    val myFavorites = R.string.discover_my_favorites
    val emptyCategory = R.string.discover_empty_category
    val noCategoryResults = R.string.discover_no_category_results
    val noCategoryHint = R.string.discover_no_category_hint
    val noSearchResults = R.string.discover_no_search_results
    val noSearchHint = R.string.discover_no_search_hint
    val refresh = R.string.discover_refresh
    val preview = R.string.discover_preview
    val makeSame = R.string.discover_make_same
    val favorite = R.string.discover_favorite
    val favorited = R.string.discover_favorited
    val official = R.string.discover_official
    val back = R.string.discover_back
    val subtitle = R.string.discover_subtitle
    val description = R.string.discover_description
    val tagsLabel = R.string.discover_tags_label
    val library = R.string.discover_library
    val mcpDetail = R.string.discover_mcp_detail
    val project = R.string.discover_project

    val all: List<Int> = listOf(
        searchPlaceholder, allCategory, sectionExperts, sectionSkills, featured,
        myFavorites, emptyCategory, noCategoryResults, noCategoryHint, noSearchResults,
        noSearchHint, refresh, preview, makeSame, favorite, favorited, official, back,
        subtitle, description, tagsLabel, library, mcpDetail, project,
    )
}

/** 分类 chip 文案资源 id；未知分类回落到「全部」。 */
fun discoverCategoryLabelRes(category: String): Int = when (category) {
    DISCOVER_CATEGORY_EXPERTS -> DiscoverCopy.sectionExperts
    DISCOVER_CATEGORY_SKILLS -> DiscoverCopy.sectionSkills
    else -> DiscoverCopy.allCategory
}

/** chip 选中态纯函数（本地切换，无导航）。 */
fun isDiscoverCategorySelected(current: String, value: String): Boolean = current == value

/** 空态三态。 */
enum class DiscoverEmptyState { NONE, CATEGORY_EMPTY, NO_CATEGORY_RESULTS, NO_SEARCH_RESULTS }

/**
 * 空态判定纯函数：搜索词优先，其次 empty 标记，最后按分类区分。
 */
fun discoverEmptyState(category: String, query: String, empty: Boolean): DiscoverEmptyState {
    if (query.isNotBlank()) return DiscoverEmptyState.NO_SEARCH_RESULTS
    if (!empty) return DiscoverEmptyState.NONE
    return if (category == DISCOVER_CATEGORY_ALL) {
        DiscoverEmptyState.CATEGORY_EMPTY
    } else {
        DiscoverEmptyState.NO_CATEGORY_RESULTS
    }
}

/** 空态标题资源 id；无空态时返回 null。 */
fun discoverEmptyTitleRes(state: DiscoverEmptyState): Int? = when (state) {
    DiscoverEmptyState.NONE -> null
    DiscoverEmptyState.CATEGORY_EMPTY -> DiscoverCopy.emptyCategory
    DiscoverEmptyState.NO_CATEGORY_RESULTS -> DiscoverCopy.noCategoryResults
    DiscoverEmptyState.NO_SEARCH_RESULTS -> DiscoverCopy.noSearchResults
}

/** 空态副提示资源 id；仅「暂无该分类的案例」无副提示。 */
fun discoverEmptyHintRes(state: DiscoverEmptyState): Int? = when (state) {
    DiscoverEmptyState.NO_CATEGORY_RESULTS -> DiscoverCopy.noCategoryHint
    DiscoverEmptyState.NO_SEARCH_RESULTS -> DiscoverCopy.noSearchHint
    else -> null
}

/**
 * 发现页静态版：本地交互态（分类 chip 切换）+ 点击占位，无网络 / 无 COS / 无导航 / 无持久化。
 * 全部文案走 strings.xml，色值全部走 MaterialTheme token。
 */
@Composable
fun DiscoverPage(
    category: String = "全部",
    query: String = "",
    empty: Boolean = false,
) {
    var selectedCategory by remember(category) { mutableStateOf(category) }
    val emptyState = discoverEmptyState(selectedCategory, query, empty)
    val searchLabel = stringResource(DiscoverCopy.searchPlaceholder)

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(DISCOVER_PAGE_MARGIN_DP.dp),
        verticalArrangement = Arrangement.spacedBy(DISCOVER_GROUP_SPACING_DP.dp),
    ) {
        OutlinedTextField(
            value = query,
            onValueChange = {},
            singleLine = true,
            modifier = Modifier
                .fillMaxWidth()
                .semantics { contentDescription = searchLabel },
            leadingIcon = { Icon(Icons.Filled.Search, contentDescription = searchLabel) },
            placeholder = { Text(searchLabel) },
        )

        Text(
            text = stringResource(DiscoverCopy.subtitle),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )

        DiscoverLeftGroup()

        DiscoverCategoryRow(
            selected = selectedCategory,
            onSelect = { selectedCategory = it },
        )

        if (emptyState == DiscoverEmptyState.NONE) {
            LazyColumn(
                modifier = Modifier.weight(1f).fillMaxWidth(),
                verticalArrangement = Arrangement.spacedBy(DISCOVER_GROUP_SPACING_DP.dp),
            ) {
                items(DISCOVER_MARKET_CARD_COUNT) { DiscoverMarketCard() }
            }
        } else {
            DiscoverEmptyBlock(
                state = emptyState,
                modifier = Modifier.weight(1f).fillMaxWidth(),
            )
        }

        DiscoverOperationBanner()
    }
}

/** 左侧组：资料库（占位）/ 专家 / 技能 / 连接器（占位）/ 项目（占位），三连卡居中。 */
@Composable
private fun DiscoverLeftGroup() {
    Row(
        modifier = Modifier.fillMaxWidth().height(DISCOVER_ROW_HEIGHT_DP.dp),
        horizontalArrangement = Arrangement.spacedBy(DISCOVER_GROUP_SPACING_DP.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DiscoverIconEntry(Icons.Filled.Home, DiscoverCopy.library)
        DiscoverLabeledEntry(Icons.Filled.Person, DiscoverCopy.sectionExperts)
        DiscoverLabeledEntry(Icons.Filled.Build, DiscoverCopy.sectionSkills)
        DiscoverLabeledEntry(Icons.Filled.Settings, DiscoverCopy.mcpDetail)
        DiscoverIconEntry(Icons.Filled.DateRange, DiscoverCopy.project)
    }
}

/** desc 占位项：只有图标 + contentDescription，点击占位无操作。 */
@Composable
private fun DiscoverIconEntry(icon: ImageVector, labelRes: Int) {
    val label = stringResource(labelRes)
    Icon(
        imageVector = icon,
        contentDescription = label,
        tint = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = Modifier.clickable {},
    )
}

/** 文案项：图标 + 逐字文案，点击占位无操作。 */
@Composable
private fun DiscoverLabeledEntry(icon: ImageVector, labelRes: Int) {
    Row(
        horizontalArrangement = Arrangement.spacedBy(DISCOVER_GROUP_SPACING_DP.dp),
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier.clickable {},
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.primary,
        )
        Text(
            text = stringResource(labelRes),
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurface,
        )
    }
}

/** 分类 chip 行：全部 / 专家 / 技能（本地切换）。 */
@Composable
private fun DiscoverCategoryRow(selected: String, onSelect: (String) -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth().height(DISCOVER_ROW_HEIGHT_DP.dp),
        horizontalArrangement = Arrangement.spacedBy(DISCOVER_GROUP_SPACING_DP.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DISCOVER_CATEGORY_VALUES.forEach { value ->
            FilterChip(
                selected = isDiscoverCategorySelected(selected, value),
                onClick = { onSelect(value) },
                label = { Text(stringResource(discoverCategoryLabelRes(value))) },
            )
        }
    }
}

/**
 * 市场卡片占位：头像 + 名称/简介骨架 + 标签 + 官方角标。
 * 运营内容（名称 / 简介 / 标签值）由 COS 下发，不在本卡范围，故以骨架占位呈现。
 */
@Composable
private fun DiscoverMarketCard() {
    Surface(
        modifier = Modifier.fillMaxWidth().clickable {},
        shape = RoundedCornerShape(DISCOVER_CARD_CORNER_DP.dp),
        color = MaterialTheme.colorScheme.surfaceVariant,
    ) {
        Row(
            modifier = Modifier.padding(DISCOVER_GROUP_SPACING_DP.dp),
            horizontalArrangement = Arrangement.spacedBy(DISCOVER_GROUP_SPACING_DP.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                modifier = Modifier
                    .size(DISCOVER_AVATAR_SIZE_DP.dp)
                    .clip(CircleShape)
                    .background(MaterialTheme.colorScheme.primaryContainer),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    imageVector = Icons.Filled.Person,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onPrimaryContainer,
                )
            }

            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(DISCOVER_GROUP_SPACING_DP.dp),
            ) {
                DiscoverSkeletonLine(0.5f)
                DiscoverSkeletonLine(0.8f)
                Row(
                    horizontalArrangement = Arrangement.spacedBy(DISCOVER_GROUP_SPACING_DP.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(
                        text = stringResource(DiscoverCopy.tagsLabel),
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    DiscoverSkeletonChip()
                    DiscoverSkeletonChip()
                }
            }

            Row(
                horizontalArrangement = Arrangement.spacedBy(DISCOVER_GROUP_SPACING_DP.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Icon(
                    imageVector = Icons.Filled.Star,
                    contentDescription = stringResource(DiscoverCopy.featured),
                    tint = MaterialTheme.colorScheme.secondary,
                )
                Text(
                    text = stringResource(DiscoverCopy.official),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSecondaryContainer,
                    modifier = Modifier
                        .clip(RoundedCornerShape(DISCOVER_CARD_CORNER_DP.dp))
                        .background(MaterialTheme.colorScheme.secondaryContainer)
                        .padding(DISCOVER_GROUP_SPACING_DP.dp),
                )
            }
        }
    }
}

@Composable
private fun DiscoverSkeletonLine(widthFraction: Float) {
    Box(
        modifier = Modifier
            .fillMaxWidth(widthFraction)
            .height(DISCOVER_GROUP_SPACING_DP.dp)
            .clip(RoundedCornerShape(DISCOVER_CARD_CORNER_DP.dp))
            .background(MaterialTheme.colorScheme.onSurfaceVariant),
    )
}

@Composable
private fun DiscoverSkeletonChip() {
    Box(
        modifier = Modifier
            .size(DISCOVER_ROW_HEIGHT_DP.dp / 3, DISCOVER_GROUP_SPACING_DP.dp)
            .clip(RoundedCornerShape(DISCOVER_CARD_CORNER_DP.dp))
            .background(MaterialTheme.colorScheme.outlineVariant),
    )
}

/** 空态块：标题 +（可选）副提示，纯文本 Column。 */
@Composable
private fun DiscoverEmptyBlock(state: DiscoverEmptyState, modifier: Modifier = Modifier) {
    Column(
        modifier = modifier,
        verticalArrangement = Arrangement.spacedBy(DISCOVER_GROUP_SPACING_DP.dp, Alignment.CenterVertically),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        discoverEmptyTitleRes(state)?.let { titleRes ->
            Text(
                text = stringResource(titleRes),
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurface,
            )
        }
        discoverEmptyHintRes(state)?.let { hintRes ->
            Text(
                text = stringResource(hintRes),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

/** 运营 banner 占位：不加载任何远程资源。 */
@Composable
private fun DiscoverOperationBanner() {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(DISCOVER_BANNER_HEIGHT_DP.dp)
            .clip(RoundedCornerShape(DISCOVER_CARD_CORNER_DP.dp))
            .background(MaterialTheme.colorScheme.surfaceVariant),
    ) {
        Row(
            modifier = Modifier
                .fillMaxSize()
                .padding(DISCOVER_GROUP_SPACING_DP.dp),
            horizontalArrangement = Arrangement.spacedBy(DISCOVER_GROUP_SPACING_DP.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                imageVector = Icons.Filled.Refresh,
                contentDescription = stringResource(DiscoverCopy.refresh),
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Text(
                text = stringResource(DiscoverCopy.myFavorites),
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}