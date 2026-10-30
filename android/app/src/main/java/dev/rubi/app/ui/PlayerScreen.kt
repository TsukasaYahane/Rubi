package dev.rubi.app.ui

import android.util.Log
import android.view.LayoutInflater
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.media3.ui.PlayerView
import dev.rubi.app.R
import dev.rubi.app.RubiApp
import dev.rubi.app.data.DanmakuDisplay
import dev.rubi.app.playback.PlayerHolder
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiCommentRow
import dev.rubi.core.FfiCommentThread
import dev.rubi.core.FfiDanmakuSet
import dev.rubi.core.FfiSignal
import dev.rubi.core.FfiVideoCard
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

private const val TAG = "RubiPlayer"

@Composable
fun PlayerScreen(app: RubiApp, nav: Nav, bvid: String, title: String) {
    val c = LocalMetro.current
    val ctx = LocalContext.current
    var video by remember { mutableStateOf<FfiVideoCard?>(null) }
    var danmaku by remember { mutableStateOf<FfiDanmakuSet?>(null) }
    var error by remember { mutableStateOf<String?>(null) }
    var tab by remember { mutableStateOf(0) }
    var display by remember { mutableStateOf(DanmakuDisplay.load(app.core)) }
    var dmStatus by remember { mutableStateOf("弹幕加载中…") }
    var dmSheet by remember { mutableStateOf(false) }

    LaunchedEffect(bvid) {
        runCatching {
            app.core.record(bvid, FfiSignal.Click)
            val v = app.core.video(bvid).also { video = it }
            val playback = app.core.playback(bvid, 80u)
            PlayerHolder.play(ctx, playback, v.title)
            val durationMs = maxOf(v.durationS.toULong() * 1000u, playback.durationMs).toUInt()
            runCatching { app.core.danmaku(playback.cid, durationMs) }
                .onSuccess { set ->
                    danmaku = set
                    dmStatus = "弹幕 ${set.items.size} · 空降点 ${set.jumpMarks.size}"
                    Log.i(TAG, "danmaku loaded: ${set.items.size} items, removed ${set.removed}")
                }
                .onFailure { e ->
                    dmStatus = "弹幕加载失败：${e.message}"
                    Log.w(TAG, "danmaku load failed", e)
                }
        }.onFailure { error = it.message }
    }

    Column(Modifier.fillMaxSize()) {
        Row(Modifier.fillMaxWidth().padding(start = 4.dp, end = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(44.dp).clickable { nav.pop() }, contentAlignment = Alignment.Center) { GlyphIcon(Glyph.Back) }
            Meta(dmStatus, Modifier.weight(1f))
            Text(
                if (display.enabled) "弹 开" else "弹 关",
                style = MetroType.meta.copy(fontSize = 12.sp),
                color = if (display.enabled) c.fg else c.fg2,
                modifier = Modifier.size(52.dp, 44.dp).wrapContentSize(Alignment.CenterEnd).clickable {
                    display = display.copy(enabled = !display.enabled)
                    runCatching { DanmakuDisplay.save(app.core, display) }
                },
            )
            Text(
                "设置",
                style = MetroType.meta.copy(fontSize = 12.sp),
                color = c.fg,
                modifier = Modifier.size(44.dp, 44.dp).wrapContentSize(Alignment.CenterEnd).clickable { dmSheet = true },
            )
        }
        Box(Modifier.fillMaxWidth().aspectRatio(16f / 9f).background(Color.Black)) {
            AndroidView(
                factory = {
                    (LayoutInflater.from(it).inflate(R.layout.player_view, null) as PlayerView).apply { player = PlayerHolder.get(it) }
                },
                modifier = Modifier.fillMaxSize(),
            )
            val set = danmaku
            if (display.enabled && set != null) DanmakuLayer(app.core, set, display, Modifier.fillMaxSize())
        }
        danmaku?.jumpMarks?.takeIf { it.isNotEmpty() }?.let { marks ->
            Row(Modifier.padding(horizontal = 16.dp, vertical = 4.dp), horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                Meta("空降点")
                marks.forEach { m ->
                    Text(m.label, style = MetroType.meta, color = c.fg, modifier = Modifier.clickable {
                        PlayerHolder.get(ctx).seekTo(m.targetMs.toLong())
                    })
                }
            }
        }
        LazyColumn(Modifier.weight(1f)) {
            item {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(video?.title ?: title, style = MetroType.title, color = c.fg)
                    video?.let { Meta("${formatCount(it.views)} 播放 · ${formatAge(it.pubdate)} · ${it.zone ?: ""}") }
                    video?.let { Text(it.upName, fontSize = 15.sp, fontWeight = FontWeight.SemiBold, color = c.fg) }
                    error?.let { Text(it, color = c.fg2, fontSize = 13.sp) }
                    danmaku?.removed?.takeIf { it.isNotEmpty() }?.let { Meta("已过滤 · " + it.joinToString(" · ")) }
                }
                Pivot(listOf("推荐", "评论"), tab, { tab = it }, MetroType.section)
            }
            if (tab == 0) {
                item { RelatedList(app, nav, bvid) }
            } else {
                video?.let { v -> item { CommentSection(app, v.aid) } }
            }
        }
    }

    if (dmSheet) {
        DanmakuSettingsSheet(
            display = display,
            onChange = {
                display = it
                runCatching { DanmakuDisplay.save(app.core, it) }
            },
            onDismiss = { dmSheet = false },
        )
    }
}

