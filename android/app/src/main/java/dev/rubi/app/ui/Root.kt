package dev.rubi.app.ui

import androidx.compose.runtime.snapshotFlow
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.rememberScrollState
import kotlinx.coroutines.flow.MutableSharedFlow
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.rubi.app.RubiApp
import dev.rubi.app.playback.AudioQueue
import dev.rubi.app.data.UiPrefs
import dev.rubi.core.FfiCommentRef
import dev.rubi.core.FfiVideoRef
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType

sealed interface Screen {
    /** `focus`: a comment to open the 评论 tab on (from a notice). */
    data class Player(
        val bvid: String,
        val title: String,
        val focus: FfiCommentRef? = null,
        /** Opened from the play list: when it ends, the list's next item follows. */
        val fromPlaylist: Boolean = false,
        /** Where to start, from 听视频's 看视频. */
        val startMs: Long = 0L,
    ) : Screen
    data object Listen : Screen
    /** Each entry carries its own state, so results survive opening a video from them. */
    class Search(val state: SearchState) : Screen
    data object Inbox : Screen
    data class Chat(val mid: ULong, val name: String) : Screen
    data class Up(val mid: ULong, val name: String) : Screen
    data object Login : Screen
    data object Filtered : Screen
    data object Settings : Screen
    data object Playlist : Screen
    data object History : Screen
    data object WatchLater : Screen
    data object FavFolders : Screen
    data class FavFolder(val id: ULong, val title: String) : Screen
}

enum class Tab(val label: String) { Home("首页"), Follow("关注"), Me("我的") }

/** Video pages kept to go back through in one chain. */
private const val MAX_VIDEO_DEPTH = 20

/** Navigation: three tabs plus a stack of full-screen pages on top. */
class Nav {
    var tab by mutableStateOf(Tab.Home)
    val stack = mutableStateListOf<Screen>()
    /** The message center's tab, kept while a video opened from it is on top. */
    var inboxTab by mutableStateOf(0)
    /** A tap on the tab that is already selected: that tab scrolls back to the top. */
    val reselected = MutableSharedFlow<Tab>(extraBufferCapacity = 1)

    fun select(t: Tab) {
        if (tab == t) reselected.tryEmit(t) else tab = t
    }

    /**
     * Opens a page. A long chain of videos keeps only its latest [MAX_VIDEO_DEPTH] pages to go
     * back through; older ones are dropped so the chain does not grow without end.
     */
    fun push(s: Screen) {
        stack.add(s)
        if (s is Screen.Player && stack.count { it is Screen.Player } > MAX_VIDEO_DEPTH) {
            stack.removeAt(stack.indexOfFirst { it is Screen.Player })
        }
    }

    /** Swaps the top page (the play list moving on to its next video). */
    fun replaceTop(s: Screen) {
        if (stack.isEmpty()) stack.add(s) else stack[stack.lastIndex] = s
    }
    fun pop() = stack.removeLastOrNull()

    /** Leaves every page on top (a whole chain of videos) for the 首页 tab. */
    fun home() {
        stack.clear()
        tab = Tab.Home
    }

    /** Whether any video page is open; when this turns false the user has left the video chain. */
    val inVideos: Boolean get() = stack.any { it is Screen.Player }

    /** Opens a video from a notice or message; av numbers are resolved to a BV id first. */
    fun openVideo(
        scope: CoroutineScope,
        app: RubiApp,
        ref: FfiVideoRef,
        title: String,
        focus: FfiCommentRef? = null,
        onError: (String) -> Unit = {},
    ) {
        scope.launch {
            runCatching { app.core.resolveVideo(ref) }
                .onSuccess { push(Screen.Player(it, title, focus)) }
                .onFailure { onError("无法打开视频：${it.message}") }
        }
    }
}

