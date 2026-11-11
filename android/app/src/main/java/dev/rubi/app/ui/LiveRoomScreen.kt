package dev.rubi.app.ui

import kotlinx.coroutines.CoroutineScope
import dev.rubi.core.SpaceService
import dev.rubi.core.LiveService
import dev.rubi.core.ActionsService
import dev.rubi.app.data.UiPrefs
import dev.rubi.app.data.CoreSettings
import dev.rubi.app.LocalFrozen
import kotlin.math.roundToInt
import dev.rubi.app.data.LiveDanmakuPrefs
import dev.rubi.app.data.PlaybackPrefs
import androidx.compose.ui.layout.boundsInWindow
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Slider
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.gestures.detectVerticalDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.activity.compose.BackHandler
import android.media.AudioManager
import android.os.SystemClock
import android.view.LayoutInflater
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameMillis
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.media3.ui.PlayerView
import dev.rubi.app.R
import dev.rubi.app.playback.LocalMedia
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.core.FfiLiveChannel
import coil3.compose.AsyncImage
import dev.rubi.core.FfiEmote
import dev.rubi.core.FfiLiveEvent
import dev.rubi.core.FfiLiveFilter
import dev.rubi.core.FfiLivePlay
import dev.rubi.core.FfiLiveRoom
import dev.rubi.core.RubiException
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/** At most this many danmaku lanes; how many fit depends on size and area. How long one takes to cross. */
private const val MAX_LANES = 16
private const val CROSS_MS = 7_000L
private val DM_SIZES = listOf(13, 15, 19)
private val DM_SIZE_NAMES = listOf("小", "标准", "大")
private val DM_AREAS = listOf(0.25f, 0.5f, 1f)
private val DM_AREA_NAMES = listOf("1/4 屏", "半屏", "全屏")
/** A lane takes its next danmaku only this long after the last one entered it. */
private const val LANE_GAP_MS = 700L
/** Lines kept in the chat list under the switches. */
private const val CHAT_LINES = 80
/** How long a gift banner or effect line stays up. */
private const val SHOW_MS = 3_000L
/** Reconnects to the message channel before giving up. */
private const val MAX_RECONNECTS = 5

private class Flying(
    val id: Long,
    val text: String,
    val color: Color,
    val lane: Int,
    val start: Long,
    val emotes: List<FfiEmote> = emptyList(),
    val sticker: String? = null,
)

private class ChatLine(
    val id: Long,
    val uname: String,
    val text: String,
    val emotes: List<FfiEmote> = emptyList(),
    val sticker: String? = null,
)

/**
 * A live room: the picture with its danmaku, the anchor and 关注, the room's three filter
 * switches with how much they held back, the chat, and a danmaku box. Gift banners, popups and
 * effects only appear when their switch is off.
 */
