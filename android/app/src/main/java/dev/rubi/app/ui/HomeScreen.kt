package dev.rubi.app.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.itemsIndexed
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import dev.rubi.app.RubiApp
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiBreakdown
import dev.rubi.core.FfiSignal
import dev.rubi.core.FfiVideoCard
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

private const val PAGE = 20u

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeScreen(app: RubiApp, nav: Nav) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    val cards = remember { mutableStateListOf<FfiVideoCard>() }
    var status by remember { mutableStateOf("") }
    var refreshing by remember { mutableStateOf(false) }
    var dual by remember { mutableStateOf(true) }
    var why by remember { mutableStateOf<FfiVideoCard?>(null) }
    var pivot by remember { mutableStateOf(0) }
    val grid = rememberLazyGridState()

    suspend fun load(reset: Boolean) {
        refreshing = true
        val report = runCatching { app.core.refreshFeed(20u) }
        val page = withContext(Dispatchers.IO) { runCatching { app.core.nextPage(PAGE) }.getOrDefault(emptyList()) }
        if (reset) cards.clear()
        cards.addAll(page.filter { p -> cards.none { it.bvid == p.bvid } })
        status = report.fold(
            { r -> "↻ 本次保留 ${r.kept} · 过滤 ${r.blocked}" + if (r.errors.isNotEmpty()) " · ${r.errors.size} 个来源失败" else "" },
            { e -> "刷新失败：${e.message}" },
        )
        refreshing = false
    }

    LaunchedEffect(Unit) { if (cards.isEmpty()) load(reset = true) }
    val nearEnd by remember { derivedStateOf { grid.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0 >= cards.size - 4 } }
    LaunchedEffect(nearEnd) { if (nearEnd && cards.isNotEmpty() && !refreshing) load(reset = false) }

    Column(Modifier.fillMaxSize()) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.weight(1f)) { Pivot(listOf("推荐", "热门", "排行"), pivot, { pivot = it }) }
            Text(
                if (dual) "▦ 双列" else "▭ 单列",
                style = MetroType.meta.copy(fontSize = 12.sp),
                color = c.fg,
                modifier = Modifier.padding(end = 16.dp).clickable { dual = !dual },
            )
        }
        if (pivot != 0) {
            HotRankList(app, nav, ranking = pivot == 2, dual = dual)
            return@Column
        }
        Row(Modifier.fillMaxWidth().height(44.dp).padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Meta(status, Modifier.weight(1f))
            Text("被过滤内容 →", style = MetroType.meta, color = c.fg, modifier = Modifier.clickable { nav.push(Screen.Filtered) })
        }
        PullToRefreshBox(isRefreshing = refreshing, onRefresh = { scope.launch { load(reset = true) } }) {
            LazyVerticalGrid(
                columns = GridCells.Fixed(if (dual) 2 else 1),
                state = grid,
                contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 28.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                verticalArrangement = Arrangement.spacedBy(if (dual) 22.dp else 30.dp),
                modifier = Modifier.fillMaxSize(),
            ) {
                items(cards, key = { it.bvid }) { card ->
                    VideoCardView(card, big = !dual, onOpen = { nav.push(Screen.Player(card.bvid, card.title)) }, onMore = { why = card })
                }
                if (cards.isEmpty() && !refreshing) {
                    item(span = { GridItemSpan(maxLineSpan) }) { Text("候选池为空，下拉刷新", color = c.fg2) }
                }
            }
        }
    }

    why?.let { card ->
        WhySheet(app, card, onDismiss = { why = null }, onDisliked = {
            cards.removeAll { it.bvid == card.bvid }
            why = null
        })
    }
}

/** 热门 (paged grid) and 排行 (numbered list); both already filtered by the rule table. */
@Composable
private fun HotRankList(app: RubiApp, nav: Nav, ranking: Boolean, dual: Boolean) {
    val c = LocalMetro.current
    val cards = remember(ranking) { mutableStateListOf<FfiVideoCard>() }
    var page by remember(ranking) { mutableStateOf(1u) }
    var loading by remember(ranking) { mutableStateOf(false) }
    var error by remember(ranking) { mutableStateOf<String?>(null) }
    val grid = rememberLazyGridState()

    suspend fun more() {
        if (loading) return
        loading = true
        runCatching { if (ranking) app.core.ranking() else app.core.popular(page) }
            .onSuccess { list ->
                cards.addAll(list.filter { n -> cards.none { it.bvid == n.bvid } })
                page++
            }
            .onFailure { error = it.message }
        loading = false
    }

    LaunchedEffect(ranking) { more() }
    if (!ranking) {
        val nearEnd by remember { derivedStateOf { (grid.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0) >= cards.size - 4 } }
        LaunchedEffect(nearEnd) { if (nearEnd && cards.isNotEmpty() && page <= 10u) more() }
    }

    LazyVerticalGrid(
        columns = GridCells.Fixed(if (dual && !ranking) 2 else 1),
        state = grid,
        contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 8.dp, bottom = 28.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalArrangement = Arrangement.spacedBy(if (ranking) 4.dp else 22.dp),
        modifier = Modifier.fillMaxSize(),
    ) {
        error?.let { e -> item(span = { GridItemSpan(maxLineSpan) }) { Text("加载失败：$e", color = c.fg2, fontSize = 13.sp) } }
        if (ranking) {
            itemsIndexed(cards, key = { _, it -> it.bvid }) { i, card ->
                Row(
                    Modifier.fillMaxWidth().clickable { nav.push(Screen.Player(card.bvid, card.title)) }.padding(vertical = 10.dp),
                    horizontalArrangement = Arrangement.spacedBy(14.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(
                        "%02d".format(i + 1),
                        style = MetroType.meta.copy(fontSize = 28.sp),
                        color = if (i < 3) c.fg else c.fg2,
                        fontWeight = if (i < 3) androidx.compose.ui.text.font.FontWeight.SemiBold else androidx.compose.ui.text.font.FontWeight.Light,
                        modifier = Modifier.width(48.dp),
                    )
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        Text(card.title, fontSize = 15.sp, color = c.fg, maxLines = 2, overflow = TextOverflow.Ellipsis)
                        Meta("${card.upName} · ${formatCount(card.views)}")
                    }
                }
            }
        } else {
            items(cards, key = { it.bvid }) { card ->
                VideoCardView(card, big = !dual, onOpen = { nav.push(Screen.Player(card.bvid, card.title)) })
            }
        }
    }
}

