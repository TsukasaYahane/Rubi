package dev.rubi.app.ui

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
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.rubi.app.RubiApp
import dev.rubi.core.FfiVideoRef
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType

sealed interface Screen {
    data class Player(val bvid: String, val title: String) : Screen
    data object Search : Screen
    data object Inbox : Screen
    data class Chat(val mid: ULong, val name: String) : Screen
    data class Up(val mid: ULong, val name: String) : Screen
    data object Login : Screen
    data object Filtered : Screen
    data object Settings : Screen
    data object History : Screen
    data object WatchLater : Screen
    data object FavFolders : Screen
    data class FavFolder(val id: ULong, val title: String) : Screen
}

enum class Tab(val label: String) { Home("首页"), Follow("关注"), Me("我的") }

/** Navigation: three tabs plus a stack of full-screen pages on top. */
class Nav {
    var tab by mutableStateOf(Tab.Home)
    val stack = mutableStateListOf<Screen>()
    /** A tap on the tab that is already selected: that tab scrolls back to the top. */
    val reselected = MutableSharedFlow<Tab>(extraBufferCapacity = 1)

    fun select(t: Tab) {
        if (tab == t) reselected.tryEmit(t) else tab = t
    }

    fun push(s: Screen) = stack.add(s)
    fun pop() = stack.removeLastOrNull()

    /** Opens a video from a notice or message; av numbers are resolved to a BV id first. */
    fun openVideo(scope: CoroutineScope, app: RubiApp, ref: FfiVideoRef, title: String, onError: (String) -> Unit = {}) {
        scope.launch {
            runCatching { app.core.resolveVideo(ref) }
                .onSuccess { push(Screen.Player(it, title)) }
                .onFailure { onError("无法打开视频：${it.message}") }
        }
    }
}

@Composable
fun RubiRoot(app: RubiApp) {
    val c = LocalMetro.current
    val nav = remember { Nav() }
    val home = remember { HomeState() }
    var unread by remember { mutableStateOf(0u) }
    var emotes by remember { mutableStateOf(emptyMap<String, String>()) }

    LaunchedEffect(Unit) {
        runCatching { app.core.ensureDeviceIds(); app.saveSession() }
        emotes = runCatching { app.core.emotes().associate { it.text to it.url } }.getOrDefault(emptyMap())
        unread = runCatching { app.core.unread().let { it.reply + it.at + it.like + it.chat } }.getOrDefault(0u)
    }
    BackHandler(enabled = nav.stack.isNotEmpty()) { nav.pop() }

    CompositionLocalProvider(LocalEmotes provides emotes) {
    Box(Modifier.fillMaxSize().background(c.bg).safeDrawingPadding()) {
        when (val top = nav.stack.lastOrNull()) {
            null -> Column(Modifier.fillMaxSize()) {
                TopBar(nav, unread)
                Box(Modifier.weight(1f)) {
                    when (nav.tab) {
                        Tab.Home -> HomeScreen(app, nav, home)
                        Tab.Follow -> FollowScreen(app, nav)
                        Tab.Me -> MeScreen(app, nav)
                    }
                }
                BottomNav(nav)
            }
            // A fresh composition per video: no danmaku, comments or tab state carried over.
            is Screen.Player -> key(top) { PlayerScreen(app, nav, top.bvid, top.title) }
            Screen.Search -> SearchScreen(app, nav)
            Screen.Inbox -> InboxScreen(app, nav)
            is Screen.Chat -> ChatScreen(app, nav, top.mid, top.name)
            is Screen.Up -> key(top) { UpScreen(app, nav, top.mid, top.name) }
            Screen.Login -> LoginScreen(app, nav)
            Screen.Filtered -> FilteredScreen(app, nav)
            Screen.Settings -> SettingsScreen(app, nav)
            Screen.History -> HistoryScreen(app, nav)
            Screen.WatchLater -> WatchLaterScreen(app, nav)
            Screen.FavFolders -> FavFoldersScreen(app, nav)
            is Screen.FavFolder -> key(top) { FavFolderScreen(app, nav, top.id, top.title) }
        }
    }
    }
}

@Composable
private fun TopBar(nav: Nav, unread: UInt) {
    val c = LocalMetro.current
    Row(
        Modifier.fillMaxWidth().height(56.dp).padding(horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        Text("RUBI", style = MetroType.brand, color = c.fg)
        Row(
            Modifier.weight(1f).height(44.dp).clickable { nav.push(Screen.Search) }
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

@Composable
fun FollowScreen(app: RubiApp, nav: Nav) {
    val c = LocalMetro.current
    var note by remember { mutableStateOf("动态是第二阶段功能。这里可以先把关注列表同步给推荐算法。") }
    val scroll = rememberScrollState()
    LaunchedEffect(Unit) { nav.reselected.collect { if (it == Tab.Follow) scroll.animateScrollTo(0) } }
    Column(Modifier.fillMaxSize().verticalScroll(scroll).padding(16.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
        Text("关注", style = MetroType.pivot, color = c.fg)
        Text(note, color = c.fg2, fontSize = 14.sp)
        var busy by remember { mutableStateOf(false) }
        if (busy) LaunchedEffect(Unit) {
            note = runCatching { "已同步 ${app.core.syncFollowings()} 个关注" }.getOrElse { "同步失败：${it.message}" }
            busy = false
        }
        AccentButton("同步关注列表") { busy = true }
    }
}
