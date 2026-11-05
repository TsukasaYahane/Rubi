package dev.rubi.app.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.PgcService
import dev.rubi.core.FfiPgcChoice
import dev.rubi.core.FfiPgcConditions
import dev.rubi.core.FfiPgcDay
import dev.rubi.core.FfiPgcDetail
import dev.rubi.core.FfiPgcEpisode
import dev.rubi.core.FfiPgcSeason
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/** Filters shown above the list, by field: 风格, 地区, 付费, 年份 (the rest are rarely wanted). */
private val SHOWN_FILTERS = listOf("style_id", "area", "season_status", "year", "release_date", "producer_id")

/** The poster corner's colour for 会员 / 付费 marks; other marks (独家…) are Cobalt. */
internal fun badgeColor(badge: String) = if (badge.contains("会员") || badge.contains("付费") || badge.contains("点播")) LiveRed else Color(0xFF0050EF)

/**
 * What a 专区 page has loaded and where it was, kept with its place in the back stack: coming
 * back from a season or a video shows it as it was instead of loading it again.
 */
class PgcZoneState(private val pgc: PgcService, private val seasonType: UInt) {
    var conditions by mutableStateOf<FfiPgcConditions?>(null)
        private set
    var timeline by mutableStateOf(emptyList<FfiPgcDay>())
        private set
    var recent by mutableStateOf(emptyList<FfiPgcSeason>())
        private set
    val chosen = mutableStateMapOf<String, String>()
    var order by mutableStateOf("")
    val opened = mutableStateMapOf<ULong, FfiPgcDetail?>()
    val list = LazyListState()
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
    private var loaded = false
    val pager = Paged<FfiPgcSeason, UInt>(scope, key = { it.seasonId }, minGapMs = 600L) { at ->
        val page = at ?: 1u
        val filters = chosen.filter { it.value != "-1" }.map { (field, value) -> FfiPgcChoice(value, field) }
        val r = withContext(Dispatchers.IO) { pgc.pgcIndex(seasonType, filters, order, page) }
        Page(r.seasons, if (r.hasMore) page + 1u else null)
    }

    /** Loads the page the first time it shows; later visits keep what is there. */
    suspend fun load() {
        if (loaded) return
        loaded = true
        val cond = runCatching { withContext(Dispatchers.IO) { pgc.pgcConditions(seasonType) } }.getOrNull()
        conditions = cond
        order = cond?.orders?.firstOrNull()?.value ?: "0"
        pager.reset()
        timeline = runCatching { withContext(Dispatchers.IO) { pgc.pgcTimeline(seasonType) } }.getOrDefault(emptyList())
        if (timeline.isEmpty()) {
            // No timeline for this 专区: the latest additions stand in.
            val latest = cond?.orders?.firstOrNull { it.name.contains("更新") || it.name.contains("上线") || it.name.contains("时间") }?.value ?: "0"
            recent = runCatching { withContext(Dispatchers.IO) { pgc.pgcIndex(seasonType, emptyList(), latest, 1u).seasons.take(10) } }.getOrDefault(emptyList())
        }
    }

    fun toggle(id: ULong) {
        if (opened.containsKey(id)) opened.remove(id)
        else {
            opened[id] = null
            scope.launch { opened[id] = runCatching { withContext(Dispatchers.IO) { pgc.pgcSeason(id) } }.getOrNull() }
        }
    }
}

/**
 * A 专区 (番剧 / 电影 / 纪录片 / 电视剧): 近期上映 on top, filters and sort as chips, then one card
 * per season — poster with its 会员 / 付费 mark, title, score, date, and details that unfold.
 */
