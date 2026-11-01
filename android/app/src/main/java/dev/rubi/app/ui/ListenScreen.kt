package dev.rubi.app.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import dev.rubi.app.RubiApp
import dev.rubi.app.playback.AudioQueue
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.core.FfiPlaylistMode
import dev.rubi.core.FfiVideoCard
import kotlinx.coroutines.delay

private val MODE_ORDER = listOf(FfiPlaylistMode.SEQUENTIAL, FfiPlaylistMode.LOOP_ALL, FfiPlaylistMode.LOOP_ONE, FfiPlaylistMode.SHUFFLE)

private fun modeLabel(m: FfiPlaylistMode) = when (m) {
    FfiPlaylistMode.SEQUENTIAL -> "顺序播放"
    FfiPlaylistMode.LOOP_ALL -> "列表循环"
    FfiPlaylistMode.LOOP_ONE -> "单曲循环"
    FfiPlaylistMode.SHUFFLE -> "随机播放"
}

/**
 * 听视频: the play list heard as audio, laid out like a music player. Opened from 正在听 on the
 * play list page or from the playback notification. Every shape is square-cornered.
 */
@Composable
fun ListenScreen(app: RubiApp, nav: Nav) {
    val c = LocalMetro.current
    val now by AudioQueue.current
    val playing by AudioQueue.playing
    var items by remember { mutableStateOf(emptyList<FfiVideoCard>()) }
    var mode by remember { mutableStateOf(FfiPlaylistMode.SEQUENTIAL) }
    var open by remember { mutableStateOf(false) }
    var position by remember { mutableStateOf(0L) }
    var length by remember { mutableStateOf(0L) }
    // While the bar is held: the fraction under the finger, applied on release.
    var dragging by remember { mutableStateOf<Float?>(null) }

    LaunchedEffect(Unit) {
        items = runCatching { app.core.playlistItems() }.getOrDefault(emptyList())
        mode = runCatching { app.core.playlistMode() }.getOrDefault(FfiPlaylistMode.SEQUENTIAL)
    }
    LaunchedEffect(Unit) {
        while (true) {
            val (p, l) = AudioQueue.progress()
            position = p
            length = l
            delay(500)
        }
    }
    // Nothing left to hear (stopped or the list ran out): leave the page.
    LaunchedEffect(now) { if (now == null) nav.pop() }
    val card = now ?: return

    val at = items.indexOfFirst { it.bvid == card.bvid }
    val shown = dragging?.let { (it * length).toLong() } ?: position
    val fraction = if (length > 0) (shown.toFloat() / length).coerceIn(0f, 1f) else 0f

    LazyColumn(Modifier.fillMaxSize()) {
        item {
            Box(Modifier.fillMaxWidth().height(52.dp), contentAlignment = Alignment.Center) {
                Box(Modifier.align(Alignment.CenterStart).padding(start = 8.dp).size(44.dp).clickable { nav.pop() }, contentAlignment = Alignment.Center) {
                    Icon(20.dp, c.fg) { back(it) }
                }
                Text("正在听 · 播放列表", fontSize = 13.sp, letterSpacing = 1.sp, color = c.fg2)
            }
        }
        item {
            Box(Modifier.padding(horizontal = 16.dp).fillMaxWidth().aspectRatio(16f / 9f).background(c.bg2)) {
                AsyncImage(model = card.cover, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
                Text(
                    formatDuration(card.durationS),
                    fontSize = 12.sp,
                    color = Color.White,
                    modifier = Modifier.align(Alignment.BottomStart).padding(10.dp).background(Color(0x8C000000)).padding(horizontal = 6.dp, vertical = 2.dp),
                )
            }
        }
        item {
            Column(Modifier.padding(start = 16.dp, end = 16.dp, top = 16.dp)) {
                Text(card.title, fontSize = 19.sp, lineHeight = 26.sp, color = c.fg, maxLines = 2, minLines = 2, overflow = TextOverflow.Ellipsis)
                Text(
                    "${card.upName}${if (at >= 0) " · 第 ${at + 1} / ${items.size} 首" else ""}",
                    fontSize = 14.sp,
                    color = c.fg2,
                    modifier = Modifier.padding(top = 4.dp),
                )
            }
        }
        item {
            Column(Modifier.padding(start = 16.dp, end = 16.dp, top = 12.dp)) {
                Canvas(
                    Modifier.fillMaxWidth().height(28.dp)
                        .pointerInput(length) {
                            detectTapGestures { o -> if (length > 0) AudioQueue.seekTo((o.x / size.width * length).toLong()) }
                        }
                        .pointerInput(length) {
                            detectHorizontalDragGestures(
                                onDragStart = { o -> dragging = (o.x / size.width).coerceIn(0f, 1f) },
                                onDragEnd = { dragging?.let { if (length > 0) AudioQueue.seekTo((it * length).toLong()) }; dragging = null },
                                onDragCancel = { dragging = null },
                            ) { change, _ -> dragging = (change.position.x / size.width).coerceIn(0f, 1f) }
                        },
                ) {
                    val y = size.height / 2
                    val track = 2.dp.toPx()
                    drawLine(c.line, Offset(0f, y), Offset(size.width, y), track)
                    drawLine(c.accent, Offset(0f, y), Offset(size.width * fraction, y), track)
                    val kw = 6.dp.toPx()
                    val kh = 14.dp.toPx()
                    drawRect(c.fg, Offset(size.width * fraction - kw / 2, y - kh / 2), androidx.compose.ui.geometry.Size(kw, kh))
                }
                Row {
                    Text(clockOf(shown), fontSize = 12.sp, color = c.fg2, modifier = Modifier.weight(1f))
                    Text(clockOf(length), fontSize = 12.sp, color = c.fg2)
                }
            }
        }
        item {
            Row(
                Modifier.fillMaxWidth().padding(start = 20.dp, end = 20.dp, top = 22.dp, bottom = 10.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                IconButton(48.dp, onClick = {
                    val m = MODE_ORDER[(MODE_ORDER.indexOf(mode) + 1) % MODE_ORDER.size]
                    if (runCatching { app.core.setPlaylistMode(m) }.isSuccess) mode = m
                }) { Icon(24.dp, c.fg) { modeIcon(it, mode) } }
                IconButton(52.dp, onClick = { AudioQueue.previous() }) { Icon(26.dp, c.fg) { skip(it, forward = false) } }
                IconButton(72.dp, background = c.accent, onClick = { AudioQueue.toggle() }) {
                    Icon(28.dp, Color.White) { if (playing) pause(it) else play(it) }
                }
                IconButton(52.dp, onClick = { AudioQueue.next() }) { Icon(26.dp, c.fg) { skip(it, forward = true) } }
                IconButton(48.dp, onClick = {
                    // Leaves listening for the video page, picking up where the audio is.
                    val from = AudioQueue.progress().first
                    nav.replaceTop(Screen.Player(card.bvid, card.title, fromPlaylist = true, startMs = from))
                }) { Icon(24.dp, c.fg) { tv(it) } }
            }
            Text(modeLabel(mode), fontSize = 12.sp, color = c.fg2, textAlign = TextAlign.Center, modifier = Modifier.fillMaxWidth())
        }
        item {
            Column(Modifier.padding(top = 18.dp)) {
                Box(Modifier.fillMaxWidth().height(1.dp).background(c.line))
                Row(
                    Modifier.fillMaxWidth().clickable { open = !open }.padding(horizontal = 16.dp, vertical = 12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text("播放列表", fontSize = 18.sp, fontWeight = FontWeight.Light, color = c.fg)
                    val following = if (items.isEmpty()) null else items[(at + 1) % items.size]
                    val hint = if (open) "共 ${items.size} 首" else following?.let { "下一首：${it.title}" }.orEmpty()
                    Text(hint, fontSize = 12.sp, color = c.fg2, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f).padding(horizontal = 8.dp))
                    Icon(18.dp, c.fg) { chevron(it, up = open) }
                }
            }
        }
        if (open) {
            items(items, key = { it.bvid }) { item ->
                val on = item.bvid == card.bvid
                Row(
                    Modifier.fillMaxWidth().clickable { if (!on) AudioQueue.jump(item) }.padding(horizontal = 16.dp, vertical = 8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    Box(Modifier.width(4.dp).height(28.dp).background(if (on) c.accent else Color.Transparent))
                    AsyncImage(model = item.cover, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.width(48.dp).aspectRatio(16f / 9f).background(c.bg2))
                    Column(Modifier.weight(1f)) {
                        Text(item.title, fontSize = 14.sp, color = if (on) c.accent else c.fg, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        Text("${item.upName} · ${formatDuration(item.durationS)}", fontSize = 12.sp, color = c.fg2, maxLines = 1)
                    }
                }
            }
        }
        item { Box(Modifier.height(24.dp)) }
    }
}

@Composable
private fun IconButton(size: Dp, background: Color = Color.Transparent, onClick: () -> Unit, content: @Composable () -> Unit) {
    Box(Modifier.size(size).background(background).clickable(onClick = onClick), contentAlignment = Alignment.Center) { content() }
}

/** A square-cornered glyph drawn on a 24-unit grid. */
@Composable
private fun Icon(size: Dp, color: Color, draw: DrawScope.(Color) -> Unit) {
    Canvas(Modifier.size(size)) { draw(color) }
}

private fun DrawScope.u(v: Float) = v / 24f * size.width

private fun DrawScope.lines(color: Color, vararg points: Pair<Float, Float>) {
    val p = Path()
    points.forEachIndexed { i, (x, y) -> if (i == 0) p.moveTo(u(x), u(y)) else p.lineTo(u(x), u(y)) }
    drawPath(p, color, style = Stroke(width = u(2f), cap = StrokeCap.Square, join = StrokeJoin.Miter))
}

private fun DrawScope.fill(color: Color, vararg points: Pair<Float, Float>) {
    val p = Path()
    points.forEachIndexed { i, (x, y) -> if (i == 0) p.moveTo(u(x), u(y)) else p.lineTo(u(x), u(y)) }
    p.close()
    drawPath(p, color)
}

private fun DrawScope.bar(color: Color, x: Float, y: Float, w: Float, h: Float) =
    drawRect(color, Offset(u(x), u(y)), androidx.compose.ui.geometry.Size(u(w), u(h)))

private fun DrawScope.play(color: Color) = fill(color, 6f to 3.5f, 6f to 20.5f, 20.5f to 12f)

private fun DrawScope.pause(color: Color) {
    bar(color, 5f, 4f, 4.5f, 16f)
    bar(color, 14.5f, 4f, 4.5f, 16f)
}

private fun DrawScope.skip(color: Color, forward: Boolean) {
    if (forward) {
        fill(color, 3f to 4f, 3f to 20f, 16.5f to 12f)
        bar(color, 18f, 4f, 3f, 16f)
    } else {
        bar(color, 3f, 4f, 3f, 16f)
        fill(color, 21f to 4f, 21f to 20f, 7.5f to 12f)
    }
}

private fun DrawScope.modeIcon(color: Color, mode: FfiPlaylistMode) {
    when (mode) {
        FfiPlaylistMode.SEQUENTIAL -> {
            lines(color, 3f to 12f, 19f to 12f)
            lines(color, 14f to 7f, 19f to 12f, 14f to 17f)
        }
        FfiPlaylistMode.LOOP_ALL, FfiPlaylistMode.LOOP_ONE -> {
            lines(color, 4f to 11f, 4f to 7f, 19f to 7f)
            lines(color, 16f to 4f, 19f to 7f, 16f to 10f)
            lines(color, 20f to 13f, 20f to 17f, 5f to 17f)
            lines(color, 8f to 14f, 5f to 17f, 8f to 20f)
            if (mode == FfiPlaylistMode.LOOP_ONE) lines(color, 11f to 10.5f, 12.5f to 10.5f, 12.5f to 14f)
        }
        FfiPlaylistMode.SHUFFLE -> {
            lines(color, 3f to 7f, 7f to 7f, 15f to 17f, 20f to 17f)
            lines(color, 17f to 14f, 20f to 17f, 17f to 20f)
            lines(color, 3f to 17f, 7f to 17f, 9.5f to 14f)
            lines(color, 12.5f to 10f, 15f to 7f, 20f to 7f)
            lines(color, 17f to 4f, 20f to 7f, 17f to 10f)
        }
    }
}

private fun DrawScope.tv(color: Color) {
    lines(color, 2f to 5f, 22f to 5f, 22f to 18f, 2f to 18f, 2f to 5f)
    lines(color, 8f to 21f, 16f to 21f)
    fill(color, 10f to 8.5f, 10f to 14.5f, 15f to 11.5f)
}

private fun DrawScope.back(color: Color) {
    lines(color, 20f to 12f, 5f to 12f)
    lines(color, 11f to 6f, 5f to 12f, 11f to 18f)
}

private fun DrawScope.chevron(color: Color, up: Boolean) =
    if (up) lines(color, 4f to 16f, 12f to 8f, 20f to 16f) else lines(color, 4f to 8f, 12f to 16f, 20f to 8f)

/** `mm:ss`, or `h:mm:ss` from an hour. */
private fun clockOf(ms: Long): String {
    val s = ms.coerceAtLeast(0L) / 1000
    return if (s >= 3600) "%d:%02d:%02d".format(s / 3600, s / 60 % 60, s % 60) else "%02d:%02d".format(s / 60, s % 60)
}
