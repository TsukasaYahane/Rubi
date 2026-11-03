package dev.rubi.app.ui

import dev.rubi.core.FfiUserHit
import androidx.compose.foundation.border
import kotlinx.coroutines.withContext
import kotlinx.coroutines.Job
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.CoroutineScope
import dev.rubi.core.FfiMatch
import dev.rubi.core.RubiCore
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.foundation.lazy.LazyListState
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
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.rubi.app.RubiApp
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiChatMessage
import dev.rubi.core.FfiChatSession
import dev.rubi.core.FfiNotice
import dev.rubi.core.FfiNoticeKind
import dev.rubi.core.FfiSharedVideo
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.style.TextOverflow
import coil3.compose.AsyncImage
import dev.rubi.core.FfiVideoCard
import dev.rubi.core.RubiException
import kotlinx.coroutines.launch

/** Underlined single-line input (Metro text box). */
@Composable
internal fun MetroField(value: String, onChange: (String) -> Unit, hint: String, modifier: Modifier = Modifier, onSubmit: () -> Unit) {
    val c = LocalMetro.current
    Box(modifier.height(44.dp), contentAlignment = Alignment.CenterStart) {
        if (value.isEmpty()) Text(hint, color = c.fg2, fontSize = 16.sp)
        BasicTextField(
            value = value,
            onValueChange = onChange,
            singleLine = true,
            textStyle = TextStyle(color = c.fg, fontSize = 16.sp),
            cursorBrush = SolidColor(c.accent),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
            keyboardActions = KeyboardActions(onAny = { onSubmit() }),
            modifier = Modifier.fillMaxWidth(),
        )
        Box(Modifier.align(Alignment.BottomStart).fillMaxWidth().height(2.dp).background(c.line))
    }
}

/**
 * One search page's state, kept by its [Screen.Search] entry: opening a result disposes the
 * page, and coming back must show the same results at the same place.
 */
class SearchState(private val core: RubiCore) {
    var query by mutableStateOf("")
    var hot by mutableStateOf(emptyList<String>())
    var hotHidden by mutableStateOf(0u)
    /** The term the results are for; empty while 热搜 shows. */
    var searched by mutableStateOf("")
        private set
    val list = LazyListState()
    /** 综合, 视频 or UP 主. */
    var tab by mutableStateOf(0)
    /** UPs whose name or alias fits; 综合 shows the first few. */
    val users = Paged<FfiUserHit, UInt>(key = { it.mid }, minGapMs = 800L) { pn ->
        val at = pn ?: 1u
        val r = withContext(Dispatchers.IO) { core.searchUsers(searched, at) }
        Page(r.users, if (r.hasMore) at + 1u else null)
    }
    /** Matches the term is about (a team, an event), shown above the videos. */
    var matches by mutableStateOf(emptyList<FfiMatch>())
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
    private var matchJob: Job? = null

    /** Result pages; one that comes back empty (all filtered) still moves on, up to B站's 50. */
    val results = Paged<FfiVideoCard, UInt>(key = { it.bvid }) { pn ->
        val at = pn ?: 1u
        val r = core.search(searched, at)
        val more = (r.cards.isNotEmpty() || r.hidden > 0u) && at < MAX_SEARCH_PAGES
        Page(r.cards, if (more) at + 1u else null, r.hidden)
    }

    fun search(term: String) {
        query = term
        searched = term
        results.reset()
        users.reset()
        matches = emptyList()
        matchJob?.cancel()
        matchJob = scope.launch {
            matches = runCatching { withContext(Dispatchers.IO) { core.esportsRelated(term) } }.getOrDefault(emptyList())
        }
    }
}

private const val MAX_SEARCH_PAGES = 50u

private val SEARCH_TABS = listOf("综合", "视频", "UP 主")