@Composable
fun PgcZoneScreen(pgc: PgcService, nav: Nav, name: String, state: PgcZoneState) {
    val c = LocalMetro.current
    val conditions = state.conditions
    val timeline = state.timeline
    val recent = state.recent
    val chosen = state.chosen
    val order = state.order
    val opened = state.opened
    val list = state.list
    val pager = state.pager
    LaunchedEffect(state) { state.load() }
    LoadMoreNearEnd(list, pager)

    Column(Modifier.fillMaxSize()) {
        PageHeader(nav, name)
        LazyColumn(Modifier.weight(1f), state = list, contentPadding = PaddingValues(bottom = 24.dp)) {
            item(key = "recent") {
                Meta(if (timeline.isNotEmpty()) "近期上映" else "近期上线", Modifier.padding(start = 16.dp, top = 6.dp, bottom = 4.dp))
                Row(Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    if (timeline.isNotEmpty()) {
                        timeline.forEach { day ->
                            day.releases.forEach { r ->
                                TimelineItem(day.date + " " + r.time, r.title, r.index, r.cover, today = day.today) {
                                    nav.push(Screen.PgcSeason(r.seasonId, r.title, PgcSeasonState(pgc, r.seasonId)))
                                }
                            }
                        }
                    } else {
                        recent.forEach { s -> TimelineItem(s.orderText.ifEmpty { s.indexShow }, s.title, s.indexShow, s.cover, today = false) { nav.push(Screen.PgcSeason(s.seasonId, s.title, PgcSeasonState(pgc, s.seasonId))) } }
                    }
                }
            }
            item(key = "filters") {
                val cond = conditions
                Column(Modifier.padding(top = 10.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    cond?.filters?.filter { it.field in SHOWN_FILTERS }?.forEach { f ->
                        ChipRow(f.values.map { it.name }, f.values.indexOfFirst { it.value == (chosen[f.field] ?: "-1") }.coerceAtLeast(0)) { i ->
                            chosen[f.field] = f.values[i].value
                            pager.reset()
                        }
                    }
                    if (cond != null && cond.orders.isNotEmpty()) {
                        ChipRow(cond.orders.map { it.name }, cond.orders.indexOfFirst { it.value == order }.coerceAtLeast(0)) { i ->
                            state.order = cond.orders[i].value
                            pager.reset()
                        }
                    }
                }
            }
            items(pager.items, key = { it.seasonId.toString() }) { s ->
                SeasonCard(
                    s,
                    detail = opened[s.seasonId],
                    open = opened.containsKey(s.seasonId),
                    onToggle = { state.toggle(s.seasonId) },
                    onOpen = { nav.push(Screen.PgcSeason(s.seasonId, s.title, PgcSeasonState(pgc, s.seasonId))) },
                )
            }
            item {
                Box(Modifier.padding(16.dp)) {
                    val e = pager.error
                    when {
                        e != null -> TextAction("加载失败，重试", onClick = pager::retry)
                        pager.loading -> Meta("加载中…")
                        pager.ended && pager.items.isEmpty() -> Meta("没有符合条件的内容")
                        pager.ended -> Meta("没有更多了")
                    }
                }
            }
        }
    }
}

@Composable
private fun TimelineItem(time: String, title: String, index: String, cover: String, today: Boolean, onClick: () -> Unit) {
    val c = LocalMetro.current
    Column(Modifier.width(104.dp).clickable(onClick = onClick)) {
        Box(Modifier.fillMaxWidth().height(2.dp).background(if (today) c.accent else c.line))
        Text(time, fontSize = 12.sp, color = if (today) c.accent else c.fg2, maxLines = 1, modifier = Modifier.padding(top = 4.dp))
        AsyncImage(model = cover, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.padding(top = 4.dp).size(104.dp).background(c.bg2))
        Text(title, fontSize = 12.sp, color = c.fg, maxLines = 1, overflow = TextOverflow.Ellipsis)
        Meta(index)
    }
}

@Composable
private fun ChipRow(labels: List<String>, selected: Int, onSelect: (Int) -> Unit) {
    val c = LocalMetro.current
    Row(Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        labels.forEachIndexed { i, label ->
            val on = i == selected
            Text(
                label,
                fontSize = 12.sp,
                color = if (on) Color.White else c.fg,
                modifier = Modifier.then(if (on) Modifier.background(c.accent) else Modifier.border(1.dp, c.line))
                    .clickable { onSelect(i) }
                    .padding(horizontal = 9.dp, vertical = 4.dp),
            )
        }
    }
}

@Composable
internal fun Poster(cover: String, badge: String, width: androidx.compose.ui.unit.Dp) {
    val c = LocalMetro.current
    Box(Modifier.width(width).height(width * 4 / 3).background(c.bg2)) {
        AsyncImage(model = cover, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
        if (badge.isNotEmpty()) {
            Text(badge, fontSize = 11.sp, color = Color.White, modifier = Modifier.align(Alignment.TopEnd).background(badgeColor(badge)).padding(horizontal = 5.dp, vertical = 1.dp))
        }
    }
}

@Composable
private fun SeasonCard(s: FfiPgcSeason, detail: FfiPgcDetail?, open: Boolean, onToggle: () -> Unit, onOpen: () -> Unit) {
    val c = LocalMetro.current
    Box(Modifier.fillMaxWidth().height(1.dp).background(c.line))
    Row(Modifier.fillMaxWidth().clickable(onClick = onOpen).padding(horizontal = 16.dp, vertical = 10.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        Poster(s.cover, s.badge, 84.dp)
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(s.title, fontSize = 15.sp, color = c.fg, maxLines = 2, overflow = TextOverflow.Ellipsis)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                if (s.score.isNotEmpty()) Text("★ ${s.score}", fontSize = 13.sp, color = Color(0xFFFFB020))
                Meta(listOf(s.indexShow, s.orderText).filter { it.isNotBlank() }.joinToString(" · "))
            }
            if (s.subtitle.isNotBlank()) Meta(s.subtitle)
            TextAction(if (open) "收起详情" else "展开详情", onClick = onToggle, dim = true)
            if (open) {
                if (detail == null) Meta("读取详情…")
                else {
                    Meta(listOf(detail.pubTime, detail.areas.joinToString("、"), detail.styles.joinToString("、")).filter { it.isNotBlank() }.joinToString(" · "))
                    Text(detail.evaluate, fontSize = 12.sp, lineHeight = 18.sp, color = c.fg2, maxLines = 6, overflow = TextOverflow.Ellipsis)
                }
            }
        }
    }
}

/** A season page's details and list place, kept with it in the back stack like [PgcZoneState]. */
class PgcSeasonState(private val pgc: PgcService, private val seasonId: ULong) {
    var detail by mutableStateOf<FfiPgcDetail?>(null)
        private set
    var error by mutableStateOf<String?>(null)
        private set
    val list = LazyListState()

    suspend fun load() {
        if (detail != null) return
        error = null
        runCatching { withContext(Dispatchers.IO) { pgc.pgcSeason(seasonId) } }.onSuccess { detail = it }.onFailure { error = errorText(it) }
    }
}

/**
 * A season: poster and facts, the synopsis, and its episodes. An episode opens on the video page
 * only when the account may watch all of it; otherwise a note says to get the rights on B站's
 * site or official app (nothing is sold or unlocked here).
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PgcSeasonScreen(pgc: PgcService, nav: Nav, title: String, state: PgcSeasonState) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    val detail = state.detail
    val error = state.error
    var blocked by remember { mutableStateOf<String?>(null) }
    var checking by remember { mutableStateOf<ULong?>(null) }
    LaunchedEffect(state) { state.load() }
    fun play(ep: FfiPgcEpisode) {
        if (checking != null) return
        checking = ep.epId
        scope.launch {
            // Asked first, so a locked episode never opens a page that cannot play.
            runCatching { withContext(Dispatchers.IO) { pgc.pgcPlay(ep.epId, ep.cid, 16u) } }
                .onSuccess {
                    val d = detail
                    nav.push(Screen.Player(ep.bvid, "${d?.title ?: title} ${ep.title} ${ep.longTitle}".trim(), episode = ep.epId, episodeCid = ep.cid))
                }
                .onFailure { blocked = (it as? dev.rubi.core.RubiException.Invalid)?.msg ?: errorText(it) }
            checking = null
        }
    }

    Column(Modifier.fillMaxSize()) {
        PageHeader(nav, null)
        val d = detail
        if (d == null) {
            Meta(error ?: "加载中…", Modifier.padding(16.dp))
            return@Column
        }
        LazyColumn(Modifier.weight(1f), state = state.list, contentPadding = PaddingValues(start = 16.dp, end = 16.dp, bottom = 24.dp)) {
            item {
                Row(horizontalArrangement = Arrangement.spacedBy(14.dp)) {
                    Poster(d.cover, "", 112.dp)
                    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        Text(d.title, style = MetroType.title, color = c.fg)
                        if (d.score.isNotEmpty()) Text("★ ${d.score}", fontSize = 15.sp, fontWeight = FontWeight.SemiBold, color = Color(0xFFFFB020))
                        Meta(d.pubTime)
                        Meta(listOf(d.areas.joinToString("、"), d.styles.joinToString("、")).filter { it.isNotBlank() }.joinToString(" · "))
                        if (!d.allowDownload) Meta("官方不允许离线缓存")
                    }
                }
                Text(d.evaluate, fontSize = 13.sp, lineHeight = 20.sp, color = c.fg2, modifier = Modifier.padding(top = 12.dp))
                Text("选集 · ${d.episodes.size}", fontSize = 18.sp, fontWeight = FontWeight.Light, color = c.fg, modifier = Modifier.padding(top = 16.dp, bottom = 6.dp))
            }
            items(d.episodes, key = { it.epId.toString() }) { ep ->
                Row(
                    Modifier.fillMaxWidth().clickable { play(ep) }.padding(vertical = 10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    Text(ep.title, fontSize = 14.sp, color = c.fg, modifier = Modifier.width(56.dp), maxLines = 1, overflow = TextOverflow.Ellipsis)
                    Text(ep.longTitle, fontSize = 14.sp, color = c.fg, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f))
                    if (ep.badge.isNotEmpty()) Text(ep.badge, fontSize = 11.sp, color = Color.White, modifier = Modifier.background(badgeColor(ep.badge)).padding(horizontal = 5.dp, vertical = 1.dp))
                    Meta(if (checking == ep.epId) "检查中…" else formatDuration((ep.durationMs / 1000u).toUInt()))
                }
            }
        }
    }
    blocked?.let { msg ->
        ModalBottomSheet(onDismissRequest = { blocked = null }, containerColor = c.bg2, shape = RectangleShape, dragHandle = null) {
            Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                Text("暂时不能在 Rubi 观看", style = MetroType.section, color = c.fg)
                Text(msg, fontSize = 14.sp, lineHeight = 21.sp, color = c.fg2)
                Meta("Rubi 不提供付费入口；开通或购买后回到这里即可观看。")
                Row(Modifier.padding(bottom = 8.dp)) { AccentButton("知道了") { blocked = null } }
            }
        }
    }
}
