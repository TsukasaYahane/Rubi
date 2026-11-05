package dev.rubi.app.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.core.SeasonsService
import dev.rubi.core.PlayQueueService
import dev.rubi.core.FfiPlayingList
import dev.rubi.core.FfiSeasonSub
import dev.rubi.core.FfiVideoCard
import dev.rubi.core.RubiException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * 一键播放 / 全部播放: videos already shown (a 收藏夹, 稍后再看) played through as a list that is
 * not kept. The video page shows it as its 播放队列; the list goes when no video page playing
 * it is left (see [dropTempWhenDone]).
 */
object TempPlay {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
    /** The start under way (looking up the first video, then reading later pages); a new start cancels it. */
    private var loading: Job? = null

    /** Most pages of a 收藏夹 read in after the first ones. */
    private const val MAX_PAGES = 50

    /**
     * Starts `bvids` under `name`. `page` reads page n (from 2) of the rest, if any, and says
     * whether more follow; those join the list while it plays.
     */
    fun start(playQueue: PlayQueueService, nav: Nav, name: String, bvids: List<String>, page: (suspend (Int) -> Pair<List<String>, Boolean>)? = null) {
        loading?.cancel()
        loading = scope.launch {
            val first = try {
                playQueue.playTemp(name, bvids)
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                null
            } ?: return@launch
            ensureActive()
            nav.push(Screen.Player(first.bvid, first.title, fromPlaylist = true))
            if (page == null) return@launch
            var pn = 2
            while (pn <= MAX_PAGES) {
                val (more, hasMore) = runCatching { withContext(Dispatchers.IO) { page(pn) } }.getOrNull() ?: break
                playQueue.extendTemp(more)
                if (!hasMore || more.isEmpty()) break
                pn++
            }
        }
    }

    fun stop(queue: PlayQueueService) {
        loading?.cancel()
        loading = null
        queue.dropTemp()
    }
}

/** Drops the temporary list once no video page playing a list is left in the back stack. */
@Composable
fun dropTempWhenDone(playQueue: PlayQueueService, nav: Nav) {
    val playingList = nav.stack.any { it is Screen.Player && it.fromPlaylist }
    LaunchedEffect(playingList) { if (!playingList) TempPlay.stop(playQueue) }
}

/**
 * 播放队列: the list a video was played from, in place of its 合集 — folded to its name and what
 * comes next; unfolded, every video with the playing one marked.
 */