@Composable
fun RubiRoot(app: RubiApp) {
    val c = LocalMetro.current
    val nav = remember { Nav() }
    val home = remember { HomeState(app.ui) }
    val rootScope = rememberCoroutineScope()
    val follow = remember { FollowState(app.core, rootScope) }
    var unread by remember { mutableStateOf(0u) }
    var emotes by remember { mutableStateOf(emptyMap<String, String>()) }

    LaunchedEffect(Unit) {
        runCatching { app.core.ensureDeviceIds(); app.saveSession() }
        emotes = runCatching { app.core.emotes().associate { it.text to it.url } }.getOrDefault(emptyMap())
        unread = runCatching { app.core.unread().let { it.reply + it.at + it.like + it.chat } }.getOrDefault(0u)
    }
    BackHandler(enabled = nav.stack.isNotEmpty()) { nav.pop() }
    // Leaving the last video page frees what the chain of videos piled up.
    LaunchedEffect(nav) {
        var watching = false
        snapshotFlow { nav.inVideos }.collect { now ->
            if (watching && !now) app.releaseVideoCaches()
            watching = now
        }
    }
    val listenRequested by AudioQueue.pageRequested
    LaunchedEffect(listenRequested) {
        if (!listenRequested) return@LaunchedEffect
        AudioQueue.pageRequested.value = false
        if (AudioQueue.current.value != null && nav.stack.lastOrNull() != Screen.Listen) nav.push(Screen.Listen)
    }

    CompositionLocalProvider(
        LocalEmotes provides emotes,
        LocalAddToPlaylist provides { card -> addToPlaylist(app, rootScope, card) },
    ) {
    val edgeToEdge by fullscreenActive
    Box(if (edgeToEdge) Modifier.fillMaxSize().background(Color.Black) else Modifier.fillMaxSize().background(c.bg).safeDrawingPadding()) {
        when (val top = nav.stack.lastOrNull()) {
            null -> Column(Modifier.fillMaxSize()) {
                TopBar(nav, unread) { nav.push(Screen.Search(SearchState(app.core))) }
                // The three tabs side by side: swipe between them, or tap the bottom bar.
                val pager = rememberPagerState(initialPage = nav.tab.ordinal) { Tab.entries.size }
                LaunchedEffect(nav.tab) { if (pager.targetPage != nav.tab.ordinal) pager.animateScrollToPage(nav.tab.ordinal) }
                LaunchedEffect(pager) { snapshotFlow { pager.settledPage }.collect { nav.tab = Tab.entries[it] } }
                HorizontalPager(pager, Modifier.weight(1f), key = { Tab.entries[it].name }) { page ->
                    when (Tab.entries[page]) {
                        Tab.Home -> HomeScreen(app, nav, home)
                        Tab.Follow -> FollowScreen(app, nav, follow, home.dual)
                        Tab.Me -> MeScreen(app, nav)
                    }
                }
                BottomNav(nav)
            }
            // A fresh composition per video: no danmaku, comments or tab state carried over.
            is Screen.Player -> key(top) { PlayerScreen(app, nav, top.bvid, top.title, top.focus, top.fromPlaylist, top.startMs) }
            Screen.Listen -> ListenScreen(app, nav)
            is Screen.Search -> SearchScreen(app, nav, top.state)
            Screen.Inbox -> InboxScreen(app, nav)
            is Screen.Chat -> ChatScreen(app, nav, top.mid, top.name)
            is Screen.Up -> key(top) { UpScreen(app, nav, top.mid, top.name) }
            Screen.Login -> LoginScreen(app, nav)
            Screen.Filtered -> FilteredScreen(app, nav)
            Screen.Settings -> SettingsScreen(app, nav)
            Screen.Playlist -> PlaylistScreen(app, nav)
            Screen.History -> HistoryScreen(app, nav)
            Screen.WatchLater -> WatchLaterScreen(app, nav)
            Screen.FavFolders -> FavFoldersScreen(app, nav)
            is Screen.FavFolder -> key(top) { FavFolderScreen(app, nav, top.id, top.title) }
        }
    }
    }
}

@Composable
private fun TopBar(nav: Nav, unread: UInt, onSearch: () -> Unit) {
    val c = LocalMetro.current
    Row(
        Modifier.fillMaxWidth().height(56.dp).padding(horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        Text("RUBI", style = MetroType.brand, color = c.fg)
        Row(
            Modifier.weight(1f).height(44.dp).clickable(onClick = onSearch)
                .border(width = 0.dp, color = Color.Transparent),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            GlyphIcon(Glyph.Search, size = 18.dp)
            Text("搜索视频、UP 主", color = c.fg2, fontSize = 14.sp)
        }
        Box(Modifier.size(44.dp).clickable { nav.push(Screen.Inbox) }, contentAlignment = Alignment.Center) {
            GlyphIcon(Glyph.Mail)
            if (unread > 0u) {
                Text(
                    unread.toString(),
                    style = MetroType.meta.copy(fontSize = 10.sp),
                    color = Color.White,
                    modifier = Modifier.align(Alignment.TopEnd).padding(top = 6.dp).background(c.accent).padding(horizontal = 3.dp),
                )
            }
        }
    }
}

@Composable
private fun BottomNav(nav: Nav) {
    val c = LocalMetro.current
    Box(Modifier.fillMaxWidth().height(1.dp).background(c.line))
    Row(Modifier.fillMaxWidth().height(56.dp)) {
        Tab.entries.forEach { t ->
            val on = nav.tab == t
            Box(Modifier.weight(1f).fillMaxSize().clickable { nav.select(t) }, contentAlignment = Alignment.Center) {
                Text(t.label, fontSize = 15.sp, color = if (on) c.fg else c.fg2, fontWeight = if (on) FontWeight.SemiBold else FontWeight.Normal)
            }
        }
    }
}

/** Header for full-screen pages: back arrow, brand, large title. */
@Composable
fun PageHeader(nav: Nav, title: String?) {
    val c = LocalMetro.current
    Row(Modifier.fillMaxWidth().height(56.dp).padding(start = 4.dp), verticalAlignment = Alignment.CenterVertically) {
        Box(Modifier.size(44.dp).clickable { nav.pop() }, contentAlignment = Alignment.Center) { GlyphIcon(Glyph.Back) }
        Text("RUBI", style = MetroType.brand, color = c.fg)
    }
    if (title != null) Text(title, style = MetroType.pivot, color = c.fg, modifier = Modifier.padding(horizontal = 16.dp))
}