/** A found UP: face, name, fans and uploads, 直播中 when on air; tap for their space. */
@Composable
private fun UserRow(u: FfiUserHit, nav: Nav) {
    val c = LocalMetro.current
    Row(
        Modifier.fillMaxWidth().clickable { nav.push(Screen.Up(u.mid, u.name)) }.padding(horizontal = 16.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        AsyncImage(model = u.face, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.size(44.dp).background(c.bg2))
        Column(Modifier.weight(1f)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                Text(u.name, fontSize = 15.sp, color = c.fg, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f, fill = false))
                if (u.liveRoom > 0uL) LiveBadge(Modifier.clickable { nav.push(Screen.LiveRoom(u.liveRoom, u.name)) })
            }
            Meta("${formatCount(u.fans)} 粉丝 · ${u.videos} 投稿")
            if (u.sign.isNotBlank()) Text(u.sign, fontSize = 12.sp, color = c.fg2, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
    }
}

@Composable
fun SearchScreen(app: RubiApp, nav: Nav, state: SearchState) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    val keyboard = LocalSoftwareKeyboardController.current
    val focus = LocalFocusManager.current
    val results = state.results
    val list = state.list
    var matchNote by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(Unit) {
        if (state.hot.isEmpty()) runCatching { app.core.hotWords() }.onSuccess { state.hot = it.words; state.hotHidden = it.hidden }
    }
    fun search(q: String) {
        // The search is under way: put the keyboard away and leave the field.
        keyboard?.hide()
        focus.clearFocus()
        state.search(q)
        scope.launch { list.scrollToItem(0) }
    }
    if (state.searched.isNotEmpty()) {
        if (state.tab == 2) LoadMoreNearEnd(list, state.users, ahead = 3) else LoadMoreNearEnd(list, results, ahead = 3)
    }
    val note = when {
        state.searched.isEmpty() -> if (state.hotHidden > 0u) "已过滤推广热搜词 ${state.hotHidden} 条" else ""
        state.tab == 2 -> state.users.error?.let(::errorText) ?: ""
        results.error != null -> errorText(results.error!!)
        else -> "结果 ${results.items.size} · 已过滤 ${results.hidden}"
    }

    Column(Modifier.fillMaxSize()) {
        Row(Modifier.fillMaxWidth().padding(end = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(44.dp).clickable { nav.pop() }, contentAlignment = Alignment.Center) { GlyphIcon(Glyph.Back) }
            MetroField(state.query, { state.query = it }, "搜索视频、UP 主", Modifier.weight(1f)) { if (state.query.isNotBlank()) search(state.query) }
        }
        if (state.searched.isNotEmpty()) {
            Row(Modifier.padding(start = 16.dp, end = 16.dp, top = 4.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                SEARCH_TABS.forEachIndexed { i, label ->
                    val on = i == state.tab
                    Text(
                        label,
                        fontSize = 13.sp,
                        color = if (on) Color.White else c.fg,
                        modifier = Modifier.then(if (on) Modifier.background(c.accent) else Modifier.border(1.dp, c.line))
                            .clickable { state.tab = i; scope.launch { list.scrollToItem(0) } }
                            .padding(horizontal = 10.dp, vertical = 5.dp),
                    )
                }
            }
        }
        if (note.isNotEmpty()) Meta(note, Modifier.padding(horizontal = 16.dp, vertical = 8.dp))
        LazyColumn(Modifier.weight(1f), state = list) {
            if (state.searched.isEmpty()) {
                item { SectionTitle("热搜") }
                items(state.hot.withIndex().toList()) { (i, w) ->
                    Row(
                        Modifier.fillMaxWidth().height(48.dp).clickable { search(w) }.padding(horizontal = 16.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text("%02d".format(i + 1), style = MetroType.meta.copy(fontSize = 16.sp), color = if (i < 3) c.fg else c.fg2, modifier = Modifier.widthIn(min = 32.dp))
                        Text(w, fontSize = 16.sp, color = c.fg)
                    }
                }
            } else if (state.tab == 2) {
                items(state.users.items, key = { "u" + it.mid }) { u -> UserRow(u, nav) }
                item {
                    Column(Modifier.padding(16.dp)) {
                        val users = state.users
                        when {
                            users.loading -> Meta("加载中…")
                            users.error != null -> TextAction("重试", onClick = users::retry)
                            users.ended && users.items.isNotEmpty() -> Meta("没有更多了")
                            users.ended -> Meta("没有找到相关 UP 主")
                        }
                    }
                }
            } else {
                // 综合: matches and the best-fitting UPs above the videos.
                if (state.tab == 0 && state.matches.isNotEmpty()) {
                    item(key = "esports") {
                        Column(Modifier.padding(horizontal = 16.dp, vertical = 6.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                            Meta("赛事 · ${state.matches.first().season}")
                            state.matches.forEach { m ->
                                MatchCard(m) {
                                    openMatch(m, nav, app.core, scope, changed = { n -> state.matches = state.matches.map { if (it.id == n.id) n else it } }, say = { matchNote = it })
                                }
                            }
                            matchNote?.let { Meta(it, color = LiveRed) }
                            TextAction("查看全部赛程 ›", onClick = { nav.showSchedule() }, dim = true)
                        }
                    }
                }
                if (state.tab == 0 && state.users.items.isNotEmpty()) {
                    item(key = "users") {
                        Column(Modifier.padding(top = 4.dp)) {
                            Meta("UP 主", Modifier.padding(horizontal = 16.dp))
                            state.users.items.take(3).forEach { u -> UserRow(u, nav) }
                            Box(Modifier.padding(horizontal = 16.dp)) {
                                TextAction("查看全部 UP 主 ›", onClick = { state.tab = 2; scope.launch { list.scrollToItem(0) } }, dim = true)
                            }
                        }
                    }
                }
                items(results.items, key = { it.bvid }) { card ->
                    Box(Modifier.padding(horizontal = 16.dp, vertical = 12.dp)) {
                        VideoCardView(card, big = true, onOpen = { nav.push(Screen.Player(card.bvid, card.title)) })
                    }
                }
                item {
                    Column(Modifier.padding(16.dp)) {
                        when {
                            results.loading -> Meta("加载中…")
                            results.error != null -> TextAction("重试", onClick = results::retry)
                            results.ended && results.items.isNotEmpty() -> Meta("没有更多了")
                            results.ended -> Meta("没有找到相关视频")
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun InboxScreen(app: RubiApp, nav: Nav) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    // Kept in Nav: opening a video disposes this page, and coming back must land on the same tab.
    var tab by nav::inboxTab
    var notices by remember { mutableStateOf(emptyList<FfiNotice>()) }
    var sessions by remember { mutableStateOf(emptyList<FfiChatSession>()) }
    var note by remember { mutableStateOf("") }

    LaunchedEffect(tab) {
        note = ""
        runCatching {
            if (tab < 3) {
                val kind = listOf(FfiNoticeKind.REPLY, FfiNoticeKind.AT, FfiNoticeKind.LIKE)[tab]
                notices = app.core.notices(kind, null).items
                note = "仅显示真人互动"
            } else {
                val s = app.core.chatSessions()
                sessions = s.sessions
                note = "已过滤营销私信 ${s.hidden} 个（未关注账号的推广卡片）"
            }
        }.onFailure { note = if (it is RubiException.NotLoggedIn) "登录后可查看消息" else "加载失败：${it.message}" }
    }

    Column(Modifier.fillMaxSize()) {
        PageHeader(nav, "消息")
        val turns = LocalPageTurns.current
        Pivot(listOf("回复", "@我", "赞", "私信"), tab, { i -> if (i != tab) turns.switch(i > tab) { tab = i } }, MetroType.section)
        Meta(note, Modifier.padding(horizontal = 16.dp, vertical = 4.dp))
        LazyColumn(Modifier.weight(1f).pivotSlide(tab)) {
            if (tab < 3) {
                items(notices, key = { it.id.toString() }) { n ->
                    val video = n.video
                    val open = Modifier.let { m ->
                        if (video == null) m else m.clickable { nav.openVideo(scope, app, video, n.context, focus = n.comment, onError = { note = it }) }
                    }
                    Column(open.fillMaxWidth().padding(horizontal = 16.dp, vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        Text(n.from, fontSize = 15.sp, fontWeight = FontWeight.SemiBold, color = c.fg)
                        EmoteText(n.text, color = c.fg, fontSize = 15.sp, lineHeight = 22.sp)
                        EmoteText(n.context, color = c.fg2, fontSize = 12.sp)
                        Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                            Meta(formatAge(n.at))
                            if (video != null) Meta("▷ 打开视频", color = c.fg)
                        }
                    }
                }
            } else {
                items(sessions, key = { it.talkerMid.toString() }) { s ->
                    Row(
                        Modifier.fillMaxWidth().clickable { nav.push(Screen.Chat(s.talkerMid, s.name)) }.padding(16.dp, 12.dp),
                        verticalAlignment = Alignment.Top,
                    ) {
                        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                            Text(s.name, fontSize = 20.sp, fontWeight = if (s.unread > 0u) FontWeight.SemiBold else FontWeight.Light, color = c.fg)
                            EmoteText((if (s.lastFromMe) "我：" else "") + s.lastText, color = c.fg2, fontSize = 14.sp, maxLines = 1)
                        }
                        Column(horizontalAlignment = Alignment.End, verticalArrangement = Arrangement.spacedBy(6.dp)) {
                            Meta(formatAge(s.lastAt))
                            if (s.unread > 0u) Tag(s.unread.toString())
                        }
                    }
                }
            }
        }
    }
}

/** A video shared in chat: cover, title and author, opening the player on tap. */
@Composable
private fun SharedVideoCard(video: FfiSharedVideo, onOpen: () -> Unit) {
    val c = LocalMetro.current
    Row(
        Modifier.widthIn(max = 300.dp).background(c.bg2).clickable(onClick = onOpen).padding(10.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        AsyncImage(
            model = video.cover,
            contentDescription = null,
            contentScale = ContentScale.Crop,
            modifier = Modifier.size(width = 112.dp, height = 63.dp).background(c.line),
        )
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(video.title, fontSize = 14.sp, lineHeight = 20.sp, color = c.fg, maxLines = 2, overflow = TextOverflow.Ellipsis)
            Meta(if (video.author.isEmpty()) "▷ 视频" else "▷ ${video.author}")
        }
    }
}

@Composable
fun ChatScreen(app: RubiApp, nav: Nav, mid: ULong, name: String) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    var messages by remember { mutableStateOf(emptyList<FfiChatMessage>()) }
    var draft by remember { mutableStateOf("") }
    var note by remember { mutableStateOf("") }
    val list = rememberLazyListState()
    // Messages run oldest to newest: open (and stay after sending) at the newest one.
    LaunchedEffect(messages.size) { if (messages.isNotEmpty()) list.scrollToItem(messages.lastIndex) }

    suspend fun reload() {
        runCatching { app.core.chatMessages(mid) }
            .onSuccess { msgs ->
                messages = msgs
                msgs.lastOrNull()?.let { runCatching { app.core.markRead(mid, it.seqno) } }
            }
            .onFailure { note = "加载失败：${it.message}" }
    }
    LaunchedEffect(mid) { reload() }

    Column(Modifier.fillMaxSize()) {
        Row(Modifier.fillMaxWidth().height(56.dp).padding(start = 4.dp, end = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(44.dp).clickable { nav.pop() }, contentAlignment = Alignment.Center) { GlyphIcon(Glyph.Back) }
            Text(name, fontSize = 18.sp, fontWeight = FontWeight.SemiBold, color = c.fg, modifier = Modifier.weight(1f))
            Meta("私信")
        }
        LazyColumn(Modifier.weight(1f).padding(horizontal = 16.dp), state = list, verticalArrangement = Arrangement.spacedBy(10.dp)) {
            items(messages, key = { it.seqno.toString() }) { m ->
                val shared = m.video
                Column(Modifier.fillMaxWidth(), horizontalAlignment = if (m.fromMe) Alignment.End else Alignment.Start) {
                    if (shared != null && shared.title.isNotEmpty()) {
                        SharedVideoCard(shared) { nav.openVideo(scope, app, shared.video, shared.title, onError = { note = it }) }
                    } else {
                        EmoteText(
                            m.text,
                            fontSize = 15.sp,
                            lineHeight = 22.sp,
                            color = if (m.fromMe) Color.White else c.fg,
                            modifier = Modifier.widthIn(max = 300.dp).background(if (m.fromMe) c.accent else c.bg2).padding(12.dp, 10.dp),
                        )
                        // A BV / av id mentioned in plain text.
                        if (shared != null) {
                            TextAction("▷ 打开视频", onClick = { nav.openVideo(scope, app, shared.video, "", onError = { note = it }) })
                        }
                    }
                }
            }
        }
        if (note.isNotEmpty()) Meta(note, Modifier.padding(horizontal = 16.dp))
        Box(Modifier.fillMaxWidth().height(1.dp).background(c.line))
        Row(Modifier.padding(16.dp, 8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            val send = {
                val text = draft.trim()
                if (text.isNotEmpty()) scope.launch {
                    runCatching { app.core.sendMessage(mid, text) }
                        .onSuccess { draft = ""; reload() }
                        .onFailure { note = "发送失败：${it.message}" }
                }
                Unit
            }
            MetroField(draft, { draft = it }, "发消息", Modifier.weight(1f), onSubmit = send)
            AccentButton("发送", send)
        }
    }
}