@Composable
fun LiveRoomScreen(space: SpaceService, actions: ActionsService, liveService: LiveService, uiPrefs: UiPrefs, settings: CoreSettings, appScope: CoroutineScope, nav: Nav, roomId: ULong, title: String) {
    val c = LocalMetro.current
    val media = LocalMedia.current
    val scope = rememberCoroutineScope()

    var room by remember { mutableStateOf<FfiLiveRoom?>(null) }
    var play by remember { mutableStateOf<FfiLivePlay?>(null) }
    var problem by remember { mutableStateOf<String?>(null) }
    var online by remember { mutableStateOf(0uL) }
    var onAir by remember { mutableStateOf(true) }
    var followed by remember { mutableStateOf<Boolean?>(null) }
    var filter by remember { mutableStateOf(uiPrefs.liveFilter) }
    var blocked by remember { mutableStateOf(Triple(0u, 0u, 0u)) }
    var channel by remember { mutableStateOf<FfiLiveChannel?>(null) }
    var pickingQuality by remember { mutableStateOf(false) }
    var fullscreen by remember { mutableStateOf(false) }
    var bars by remember { mutableStateOf(true) }
    var settingsOpen by remember { mutableStateOf(false) }
    var dm by remember { mutableStateOf(uiPrefs.liveDanmaku) }
    var laneCount by remember { mutableStateOf(8) }
    val gestures = rememberGestureRestore()
    // 小窗: the live picture goes on in picture-in-picture (by the button, or on leaving when
    // 切到后台自动小窗 is on), showing the picture alone.
    val ctx = LocalContext.current
    val activity = remember(ctx) { ctx.activity() }
    val inPip by media.pip.active
    val compact = fullscreen || inPip
    DisposableEffect(Unit) {
        media.pip.onPlayer = true
        media.pip.autoEnter = PlaybackPrefs.autoPip(settings)
        activity?.let(media.pip::sync)
        onDispose {
            media.pip.onPlayer = false
            media.pip.clearVideo()
            activity?.let(media.pip::sync)
        }
    }

    val flying = remember { mutableStateListOf<Flying>() }
    val laneLast = remember { LongArray(MAX_LANES) }
    val chat = remember { mutableStateListOf<ChatLine>() }
    var banner by remember { mutableStateOf<Pair<String, Long>?>(null) }
    var effect by remember { mutableStateOf<Pair<String, Long>?>(null) }
    var popup by remember { mutableStateOf<String?>(null) }
    var serial by remember { mutableLongStateOf(0L) }
    var now by remember { mutableLongStateOf(SystemClock.uptimeMillis()) }

    fun launchDanmaku(text: String, color: Color, emotes: List<FfiEmote> = emptyList(), sticker: String? = null) {
        val t = SystemClock.uptimeMillis()
        flying.removeAll { t - it.start > CROSS_MS }
        if (!dm.enabled) return
        // The first lane free for a while; when every lane is busy the danmaku is skipped.
        val lane = (0 until laneCount.coerceIn(1, MAX_LANES)).firstOrNull { t - laneLast[it] >= LANE_GAP_MS } ?: return
        laneLast[lane] = t
        flying.add(Flying(serial++, text, color, lane, t, emotes, sticker))
    }

    fun show(ev: FfiLiveEvent) {
        val t = SystemClock.uptimeMillis()
        when (ev) {
            is FfiLiveEvent.Danmaku -> {
                launchDanmaku(ev.text, Color((0xFF000000 or ev.color.toLong()).toInt()), ev.emotes, ev.sticker)
                chat.add(ChatLine(serial++, ev.uname, ev.text, ev.emotes, ev.sticker))
                if (chat.size > CHAT_LINES) chat.removeRange(0, chat.size - CHAT_LINES)
            }
            is FfiLiveEvent.Gift -> banner = ev.text to t
            is FfiLiveEvent.Effect -> effect = ev.text to t
            is FfiLiveEvent.Popup -> popup = ev.text
            is FfiLiveEvent.Online -> online = ev.count
            is FfiLiveEvent.Status -> onAir = ev.live
        }
    }

    suspend fun startPlaying(qn: UInt?) {
        val r = room ?: return
        runCatching { withContext(Dispatchers.IO) { liveService.livePlay(r.roomId, qn) } }
            .onSuccess { p ->
                play = p
                onAir = p.live
                if (p.live && p.streams.isNotEmpty()) media.player.playLive(p, r.title, r.uname) else media.player.clear()
            }
            .onFailure { problem = "无法播放：${errorText(it)}" }
    }

    // The room header first (short ids resolve to the real room), then the picture.
    LaunchedEffect(roomId) {
        media.listening.stop()
        runCatching { withContext(Dispatchers.IO) { liveService.liveRoom(roomId) } }
            .onSuccess { r ->
                room = r
                online = r.online
                onAir = r.live
                startPlaying(null)
                if (r.uid > 0uL) followed = runCatching { withContext(Dispatchers.IO) { space.upProfile(r.uid).followed } }.getOrNull()
            }
            .onFailure { problem = errorText(it) }
    }
    DisposableEffect(Unit) { onDispose { media.player.clear() } }

    // The message channel, rejoined when it drops.
    val realId = room?.roomId
    // Frozen (节电): the channel is left, and joined again on waking.
    val frozen = LocalFrozen.current
    LaunchedEffect(realId, frozen) {
        val id = realId ?: return@LaunchedEffect
        if (frozen) return@LaunchedEffect
        var tries = 0
        while (tries <= MAX_RECONNECTS) {
            try {
                val ch = withContext(Dispatchers.IO) { liveService.liveChannel(id, filter) }
                channel = ch
                tries = 0
                while (true) {
                    val batch = ch.next() ?: break
                    blocked = Triple(batch.blockedGifts, batch.blockedPopups, batch.blockedEffects)
                    batch.events.forEach(::show)
                }
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                tries++
            } finally {
                // The page scope is already cancelled when the page goes: leave on the app's,
                // so the socket is closed and the Rust object freed now, not at some GC.
                channel?.let { ch -> appScope.launch { runCatching { ch.leave() }; ch.close() } }
                channel = null
            }
            delay(3_000L)
        }
    }
    LaunchedEffect(frozen) {
        if (!frozen) while (true) withFrameMillis { now = SystemClock.uptimeMillis() }
    }

    fun setDm(next: LiveDanmakuPrefs) {
        dm = next
        uiPrefs.liveDanmaku = next
        if (!next.enabled) flying.clear()
    }

    fun send(text: String, done: (String?) -> Unit) {
        val r = room ?: return
        scope.launch {
            runCatching { withContext(Dispatchers.IO) { liveService.liveSend(r.roomId, text) } }
                .onSuccess { done(null) }
                .onFailure { done(if (it is RubiException.NotLoggedIn) "登录后才能发弹幕" else "发送失败：${errorText(it)}") }
        }
    }

    FullscreenMode(fullscreen)
    BackHandler(enabled = fullscreen) { if (settingsOpen) settingsOpen = false else fullscreen = false }

    fun setFilter(f: FfiLiveFilter) {
        filter = f
        uiPrefs.liveFilter = f
        channel?.setFilter(f)
        if (f.popups) popup = null
    }

    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            // Header: back, title, viewers.
            if (!compact) Row(Modifier.fillMaxWidth().height(48.dp).padding(start = 4.dp, end = 12.dp), verticalAlignment = Alignment.CenterVertically) {
                Box(Modifier.size(44.dp).clickable { nav.pop() }, contentAlignment = Alignment.Center) { GlyphIcon(Glyph.Back) }
                Text(room?.title ?: title, fontSize = 15.sp, color = c.fg, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f))
                if (online > 0uL) Meta("${formatCount(online)}人", Modifier.padding(start = 8.dp))
            }
            // The picture, its danmaku, and what the switches let through.
            // The same slot in both modes, so the PlayerView survives the switch.
            BoxWithConstraints(
                (if (compact) Modifier.fillMaxSize() else Modifier.fillMaxWidth().aspectRatio(16f / 9f))
                    // Where the picture is, so 小窗 shrinks from it.
                    .onGloballyPositioned { coords ->
                        val a = activity
                        if (a != null && !inPip) {
                            val r = coords.boundsInWindow()
                            media.pip.videoAt(a, r.left.toInt(), r.top.toInt(), r.right.toInt(), r.bottom.toInt())
                        }
                    }
                    .background(Color.Black),
            ) {
                AndroidView(
                    factory = {
                        (LayoutInflater.from(it).inflate(R.layout.player_view, null) as PlayerView).apply {
                            player = media.player.get()
                        }
                    },
                    onRelease = { it.player = null },
                    modifier = Modifier.fillMaxSize(),
                )
                val density = LocalDensity.current
                val widthPx = with(density) { maxWidth.toPx() }
                val fontSp = DM_SIZES[dm.size.coerceIn(0, 2)]
                val laneH = with(density) { (fontSp * 1.6f).sp.toPx() }
                // As many lanes as fit in the area danmaku may cover.
                val fits = ((with(density) { maxHeight.toPx() } * DM_AREAS[dm.area.coerceIn(0, 2)]) / laneH).toInt().coerceIn(1, MAX_LANES)
                LaunchedEffect(fits) { laneCount = fits }
                val style = TextStyle(fontSize = fontSp.sp, color = Color.White, shadow = Shadow(Color.Black, blurRadius = 4f))
                // No danmaku in the small window: too little room to read them.
                if (dm.enabled && !inPip) flying.forEach { f ->
                    if (f.lane >= fits) return@forEach
                    val at = Modifier.offset {
                        val x = widthPx - (now - f.start) * (widthPx + 600f) / CROSS_MS
                        IntOffset(x.toInt(), (f.lane * laneH + laneH * 0.2f).toInt())
                    }
                    val sticker = f.sticker
                    if (sticker != null) {
                        // A big emote flies as its image, about two lines tall.
                        AsyncImage(
                            model = sticker,
                            contentDescription = f.text,
                            alpha = dm.opacity,
                            modifier = at.size((fontSp * 2.2f).dp),
                        )
                    } else {
                        EmoteText(
                            f.text,
                            color = f.color.copy(alpha = dm.opacity),
                            fontSize = fontSp.sp,
                            maxLines = 1,
                            extra = f.emotes,
                            style = style.copy(color = f.color.copy(alpha = dm.opacity)),
                            softWrap = false,
                            overflow = TextOverflow.Clip,
                            modifier = at,
                        )
                    }
                }
                if (onAir && !compact) LiveBadge(Modifier.align(Alignment.TopStart).padding(6.dp))
                if (!onAir) Text("主播还没有开播", fontSize = 14.sp, color = Color.White, modifier = Modifier.align(Alignment.Center))
                banner?.takeIf { now - it.second < SHOW_MS && !inPip }?.let {
                    Text(
                        "🎁 ${it.first}",
                        fontSize = 12.sp,
                        color = Color.White,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.align(Alignment.CenterStart).background(Color(0xFFC9A227)).padding(horizontal = 10.dp, vertical = 4.dp),
                    )
                }
                effect?.takeIf { now - it.second < SHOW_MS && !inPip }?.let {
                    Text(
                        it.first,
                        fontSize = 12.sp,
                        color = Color.White,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.align(Alignment.BottomStart).fillMaxWidth().background(c.accent.copy(alpha = 0.8f)).padding(horizontal = 10.dp, vertical = 3.dp),
                    )
                }
                if (inPip) {
                    // The small window shows the picture alone.
                } else if (!fullscreen) {
                    if (activity != null && media.pip.supported(activity)) {
                        Row(
                            Modifier.align(Alignment.TopEnd).clickable { media.pip.enter(activity) }.padding(horizontal = 10.dp, vertical = 8.dp),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(4.dp),
                        ) {
                            if (!LocalWordsUi.current) LineIconView(LineIcon.Pip, Color.White, 18.dp)
                            Text("小窗", fontSize = 12.sp, color = Color.White)
                        }
                    }
                    Text(
                        "全屏",
                        fontSize = 12.sp,
                        color = Color.White,
                        modifier = Modifier.align(Alignment.BottomEnd).clickable { fullscreen = true; bars = true }.padding(horizontal = 10.dp, vertical = 8.dp),
                    )
                } else {
                    LiveGestures(gestures, onTap = { if (settingsOpen) settingsOpen = false else bars = !bars })
                    if (bars) {
                        LiveTopBar(
                            title = room?.title ?: title,
                            online = online,
                            onAir = onAir,
                            onBack = { fullscreen = false },
                            onSettings = { settingsOpen = true },
                            modifier = Modifier.align(Alignment.TopStart),
                        )
                        LiveBottomBar(
                            dm = dm,
                            quality = play?.let { p -> p.qualities.firstOrNull { it.qn == p.qn }?.name },
                            onToggleDm = { setDm(dm.copy(enabled = !dm.enabled)) },
                            onQuality = { fullscreen = false; pickingQuality = true },
                            onExit = { fullscreen = false },
                            onSend = ::send,
                            modifier = Modifier.align(Alignment.BottomStart),
                        )
                    }
                    if (settingsOpen) {
                        LiveSettingsPanel(
                            dm = dm,
                            onDm = ::setDm,
                            filter = filter,
                            onFilter = ::setFilter,
                            blocked = blocked,
                            onClose = { settingsOpen = false },
                            modifier = Modifier.align(Alignment.CenterEnd),
                        )
                    }
                }
            }
            if (!compact) {
            problem?.let { Meta(it, Modifier.padding(horizontal = 16.dp, vertical = 4.dp), color = LiveRed) }
            // Anchor, quality, 关注.
            Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                Text(
                    room?.uname.orEmpty(),
                    fontSize = 15.sp,
                    color = c.fg,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.weight(1f).clickable { room?.let { nav.push(Screen.Up(it.uid, it.uname)) } },
                )
                val p = play
                val q = p?.qualities?.firstOrNull { it.qn == p.qn }
                if (q != null && p.qualities.size > 1) {
                    Text(q.name + " ▾", fontSize = 13.sp, color = c.fg2, modifier = Modifier.clickable { pickingQuality = !pickingQuality }.padding(horizontal = 10.dp, vertical = 6.dp))
                }
                val f = followed
                val r = room
                if (f != null && r != null) {
                    Text(
                        if (f) "已关注" else "关注",
                        fontSize = 13.sp,
                        color = if (f) c.fg2 else c.fg,
                        modifier = Modifier.border(1.dp, if (f) c.line else c.fg).clickable {
                            scope.launch {
                                runCatching { withContext(Dispatchers.IO) { actions.followMid(r.uid, r.uname, !f) } }
                                    .onSuccess { followed = !f }
                                    .onFailure { problem = if (it is RubiException.NotLoggedIn) "登录后才能关注" else "关注失败：${errorText(it)}" }
                            }
                        }.padding(horizontal = 10.dp, vertical = 4.dp),
                    )
                }
            }
            val p = play
            if (pickingQuality && p != null) {
                Row(
                    Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(horizontal = 16.dp),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    p.qualities.forEach { q ->
                        val on = q.qn == p.qn
                        Text(
                            q.name,
                            fontSize = 13.sp,
                            color = if (on) Color.White else c.fg,
                            modifier = Modifier.then(if (on) Modifier.background(c.accent) else Modifier.border(1.dp, c.line))
                                .clickable {
                                    pickingQuality = false
                                    if (!on) scope.launch { startPlaying(q.qn) }
                                }
                                .padding(horizontal = 10.dp, vertical = 6.dp),
                        )
                    }
                }
            }
            // The room's filter switches.
            Text("直播间过滤", fontSize = 13.sp, color = c.fg, modifier = Modifier.padding(start = 16.dp, top = 6.dp, bottom = 2.dp))
            FilterRow("礼物横幅", filter.gifts) { setFilter(filter.copy(gifts = !filter.gifts)) }
            FilterRow("抽奖 / 红包 / 购物弹窗", filter.popups) { setFilter(filter.copy(popups = !filter.popups)) }
            FilterRow("高能弹幕与进场特效", filter.effects) { setFilter(filter.copy(effects = !filter.effects)) }
            val (g, pp, e) = blocked
            Meta("本场已拦下：礼物横幅 $g · 弹窗 $pp · 特效 $e", Modifier.padding(horizontal = 16.dp, vertical = 4.dp))
            // The chat, newest at the bottom.
            val chatList = rememberLazyListState()
            LaunchedEffect(chat.size) { if (chat.isNotEmpty()) chatList.scrollToItem(chat.lastIndex) }
            LazyColumn(Modifier.weight(1f).fillMaxWidth(), state = chatList, contentPadding = PaddingValues(horizontal = 16.dp, vertical = 4.dp)) {
                items(chat, key = { it.id }) { line ->
                    Row(Modifier.padding(vertical = 2.dp)) {
                        Text(line.uname + "：", fontSize = 13.sp, color = c.fg2, maxLines = 1)
                        val sticker = line.sticker
                        if (sticker != null) {
                            AsyncImage(model = sticker, contentDescription = line.text, modifier = Modifier.size(48.dp))
                        } else {
                            EmoteText(line.text, color = c.fg, fontSize = 13.sp, extra = line.emotes)
                        }
                    }
                }
            }
            DanmakuBox(onSend = ::send)
            }
        }
        // A popup only when its switch is off; it can only be closed, never joined from here.
        popup?.let { text ->
            Column(
                Modifier.align(Alignment.Center).padding(horizontal = 30.dp).fillMaxWidth().background(c.bg2).border(1.dp, c.line).padding(16.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                Text(text, fontSize = 15.sp, color = c.fg)
                TextAction("关闭", onClick = { popup = null })
            }
        }
    }
}