@Composable
fun VideoCardView(card: FfiVideoCard, big: Boolean, onOpen: () -> Unit, onMore: (() -> Unit)? = null) {
    val c = LocalMetro.current
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Column(Modifier.clickable(onClick = onOpen), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Box(Modifier.fillMaxWidth().aspectRatio(16f / 9f).background(c.bg2)) {
                AsyncImage(model = card.cover, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
                if (card.reason.isNotEmpty()) Tag(card.reason, Modifier.align(Alignment.TopStart))
                Text(
                    formatDuration(card.durationS),
                    style = MetroType.meta,
                    color = Color.White,
                    modifier = Modifier.align(Alignment.BottomEnd).background(Color(0xBF000000)).padding(horizontal = 6.dp),
                )
            }
            Text(
                card.title,
                style = if (big) MetroType.body.copy(fontSize = 18.sp, lineHeight = 26.sp) else MetroType.body,
                color = c.fg,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
        }
        Row(verticalAlignment = Alignment.Top) {
            Column(Modifier.weight(1f)) {
                Text(card.upName, fontSize = 12.sp, color = c.fg2, maxLines = 1)
                Meta("${formatCount(card.views)} · ${formatAge(card.pubdate)}")
            }
            if (onMore != null) {
                Box(Modifier.size(44.dp).clickable(onClick = onMore), contentAlignment = Alignment.Center) { GlyphIcon(Glyph.More, size = 18.dp) }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class, androidx.compose.foundation.layout.ExperimentalLayoutApi::class)
@Composable
private fun WhySheet(app: RubiApp, card: FfiVideoCard, onDismiss: () -> Unit, onDisliked: () -> Unit) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    var breakdown by remember { mutableStateOf<FfiBreakdown?>(null) }
    LaunchedEffect(card.bvid) { breakdown = withContext(Dispatchers.IO) { runCatching { app.core.explain(card.bvid) }.getOrNull() } }

    ModalBottomSheet(onDismissRequest = onDismiss, containerColor = c.bg2, shape = androidx.compose.ui.graphics.RectangleShape, dragHandle = null) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Row(verticalAlignment = Alignment.Bottom) {
                Text("为什么推荐", style = MetroType.section, color = c.fg, modifier = Modifier.weight(1f))
                Text(breakdown?.let { "%.2f".format(it.total) } ?: "", style = MetroType.meta.copy(fontSize = 28.sp), color = c.fg)
            }
            Text(card.title, color = c.fg2, fontSize = 14.sp, maxLines = 2)
            breakdown?.parts?.forEach { p ->
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text(p.factor, fontSize = 13.sp, color = c.fg, modifier = Modifier.width(72.dp))
                    Box(Modifier.weight(1f).height(6.dp).background(c.line)) {
                        val frac = (kotlin.math.abs(p.contribution) / 0.4f).coerceIn(0f, 1f)
                        Box(Modifier.fillMaxWidth(frac).height(6.dp).background(if (p.contribution >= 0) c.accent else c.fg2))
                    }
                    Meta("%+.2f".format(p.contribution), Modifier.width(52.dp), c.fg)
                }
            }
            Meta("得分 = Σ 权重 × 归一化因子 · 权重可在设置中调整")
            Text("不感兴趣", fontSize = 16.sp, color = c.fg, modifier = Modifier.padding(top = 6.dp))
            FlowRow(horizontalArrangement = Arrangement.spacedBy(20.dp)) {
                listOf(
                    "这个 UP 主" to FfiSignal.DislikeUp,
                    "这个分区" to FfiSignal.DislikeZone,
                    "这个标签" to FfiSignal.DislikeTag(""),
                    "这类标题" to FfiSignal.DislikeTitle,
                ).forEach { (label, signal) ->
                    TextAction(label, onClick = {
                        scope.launch {
                            runCatching { app.core.record(card.bvid, signal) }
                            onDisliked()
                        }
                    })
                }
            }
        }
    }
}
