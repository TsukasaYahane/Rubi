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
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyGridState
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.core.FfiEsportsGame
import dev.rubi.core.FfiLiveArea
import dev.rubi.core.FfiLiveRoom
import dev.rubi.core.FfiMatch
import dev.rubi.core.FfiMatchState
import dev.rubi.core.RubiCore
import dev.rubi.core.RubiException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.util.Calendar

/** The red of B站's 直播中 badge, kept so a live room reads as live at a glance. */
val LiveRed = Color(0xFFE5484D)

/**
 * The 直播 tab, hoisted above the screen so rooms and the schedule survive opening a room:
 * the 直播 pivot (area chips over a grid of rooms) and the 赛事 pivot (game chips over the
 * schedule).
 */
class LiveState(private val core: RubiCore) {
    var pivot by mutableStateOf(0)
    var areas by mutableStateOf(emptyList<FfiLiveArea>())
        private set
    /** 0 is 推荐; otherwise the area at `zone - 1`. */
    var zone by mutableStateOf(0)
        private set
    val grid = LazyGridState()
    val rooms = Paged<FfiLiveRoom, UInt>(key = { it.roomId }, minGapMs = 800L) { page ->
        val z = zone
        withContext(Dispatchers.IO) {
            if (z == 0) {
                Page(core.liveRecommended(), null)
            } else {
                val n = page ?: 1u
                val p = core.liveAreaRooms(areas[z - 1].id, n)
                Page(p.rooms, if (p.hasMore) n + 1u else null)
            }
        }
    }

    var games by mutableStateOf(emptyList<FfiEsportsGame>())
        private set
    /** 0 for every game. */
    var game by mutableStateOf(0u)
        private set
    val list = LazyListState()
    val matches = Paged<FfiMatch, UInt>(key = { it.id }, minGapMs = 800L) { page ->
        val g = game
        withContext(Dispatchers.IO) {
            val n = page ?: 1u
            val p = core.esportsSchedule(g, n)
            Page(p.matches, if (p.hasMore) n + 1u else null)
        }
    }

    suspend fun loadAreas() {
        if (areas.isEmpty()) areas = runCatching { withContext(Dispatchers.IO) { core.liveAreas() } }.getOrDefault(emptyList())
    }

    suspend fun loadGames() {
        if (games.isEmpty()) games = runCatching { withContext(Dispatchers.IO) { core.esportsGames() } }.getOrDefault(emptyList())
    }

    fun pickZone(z: Int) {
        if (z == zone && rooms.items.isNotEmpty()) return
        zone = z
        rooms.reset()
    }

    fun pickGame(g: UInt) {
        if (g == game && matches.items.isNotEmpty()) return
        game = g
        matches.reset()
    }

}

@Composable
fun LiveScreen(state: LiveState, nav: Nav, core: RubiCore) {
    // 查看全部赛程 from search lands here on 赛事.
    LaunchedEffect(nav.openSchedule) {
        if (nav.openSchedule) {
            state.pivot = 1
            nav.openSchedule = false
        }
    }
    LaunchedEffect(Unit) {
        nav.reselected.collect { if (it == Tab.Live) if (state.pivot == 0) state.grid.animateScrollToItem(0) else state.list.animateScrollToItem(0) }
    }
    Column(Modifier.fillMaxSize()) {
        val turns = LocalPageTurns.current
        Pivot(listOf("直播", "赛事"), state.pivot, { i -> turns.switch(i > state.pivot) { state.pivot = i } })
        Column(Modifier.weight(1f).pivotSlide(state.pivot)) {
            if (state.pivot == 0) LiveRooms(state, nav) else Schedule(state, nav, core)
        }
    }
}

@Composable
private fun LiveRooms(state: LiveState, nav: Nav) {
    val c = LocalMetro.current
    LaunchedEffect(Unit) {
        state.loadAreas()
        if (state.rooms.items.isEmpty() && !state.rooms.loading) state.rooms.more()
    }
    Chips(listOf("推荐") + state.areas.map { it.name }, state.zone, state::pickZone)
    LoadMoreNearEnd(state.grid, state.rooms)
    LazyVerticalGrid(
        columns = GridCells.Fixed(2),
        state = state.grid,
        contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 6.dp, bottom = 24.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
        modifier = Modifier.fillMaxSize(),
    ) {
        items(state.rooms.items, key = { it.roomId.toString() }) { r ->
            RoomCard(r) { nav.push(Screen.LiveRoom(r.roomId, r.title)) }
        }
        item(span = { GridItemSpan(2) }) {
            val rooms = state.rooms
            val e = rooms.error
            when {
                e != null -> Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
                    Meta(errorText(e), Modifier.weight(1f))
                    TextAction("重试", onClick = rooms::retry)
                }
                rooms.loading -> Meta("加载中…")
                rooms.items.isEmpty() -> Meta("这里还没有开播的直播间")
                state.zone == 0 -> TextAction("换一批", onClick = rooms::reset)
                rooms.ended -> Meta("没有更多了")
                else -> Box(Modifier.height(1.dp).background(c.line))
            }
        }
    }
}