@Composable
private fun FilterRow(label: String, on: Boolean, onToggle: () -> Unit) {
    val c = LocalMetro.current
    Row(
        Modifier.fillMaxWidth().clickable(onClick = onToggle).padding(horizontal = 16.dp, vertical = 7.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(label, fontSize = 13.sp, color = c.fg, modifier = Modifier.weight(1f))
        SquareSwitch(on)
    }
}

/** The danmaku box: a field and 发送. `done` gets null on success or what went wrong. */
@Composable
private fun DanmakuBox(onSend: (String, (String?) -> Unit) -> Unit) {
    val c = LocalMetro.current
    var text by remember { mutableStateOf("") }
    var sending by remember { mutableStateOf(false) }
    var note by remember { mutableStateOf<String?>(null) }
    fun send() {
        val t = text.trim()
        if (t.isEmpty() || sending) return
        sending = true
        onSend(t) { err ->
            sending = false
            note = err
            if (err == null) text = ""
        }
    }
    Box(Modifier.fillMaxWidth().height(1.dp).background(c.line))
    note?.let { Meta(it, Modifier.padding(start = 12.dp, top = 4.dp), color = LiveRed) }
    Row(Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 8.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
        Box(Modifier.weight(1f).heightIn(min = 40.dp).border(1.dp, c.line).padding(horizontal = 10.dp, vertical = 10.dp)) {
            if (text.isEmpty()) Text("发个弹幕…", fontSize = 14.sp, color = c.fg2)
            BasicTextField(
                value = text,
                onValueChange = { text = it.take(40) },
                singleLine = true,
                textStyle = TextStyle(color = c.fg, fontSize = 14.sp),
                cursorBrush = SolidColor(c.accent),
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
                keyboardActions = KeyboardActions(onSend = { send() }),
                modifier = Modifier.fillMaxWidth(),
            )
        }
        Box(
            Modifier.height(40.dp).background(if (text.isBlank() || sending) c.fg2 else c.accent).clickable { send() }.padding(horizontal = 14.dp),
            contentAlignment = Alignment.Center,
        ) { Text(if (sending) "发送中" else "发送", fontSize = 13.sp, color = Color.White) }
    }
}

/** Fullscreen touches: a tap shows or hides the bars; dragging up or down sets brightness on the left half, volume on the right. */
@Composable
private fun LiveGestures(restore: GestureRestore, onTap: () -> Unit) {
    val ctx = LocalContext.current
    val window = remember(ctx) { ctx.activity()?.window }
    val audio = remember(ctx) { ctx.getSystemService(AudioManager::class.java) }
    var badge by remember { mutableStateOf("") }
    BoxWithConstraints(Modifier.fillMaxSize()) {
        val w = constraints.maxWidth.toFloat().coerceAtLeast(1f)
        val h = constraints.maxHeight.toFloat().coerceAtLeast(1f)
        Box(
            Modifier.fillMaxSize()
                .pointerInput(Unit) { detectTapGestures(onTap = { onTap() }) }
                .pointerInput(Unit) {
                    var left = true
                    var value = 0f
                    detectVerticalDragGestures(
                        onDragStart = { at ->
                            left = at.x < w / 2
                            value = if (left) currentBrightness(ctx, window)
                            else audio.getStreamVolume(AudioManager.STREAM_MUSIC).toFloat() / audio.getStreamMaxVolume(AudioManager.STREAM_MUSIC).coerceAtLeast(1)
                        },
                        onDragEnd = { badge = "" },
                        onDragCancel = { badge = "" },
                    ) { _, dy ->
                        if (left) {
                            value = (value - dy / h).coerceIn(0.01f, 1f)
                            window?.let { it.attributes = it.attributes.apply { screenBrightness = value } }
                            badge = "亮度 ${(value * 100).toInt()}%"
                        } else {
                            value = (value - dy / h).coerceIn(0f, 1f)
                            val max = audio.getStreamMaxVolume(AudioManager.STREAM_MUSIC)
                            if (restore.volumeBefore == null) restore.volumeBefore = audio.getStreamVolume(AudioManager.STREAM_MUSIC)
                            audio.setStreamVolume(AudioManager.STREAM_MUSIC, (value * max).toInt(), 0)
                            badge = "音量 ${(value * 100).toInt()}%"
                        }
                    }
                },
        )
        if (badge.isNotEmpty()) {
            Text(badge, fontSize = 15.sp, color = Color.White, modifier = Modifier.align(Alignment.Center).background(Color.Black.copy(alpha = 0.6f)).padding(horizontal = 14.dp, vertical = 8.dp))
        }
    }
}

private val BarShade = Color.Black.copy(alpha = 0.55f)

@Composable
private fun LiveTopBar(title: String, online: ULong, onAir: Boolean, onBack: () -> Unit, onSettings: () -> Unit, modifier: Modifier) {
    Row(
        modifier.fillMaxWidth().height(44.dp).background(BarShade).padding(horizontal = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Text("←", fontSize = 18.sp, color = Color.White, modifier = Modifier.clickable(onClick = onBack).padding(horizontal = 8.dp, vertical = 6.dp))
        Text(title, fontSize = 14.sp, color = Color.White, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f))
        if (online > 0uL) Text("${formatCount(online)}人", fontSize = 12.sp, color = Color.White.copy(alpha = 0.75f))
        if (onAir) LiveBadge()
        Text("弹幕设置", fontSize = 13.sp, color = Color.White, modifier = Modifier.clickable(onClick = onSettings).padding(horizontal = 6.dp, vertical = 6.dp))
    }
}

/** 弹 toggle, a danmaku box as tall as it, 发送, quality and 退出全屏. */
@Composable
private fun LiveBottomBar(
    dm: LiveDanmakuPrefs,
    quality: String?,
    onToggleDm: () -> Unit,
    onQuality: () -> Unit,
    onExit: () -> Unit,
    onSend: (String, (String?) -> Unit) -> Unit,
    modifier: Modifier,
) {
    val c = LocalMetro.current
    var text by remember { mutableStateOf("") }
    var sending by remember { mutableStateOf(false) }
    var note by remember { mutableStateOf<String?>(null) }
    fun send() {
        val t = text.trim()
        if (t.isEmpty() || sending) return
        sending = true
        onSend(t) { err ->
            sending = false
            note = err
            if (err == null) text = ""
        }
    }
    Column(modifier.fillMaxWidth().background(BarShade)) {
        note?.let { Text(it, fontSize = 12.sp, color = LiveRed, modifier = Modifier.padding(start = 12.dp, top = 4.dp)) }
        Row(Modifier.fillMaxWidth().height(40.dp).padding(horizontal = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            Box(Modifier.height(BAR_FIELD).border(1.dp, Color.White).clickable(onClick = onToggleDm).padding(horizontal = 8.dp), contentAlignment = Alignment.Center) {
                Text(if (dm.enabled) "弹 开" else "弹 关", fontSize = 12.sp, color = Color.White)
            }
            Box(Modifier.weight(1f).height(BAR_FIELD).background(Color.White.copy(alpha = 0.12f)).border(1.dp, Color.White.copy(alpha = 0.5f)).padding(horizontal = 8.dp), contentAlignment = Alignment.CenterStart) {
                if (text.isEmpty()) Text("发个弹幕…", fontSize = 12.sp, color = Color.White.copy(alpha = 0.6f))
                BasicTextField(
                    value = text,
                    onValueChange = { text = it.take(40); note = null },
                    singleLine = true,
                    textStyle = TextStyle(color = Color.White, fontSize = 12.sp),
                    cursorBrush = SolidColor(Color.White),
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
                    keyboardActions = KeyboardActions(onSend = { send() }),
                    modifier = Modifier.fillMaxWidth(),
                )
            }
            Box(Modifier.height(BAR_FIELD).background(if (text.isBlank() || sending) c.fg2 else c.accent).clickable { send() }.padding(horizontal = 12.dp), contentAlignment = Alignment.Center) {
                Text(if (sending) "发送中" else "发送", fontSize = 12.sp, color = Color.White)
            }
            if (quality != null) Text("$quality ▾", fontSize = 12.sp, color = Color.White.copy(alpha = 0.8f), modifier = Modifier.clickable(onClick = onQuality))
            Text("退出全屏", fontSize = 12.sp, color = Color.White, modifier = Modifier.clickable(onClick = onExit))
        }
    }
}

/** The height of the bottom bar's toggle, field and button. */
private val BAR_FIELD = 26.dp

/** Danmaku look and the room's filter switches, slid in from the right. */
@Composable
private fun LiveSettingsPanel(
    dm: LiveDanmakuPrefs,
    onDm: (LiveDanmakuPrefs) -> Unit,
    filter: FfiLiveFilter,
    onFilter: (FfiLiveFilter) -> Unit,
    blocked: Triple<UInt, UInt, UInt>,
    onClose: () -> Unit,
    modifier: Modifier,
) {
    Column(
        modifier.fillMaxHeight().width(280.dp).background(Color.Black.copy(alpha = 0.9f))
            .pointerInput(Unit) { detectTapGestures { } }
            .verticalScroll(rememberScrollState()).padding(horizontal = 16.dp, vertical = 12.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text("弹幕设置", fontSize = 15.sp, color = Color.White, modifier = Modifier.weight(1f))
            Text("✕", fontSize = 15.sp, color = Color.White, modifier = Modifier.clickable(onClick = onClose).padding(6.dp))
        }
        PanelSwitch("显示弹幕", dm.enabled) { onDm(dm.copy(enabled = !dm.enabled)) }
        Text("不透明度 ${(dm.opacity * 100).toInt()}%", fontSize = 13.sp, color = Color.White)
        Slider(
            value = dm.opacity,
            onValueChange = { onDm(dm.copy(opacity = (it * 20).roundToInt() / 20f)) },
            valueRange = 0.2f..1f,
            colors = SliderDefaults.colors(thumbColor = Color.White, activeTrackColor = LocalMetro.current.accent),
        )
        Text("字号", fontSize = 13.sp, color = Color.White)
        Segments(DM_SIZE_NAMES, dm.size) { onDm(dm.copy(size = it)) }
        Text("显示区域", fontSize = 13.sp, color = Color.White)
        Segments(DM_AREA_NAMES, dm.area) { onDm(dm.copy(area = it)) }
        Box(Modifier.fillMaxWidth().height(1.dp).background(Color.White.copy(alpha = 0.2f)))
        Text("直播间过滤", fontSize = 13.sp, color = Color.White)
        PanelSwitch("礼物横幅", filter.gifts) { onFilter(filter.copy(gifts = !filter.gifts)) }
        PanelSwitch("抽奖 / 红包 / 购物弹窗", filter.popups) { onFilter(filter.copy(popups = !filter.popups)) }
        PanelSwitch("高能弹幕与进场特效", filter.effects) { onFilter(filter.copy(effects = !filter.effects)) }
        val (g, p, e) = blocked
        Text("本场已拦下：礼物横幅 $g · 弹窗 $p · 特效 $e", fontSize = 12.sp, color = Color.White.copy(alpha = 0.7f))
    }
}

@Composable
private fun PanelSwitch(label: String, on: Boolean, onToggle: () -> Unit) {
    Row(Modifier.fillMaxWidth().clickable(onClick = onToggle).padding(vertical = 2.dp), verticalAlignment = Alignment.CenterVertically) {
        Text(label, fontSize = 13.sp, color = Color.White, modifier = Modifier.weight(1f))
        SquareSwitch(on)
    }
}

@Composable
private fun Segments(labels: List<String>, selected: Int, onSelect: (Int) -> Unit) {
    val c = LocalMetro.current
    Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        labels.forEachIndexed { i, label ->
            val on = i == selected
            Box(
                Modifier.weight(1f).height(28.dp)
                    .then(if (on) Modifier.background(c.accent) else Modifier.border(1.dp, Color.White.copy(alpha = 0.4f)))
                    .clickable { onSelect(i) },
                contentAlignment = Alignment.Center,
            ) { Text(label, fontSize = 12.sp, color = Color.White) }
        }
    }
}