@Composable
fun QueueBlock(playQueue: PlayQueueService, currentBvid: String, onPick: (FfiVideoCard) -> Unit) {
    val c = LocalMetro.current
    var queue by remember { mutableStateOf<FfiPlayingList?>(null) }
    var open by remember { mutableStateOf(false) }
    // Read again when unfolded: a 收藏夹 keeps loading while it plays.
    LaunchedEffect(currentBvid, open) { queue = runCatching { withContext(Dispatchers.IO) { playQueue.playingList() } }.getOrNull() }
    val q = queue ?: return
    val all = q.items
    val at = all.indexOfFirst { it.bvid == currentBvid }
    val next = all.getOrNull(at + 1)
    Column {
        Box(Modifier.fillMaxWidth().height(1.dp).background(c.line))
        Row(
            Modifier.fillMaxWidth().clickable { open = !open }.padding(horizontal = 16.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            val name = q.name.ifEmpty { "播放列表" } + if (q.temporary) "（临时）" else ""
            Text("播放队列 · $name", fontSize = 17.sp, fontWeight = FontWeight.Light, color = c.fg, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f, fill = false))
            Text(if (at >= 0) " (第 ${at + 1} / ${all.size} 个)" else " (${all.size} 个)", fontSize = 12.sp, color = c.fg2, modifier = Modifier.weight(1f))
            Text(if (open) "收起 ▴" else "展开 ▾", fontSize = 14.sp, color = c.fg)
        }
        if (!open) {
            val hint = next?.let { "下一个：${it.title}" } ?: "已是最后一个"
            Text(hint + if (q.temporary) " · 退出播放页即清理" else "", fontSize = 12.sp, color = c.fg2, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.padding(start = 16.dp, end = 16.dp, bottom = 10.dp))
        } else {
            all.forEach { v ->
                val on = v.bvid == currentBvid
                Row(
                    Modifier.fillMaxWidth().clickable(enabled = !on) { onPick(v) }.padding(horizontal = 16.dp, vertical = 8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    Box(Modifier.width(4.dp).height(32.dp).background(if (on) c.accent else Color.Transparent))
                    AsyncImage(model = v.cover, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.width(64.dp).aspectRatio(16f / 9f).background(c.bg2))
                    Column(Modifier.weight(1f)) {
                        Text(v.title, fontSize = 14.sp, color = if (on) c.accent else c.fg, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        Text(listOf(v.upName, formatDuration(v.durationS)).filter { it.isNotBlank() }.joinToString(" · "), fontSize = 12.sp, color = c.fg2)
                    }
                }
            }
        }
    }
}

/** 订阅 / 已订阅 on a 合集; guests are sent to log in. */
@Composable
fun SeasonSubscribeButton(seasons: SeasonsService, nav: Nav, seasonId: ULong) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    var on by remember(seasonId) { mutableStateOf<Boolean?>(null) }
    var busy by remember { mutableStateOf(false) }
    LaunchedEffect(seasonId) { on = runCatching { seasons.seasonSubscribed(seasonId) }.getOrDefault(false) }
    val subscribed = on == true
    Text(
        if (subscribed) "已订阅" else "+ 订阅",
        fontSize = 13.sp,
        color = if (subscribed) c.fg else Color.White,
        modifier = Modifier.then(if (subscribed) Modifier.border(1.dp, c.line) else Modifier.background(c.accent))
            .clickable(enabled = on != null && !busy) {
                busy = true
                scope.launch {
                    runCatching { seasons.subscribeSeason(seasonId, !subscribed) }
                        .onSuccess { on = !subscribed }
                        .onFailure { if (it is RubiException.NotLoggedIn) nav.push(Screen.Login) }
                    busy = false
                }
            }
            .padding(horizontal = 10.dp, vertical = 4.dp),
    )
}

/** 订阅的合集, kept with the page in the back stack. */
class SeasonSubsState {
    var subs by mutableStateOf<List<FfiSeasonSub>?>(null)
    var error by mutableStateOf<Throwable?>(null)
    val list = LazyListState()
}

/**
 * 我的 › 订阅的合集: each with how many videos came since it was last opened; 编辑 ticks
 * several to unsubscribe together.
 */
@Composable
fun SeasonSubsScreen(seasons: SeasonsService, nav: Nav, state: SeasonSubsState) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    var editing by remember { mutableStateOf(false) }
    val picked = remember { mutableStateListOf<ULong>() }
    var note by remember { mutableStateOf("") }
    var busy by remember { mutableStateOf(false) }
    suspend fun load() {
        runCatching { seasons.seasonSubscriptions() }.onSuccess { state.subs = it; state.error = null }.onFailure { state.error = it }
    }
    // Read again on return: opening a season clears its new count.
    LaunchedEffect(Unit) { load() }

    Column(Modifier.fillMaxSize()) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f)) { PageHeader(nav, "订阅的合集") }
            Box(Modifier.padding(end = 12.dp)) {
                IconAction(if (editing) LineIcon.Clear else LineIcon.Edit, if (editing) "完成" else "编辑") { editing = !editing; picked.clear(); note = "" }
            }
        }
        val subs = state.subs
        val e = state.error
        when {
            e != null -> Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                Meta(errorText(e))
                if (e is RubiException.NotLoggedIn) AccentButton("登录") { nav.push(Screen.Login) }
                else TextAction("重试", onClick = { scope.launch { load() } })
            }
            subs == null -> Meta("加载中…", Modifier.padding(16.dp))
            subs.isEmpty() -> Meta("还没有订阅合集。在视频页的合集栏点「订阅」即可。", Modifier.padding(16.dp))
        }
        LazyColumn(Modifier.weight(1f), state = state.list, contentPadding = PaddingValues(bottom = 24.dp)) {
            items(subs.orEmpty(), key = { it.id.toString() }) { s ->
                val on = s.id in picked
                Row(
                    Modifier.fillMaxWidth().clickable {
                        if (editing) { if (on) picked.remove(s.id) else picked.add(s.id) }
                        else nav.push(Screen.SeasonVideos(s.upMid, s.id, s.title, SeasonVideosState(seasons, s.upMid, s.id)))
                    }.padding(horizontal = 16.dp, vertical = 8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    if (editing) Box(Modifier.size(18.dp).border(2.dp, c.fg).padding(3.dp).background(if (on) c.accent else Color.Transparent))
                    AsyncImage(model = s.cover, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.size(width = 136.dp, height = 76.dp).background(c.bg2))
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                        Text(s.title, fontSize = 15.sp, color = c.fg, maxLines = 2, overflow = TextOverflow.Ellipsis)
                        Meta(s.upName)
                        Row {
                            if (s.new > 0u) Text("${s.new} 个新视频 · ", fontSize = 12.sp, color = c.accent)
                            Meta("${s.count} 个视频")
                        }
                    }
                }
            }
        }
        if (note.isNotEmpty()) Meta(note, Modifier.padding(horizontal = 16.dp, vertical = 4.dp), color = c.accent)
        if (editing) {
            Box(Modifier.fillMaxWidth().height(1.dp).background(c.line))
            Row(Modifier.padding(horizontal = 16.dp, vertical = 10.dp), verticalAlignment = Alignment.CenterVertically) {
                Meta("已选 ${picked.size}", Modifier.weight(1f))
                val some = picked.isNotEmpty() && !busy
                IconAction(LineIcon.Delete, "取消订阅", if (some) LiveRed else c.fg2) {
                    if (!some) return@IconAction
                    busy = true
                    val ids = picked.toList()
                    scope.launch {
                        var failed: Throwable? = null
                        for (id in ids) runCatching { seasons.subscribeSeason(id, false) }.onFailure { failed = it }
                        note = failed?.let { "有的没取消成功：${errorText(it)}" } ?: "已取消订阅 ${ids.size} 个合集"
                        picked.clear()
                        load()
                        busy = false
                    }
                }
            }
        }
    }
}

