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
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.layout
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.core.FfiEpisode
import dev.rubi.core.FfiPart
import dev.rubi.core.FfiSeason

/** Lets a child run edge to edge out of a column padded by `by` on each side. */
fun Modifier.bleed(by: Dp): Modifier = layout { measurable, constraints ->
    val extra = (by * 2).roundToPx()
    val placeable = measurable.measure(constraints.copy(minWidth = constraints.minWidth + extra, maxWidth = constraints.maxWidth + extra))
    layout(placeable.width - extra, placeable.height) { placeable.place(-extra / 2, 0) }
}

/**
 * 分P: one square tile per part, swiped sideways; each shows its number, length, title and how
 * far it was watched. The part playing has an accent edge.
 */
@Composable
fun PartsRow(parts: List<FfiPart>, current: Int, progressOf: (FfiPart) -> Float, onPick: (Int) -> Unit) {
    val c = LocalMetro.current
    val list = rememberLazyListState()
    LaunchedEffect(current) { if (current in parts.indices) list.animateScrollToItem(maxOf(0, current - 1)) }
    Column {
        Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 6.dp), verticalAlignment = Alignment.Bottom) {
            Text("分 P", fontSize = 17.sp, fontWeight = FontWeight.Light, color = c.fg)
            Text(" (${parts.size})", fontSize = 12.sp, color = c.fg2, modifier = Modifier.weight(1f))
            Text("左右滑动 · 记住每 P 进度", fontSize = 12.sp, color = c.fg2)
        }
        LazyRow(state = list, contentPadding = PaddingValues(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            itemsIndexed(parts, key = { _, p -> p.cid.toString() }) { i, part ->
                val on = i == current
                Row(Modifier.width(128.dp).background(if (on) c.accent.copy(alpha = 0.18f) else c.bg2).clickable { onPick(i) }) {
                    Box(Modifier.width(3.dp).height(64.dp).background(if (on) c.accent else Color.Transparent))
                    Column(Modifier.padding(horizontal = 8.dp, vertical = 8.dp)) {
                        Text("P${i + 1} · ${formatDuration(part.durationS)}", fontSize = 12.sp, color = c.fg2)
                        Text(part.title, fontSize = 14.sp, color = c.fg, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        Box(Modifier.padding(top = 6.dp).fillMaxWidth().height(2.dp).background(c.line)) {
                            Box(Modifier.fillMaxWidth(progressOf(part).coerceIn(0f, 1f)).fillMaxHeight().background(c.accent))
                        }
                    }
                }
            }
        }
    }
}

/**
 * The 合集 the video is in: folded to its name and what comes next; unfolded, every video by
 * section with the playing one marked.
 */
@Composable
fun SeasonBlock(season: FfiSeason, currentBvid: String, subscribe: (@Composable () -> Unit)? = null, onPick: (FfiEpisode) -> Unit) {
    val c = LocalMetro.current
    var open by remember { mutableStateOf(false) }
    val all = season.sections.flatMap { it.episodes }
    val at = all.indexOfFirst { it.bvid == currentBvid }
    val next = all.getOrNull(at + 1)
    Column {
        Box(Modifier.fillMaxWidth().height(1.dp).background(c.line))
        Row(
            Modifier.fillMaxWidth().clickable { open = !open }.padding(horizontal = 16.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text("合集 · ${season.title}", fontSize = 17.sp, fontWeight = FontWeight.Light, color = c.fg, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f, fill = false))
            Text(" (第 ${at + 1} / ${all.size} 个)", fontSize = 12.sp, color = c.fg2, modifier = Modifier.weight(1f))
            subscribe?.let { Box(Modifier.padding(end = 10.dp)) { it() } }
            Text(if (open) "收起 ▴" else "展开 ▾", fontSize = 14.sp, color = c.fg)
        }
        if (!open) {
            Text(next?.let { "下一个：${it.title}" } ?: "已是最后一个", fontSize = 12.sp, color = c.fg2, modifier = Modifier.padding(start = 16.dp, end = 16.dp, bottom = 10.dp))
        } else {
            season.sections.forEach { section ->
                if (season.sections.size > 1) Text(section.title, fontSize = 12.sp, color = c.fg2, modifier = Modifier.padding(start = 16.dp, top = 8.dp, bottom = 2.dp))
                section.episodes.forEach { ep ->
                    val on = ep.bvid == currentBvid
                    Row(
                        Modifier.fillMaxWidth().clickable(enabled = !on) { onPick(ep) }.padding(horizontal = 16.dp, vertical = 8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        Box(Modifier.width(4.dp).height(32.dp).background(if (on) c.accent else Color.Transparent))
                        AsyncImage(model = ep.cover, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.width(64.dp).aspectRatio(16f / 9f).background(c.bg2))
                        Column(Modifier.weight(1f)) {
                            Text(ep.title, fontSize = 14.sp, color = if (on) c.accent else c.fg, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            Text(formatDuration(ep.durationS), fontSize = 12.sp, color = c.fg2)
                        }
                    }
                }
            }
        }
    }
}

/** 播完自动连播下一个, with the square switch. */
@Composable
fun AutoNextRow(on: Boolean, onToggle: () -> Unit) {
    val c = LocalMetro.current
    Column {
        Box(Modifier.fillMaxWidth().height(1.dp).background(c.line))
        Row(
            Modifier.fillMaxWidth().heightIn(min = 48.dp).clickable(onClick = onToggle).padding(horizontal = 16.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text("播完自动连播下一个", fontSize = 14.sp, color = c.fg)
            Text(if (on) " 开" else " 关", fontSize = 12.sp, color = c.fg2, modifier = Modifier.weight(1f))
            SquareSwitch(on)
        }
    }
}

/**
 * A square-cornered switch: an outlined bar with a white block in one half. Off: the block on
 * the left, the rest grey. On: the block on the right, the rest in accent.
 */
@Composable
fun SquareSwitch(on: Boolean) {
    val c = LocalMetro.current
    Box(
        Modifier.size(width = 48.dp, height = 24.dp).background(if (on) c.accent else c.fg2).border(2.dp, c.fg),
        contentAlignment = if (on) Alignment.CenterEnd else Alignment.CenterStart,
    ) {
        Box(Modifier.width(24.dp).fillMaxHeight().padding(2.dp).background(Color.White))
    }
}