@Composable
private fun RoomCard(r: FfiLiveRoom, onClick: () -> Unit) {
    val c = LocalMetro.current
    Column(Modifier.clickable(onClick = onClick)) {
        Box(Modifier.fillMaxWidth().aspectRatio(16f / 9f).background(c.bg2)) {
            AsyncImage(model = r.cover, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
            LiveBadge(Modifier.align(Alignment.TopStart).padding(4.dp))
            Text(
                formatCount(r.online),
                fontSize = 11.sp,
                color = Color.White,
                modifier = Modifier.align(Alignment.BottomEnd).padding(3.dp).background(Color.Black.copy(alpha = 0.55f)).padding(horizontal = 4.dp),
            )
        }
        Text(r.title, fontSize = 13.sp, lineHeight = 17.sp, color = c.fg, maxLines = 2, minLines = 2, overflow = TextOverflow.Ellipsis, modifier = Modifier.padding(top = 5.dp))
        Meta(r.uname.ifEmpty { r.area }, Modifier.padding(top = 1.dp))
    }
}

@Composable
fun LiveBadge(modifier: Modifier = Modifier, text: String = "直播中") {
    Text(text, fontSize = 11.sp, color = Color.White, modifier = modifier.background(LiveRed).padding(horizontal = 5.dp, vertical = 1.dp))
}

/** A row of square chips, the selected one filled with accent. */
@Composable
private fun Chips(labels: List<String>, selected: Int, onSelect: (Int) -> Unit) {
    val c = LocalMetro.current
    Row(
        Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(horizontal = 16.dp, vertical = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        labels.forEachIndexed { i, label ->
            val on = i == selected
            Text(
                label,
                fontSize = 13.sp,
                color = if (on) Color.White else c.fg,
                modifier = Modifier
                    .then(if (on) Modifier.background(c.accent) else Modifier.border(1.dp, c.line))
                    .clickable { onSelect(i) }
                    .padding(horizontal = 10.dp, vertical = 6.dp),
            )
        }
    }
}

@Composable
private fun Schedule(state: LiveState, nav: Nav, core: RubiCore) {
    val scope = rememberCoroutineScope()
    var problem by remember { mutableStateOf<String?>(null) }
    LaunchedEffect(Unit) {
        state.loadGames()
        if (state.matches.items.isEmpty() && !state.matches.loading) state.matches.more()
    }
    // Opened fresh: start at the first match not over yet, past yesterday's results.
    var placed by remember { mutableStateOf(state.matches.items.isNotEmpty()) }
    LaunchedEffect(state.matches.items.size) {
        if (placed || state.matches.items.isEmpty()) return@LaunchedEffect
        val at = state.matches.items.indexOfFirst { it.state != FfiMatchState.DONE }
        if (at > 0) state.list.scrollToItem(at)
        if (at >= 0 || state.matches.ended) placed = true
    }
    val ids = listOf(0u) + state.games.map { it.id }
    Chips(listOf("全部") + state.games.map { it.title }, ids.indexOf(state.game).coerceAtLeast(0)) {
        placed = false
        state.pickGame(ids[it])
    }
    Meta("赛程 · 昨天到 7 天后", Modifier.padding(start = 16.dp, end = 16.dp, bottom = 4.dp))
    problem?.let { Meta(it, Modifier.padding(horizontal = 16.dp), color = LiveRed) }
    LoadMoreNearEnd(state.list, state.matches)
    LazyColumn(Modifier.fillMaxSize(), state = state.list, contentPadding = PaddingValues(bottom = 24.dp)) {
        items(state.matches.items, key = { it.id.toString() }) { m ->
            MatchRow(m) { openMatch(m, nav, core, scope, changed = { state.matches.replace(it) }, say = { problem = it }) }
        }
        item {
            val e = state.matches.error
            Box(Modifier.padding(16.dp)) {
                when {
                    e != null -> TextAction("加载失败，重试", onClick = state.matches::retry)
                    state.matches.loading -> Meta("加载中…")
                    state.matches.items.isEmpty() -> Meta("这几天没有比赛")
                    state.matches.ended -> Meta("没有更多了")
                }
            }
        }
    }
}

@Composable
private fun MatchRow(m: FfiMatch, onClick: () -> Unit) {
    val c = LocalMetro.current
    Box(Modifier.fillMaxWidth().height(1.dp).background(c.line))
    MatchLine(m, Modifier.fillMaxWidth().clickable(onClick = onClick).padding(horizontal = 16.dp, vertical = 10.dp))
}

/** A match as a bordered card (above search results). */
@Composable
fun MatchCard(m: FfiMatch, onClick: () -> Unit) {
    val c = LocalMetro.current
    MatchLine(m, Modifier.fillMaxWidth().border(1.dp, c.line).clickable(onClick = onClick).padding(horizontal = 12.dp, vertical = 10.dp))
}

/**
 * What tapping a match does: on now opens its room, over opens the replay, to come toggles 预约.
 * `changed` gets the match after a 预约 change; `say` what went wrong.
 */
fun openMatch(m: FfiMatch, nav: Nav, core: RubiCore, scope: CoroutineScope, changed: (FfiMatch) -> Unit, say: (String?) -> Unit) {
    say(null)
    when (m.state) {
        FfiMatchState.LIVE -> if (m.liveRoom > 0uL) nav.push(Screen.LiveRoom(m.liveRoom, "${m.home.name} vs ${m.away.name}"))
        FfiMatchState.DONE ->
            if (m.replayBvid.isNotEmpty()) nav.push(Screen.Player(m.replayBvid, "${m.season} ${m.stage} ${m.home.name} vs ${m.away.name}"))
            else say("这场还没有回放")
        FfiMatchState.UPCOMING -> scope.launch {
            val on = !m.subscribed
            runCatching { withContext(Dispatchers.IO) { core.esportsSubscribe(m.id, on) } }
                .onSuccess { changed(m.copy(subscribed = on)) }
                .onFailure { say(if (it is RubiException.NotLoggedIn) "登录后才能预约" else "预约失败：${errorText(it)}") }
        }
    }
}

@Composable
private fun MatchLine(m: FfiMatch, modifier: Modifier) {
    val c = LocalMetro.current
    Row(
        modifier,
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Column(Modifier.width(78.dp)) {
            Meta(whenText(m.startAt))
            Text(m.stage, fontSize = 11.sp, color = c.fg2, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
        Row(Modifier.weight(1f), verticalAlignment = Alignment.CenterVertically) {
            Text(m.home.name, fontSize = 15.sp, color = c.fg, textAlign = TextAlign.End, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f))
            val score = if (m.state == FfiMatchState.UPCOMING) "VS" else "${m.homeScore} : ${m.awayScore}"
            Text(score, fontSize = 18.sp, fontWeight = FontWeight.SemiBold, color = c.fg, textAlign = TextAlign.Center, modifier = Modifier.width(58.dp))
            Text(m.away.name, fontSize = 15.sp, color = c.fg, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f))
        }
        Box(Modifier.width(64.dp), contentAlignment = Alignment.CenterEnd) {
            when (m.state) {
                FfiMatchState.LIVE -> LiveBadge()
                FfiMatchState.DONE -> Text(if (m.replayBvid.isNotEmpty()) "已结束 · 回放" else "已结束", fontSize = 12.sp, color = c.fg2, textAlign = TextAlign.End)
                FfiMatchState.UPCOMING -> Text(if (m.subscribed) "已预约" else "预约", fontSize = 12.sp, color = if (m.subscribed) c.fg2 else c.accent)
            }
        }
    }
}

/** 「今天 19:00」「明天 17:00」「11月4日 15:00」. */
private fun whenText(startAt: Long): String {
    val at = Calendar.getInstance().apply { timeInMillis = startAt * 1000 }
    val today = Calendar.getInstance()
    val days = dayNumber(at) - dayNumber(today)
    val time = "%02d:%02d".format(at.get(Calendar.HOUR_OF_DAY), at.get(Calendar.MINUTE))
    val day = when (days) {
        -1L -> "昨天"
        0L -> "今天"
        1L -> "明天"
        else -> "${at.get(Calendar.MONTH) + 1}月${at.get(Calendar.DAY_OF_MONTH)}日"
    }
    return "$day $time"
}

private fun dayNumber(c: Calendar): Long = (c.timeInMillis + c.get(Calendar.ZONE_OFFSET) + c.get(Calendar.DST_OFFSET)) / 86_400_000L