/** A subscribed season's videos, kept with the page in the back stack. */
class SeasonVideosState(seasons: SeasonsService, upMid: ULong, seasonId: ULong) {
    val pager = Paged<FfiVideoCard, UInt>(key = { it.bvid }) { pn ->
        val at = pn ?: 1u
        val page = seasons.seasonVideos(upMid, seasonId, at)
        Page(page.cards, if (page.hasMore) at + 1u else null)
    }
    val list = LazyListState()
}

@Composable
fun SeasonVideosScreen(seasons: SeasonsService, nav: Nav, seasonId: ULong, title: String, state: SeasonVideosState) {
    val pager = state.pager
    LoadMoreNearEnd(state.list, pager)
    Column(Modifier.fillMaxSize()) {
        Row(verticalAlignment = Alignment.Bottom) {
            Column(Modifier.weight(1f)) { PageHeader(nav, title) }
            Box(Modifier.padding(end = 16.dp, bottom = 10.dp)) { SeasonSubscribeButton(seasons, nav, seasonId) }
        }
        val rowColumns = LocalScreenLayout.current.rowColumns()
        LazyColumn(Modifier.weight(1f), state = state.list, contentPadding = PaddingValues(bottom = 24.dp)) {
            gridItems(pager.items, rowColumns, key = { it.bvid }) { card ->
                VideoRow(card, "${formatCount(card.views)} 播放 · ${formatAge(card.pubdate)}") { nav.push(Screen.Player(card.bvid, card.title)) }
            }
            listFooter(nav, pager)
        }
    }
}