@Composable
private fun RelatedList(app: RubiApp, nav: Nav, current: String) {
    val cards = remember { mutableStateListOf<FfiVideoCard>() }
    LaunchedEffect(current) {
        val page = withContext(Dispatchers.IO) { runCatching { app.core.nextPage(6u) }.getOrDefault(emptyList()) }
        cards.addAll(page.filter { it.bvid != current })
    }
    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(24.dp)) {
        cards.forEach { card -> VideoCardView(card, big = false, onOpen = { nav.push(Screen.Player(card.bvid, card.title)) }) }
    }
}

@Composable
private fun CommentSection(app: RubiApp, aid: ULong) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    var byTime by remember { mutableStateOf(false) }
    var pinned by remember { mutableStateOf(emptyList<FfiCommentRow>()) }
    var hiddenPinned by remember { mutableStateOf(0u) }
    val threads = remember { mutableStateListOf<FfiCommentThread>() }
    val collapsed = remember { mutableStateMapOf<ULong, Boolean>() }
    var error by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(aid, byTime) {
        runCatching { app.core.comments(aid, byTime, null) }
            .onSuccess { page ->
                pinned = page.pinned
                hiddenPinned = page.hiddenPinned
                threads.clear()
                threads.addAll(page.threads)
            }
            .onFailure { error = it.message }
    }

    Column(Modifier.padding(horizontal = 16.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Meta("已隐藏置顶带货评论 $hiddenPinned 条", Modifier.weight(1f))
            listOf("按热度" to false, "按时间" to true).forEach { (label, t) ->
                Text(
                    label,
                    style = MetroType.meta.copy(fontSize = 12.sp),
                    color = if (byTime == t) c.fg else c.fg2,
                    fontWeight = if (byTime == t) FontWeight.SemiBold else FontWeight.Normal,
                    modifier = Modifier.padding(start = 16.dp).size(width = 56.dp, height = 44.dp).clickable { byTime = t },
                )
            }
        }
        error?.let { Text(it, color = c.fg2, fontSize = 13.sp) }
        pinned.forEach { CommentRow(it, open = BooleanArray(0), pinnedLabel = true) }
        threads.forEachIndexed { index, thread ->
            val isCollapsed = collapsed[thread.rootRpid] == true
            val rows = if (isCollapsed) thread.rows.take(1) else thread.rows
            Column(Modifier.padding(vertical = 10.dp)) {
                val open = BooleanArray(16)
                rows.forEach { row ->
                    // Snapshot: drawing happens later, after `open` has moved on.
                    CommentRow(row, open.copyOf())
                    if (row.depth.toInt() in 1 until open.size) open[row.depth.toInt()] = row.hasLaterSibling
                }
                val shown = (thread.rows.size - 1).toUInt()
                val actions = buildList {
                    if (isCollapsed) add("展开 ${thread.totalReplies} 条回复" to { collapsed[thread.rootRpid] = false })
                    else {
                        if (thread.hidden > 0u) add("再看 5 条 · 剩 ${thread.hidden}" to {
                            scope.launch {
                                runCatching { app.core.commentThread(aid, thread.rootRpid, shown + 5u) }
                                    .onSuccess { threads[index] = it }
                            }
                            Unit
                        })
                        if (thread.rows.size > 1) add("折叠" to { collapsed[thread.rootRpid] = true })
                    }
                }
                if (actions.isNotEmpty()) {
                    Row(Modifier.padding(start = 22.dp), horizontalArrangement = Arrangement.spacedBy(20.dp)) {
                        actions.forEach { (label, act) -> TextAction(label, onClick = act) }
                    }
                }
            }
        }
    }
}

/**
 * One fishbone row. `open[l]` tells whether the ancestor branch at depth `l` continues
 * below this row, so its vertical line is drawn through.
 */
@Composable
private fun CommentRow(row: FfiCommentRow, open: BooleanArray, pinnedLabel: Boolean = false) {
    val c = LocalMetro.current
    val depth = row.depth.toInt()
    val indent = if (depth == 0) 0.dp else ((depth - 1) * 20 + 22).dp
    Column(
        Modifier.fillMaxWidth()
            .drawBehind {
                if (depth == 0) return@drawBehind
                val stroke = 1.dp.toPx()
                val tick = 16.dp.toPx()
                fun x(l: Int) = ((l - 1) * 20 + 6).dp.toPx()
                for (l in 1 until depth) {
                    if (l < open.size && open[l]) drawLine(c.spine, Offset(x(l), 0f), Offset(x(l), size.height), stroke)
                }
                val xd = x(depth)
                drawLine(c.spine, Offset(xd, 0f), Offset(xd, if (row.hasLaterSibling) size.height else tick), stroke)
                drawLine(c.spine, Offset(xd, tick), Offset(xd + 10.dp.toPx(), tick), stroke)
            }
            .padding(start = indent, top = 6.dp, bottom = 6.dp),
        verticalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            if (pinnedLabel) Tag("置顶")
            Text(row.name, fontSize = if (depth == 0) 14.sp else 13.sp, fontWeight = FontWeight.SemiBold, color = c.fg)
            if (row.isUp) Tag("UP")
            Meta(formatAge(row.ctime))
        }
        EmoteText(
            (row.replyTo?.let { "回复 @$it：" } ?: "") + row.text,
            color = c.fg,
            fontSize = 14.sp,
            lineHeight = 21.sp,
            extra = row.emotes,
        )
        Meta("赞 ${row.likes}")
    }
}
