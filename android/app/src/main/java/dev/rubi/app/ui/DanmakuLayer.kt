package dev.rubi.app.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameMillis
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.drawText
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.unit.sp
import dev.rubi.app.data.DanmakuDisplay
import dev.rubi.app.playback.PlayerHolder
import dev.rubi.core.FfiDanmaku
import dev.rubi.core.FfiDanmakuSet
import dev.rubi.core.FfiLane
import dev.rubi.core.FfiPlacement
import dev.rubi.core.RubiCore
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

private const val BASE_FONT_SP = 14f
private const val BASE_LINE_DP = 20f
private const val WHITE = 0xFFFFFFu

/**
 * Draws danmaku over the player. Rust assigns tracks once per video and display setting
 * ([RubiCore.layoutDanmaku]); this only positions what is on screen at the current time.
 */
@Composable
fun DanmakuLayer(core: RubiCore, set: FfiDanmakuSet, display: DanmakuDisplay, modifier: Modifier = Modifier) {
    BoxWithConstraints(modifier) {
        val fontSp = BASE_FONT_SP * display.fontScale
        val lineDp = BASE_LINE_DP * display.fontScale
        val tracks = ((maxHeight.value * display.area) / lineDp).toInt().coerceAtLeast(1)
        val widthDp = maxWidth.value
        val scrollMs = display.scrollMs

        val shown = remember(set, display) { set.items.filter { visible(it, display) } }
        val byId = remember(shown) { shown.associateBy { it.id } }
        val placements by produceState(emptyList<FfiPlacement>(), shown, widthDp, fontSp, tracks, scrollMs) {
            value = withContext(Dispatchers.Default) {
                core.layoutDanmaku(shown, widthDp, fontSp, tracks.toUInt(), scrollMs.toUInt())
            }
        }
        val player = PlayerHolder.get(LocalContext.current)
        var pos by remember { mutableLongStateOf(0L) }
        LaunchedEffect(player) { while (true) withFrameMillis { pos = player.currentPosition } }

        val measurer = rememberTextMeasurer(cacheSize = 256)
        val base = remember(fontSp) { TextStyle(fontSize = fontSp.sp, shadow = Shadow(Color.Black, blurRadius = 3f)) }

        Canvas(Modifier.fillMaxSize()) {
            val lineH = lineDp * density
            val now = pos
            var i = firstStartingAfter(placements, now - scrollMs)
            while (i < placements.size) {
                val p = placements[i++]
                val t0 = p.startMs.toLong()
                if (t0 > now) break
                val age = now - t0
                val d = byId[p.id] ?: continue
                val life = if (p.lane == FfiLane.SCROLL) scrollMs else DanmakuDisplay.FIXED_MS
                if (age >= life) continue
                val layout = measurer.measure(d.content, base.copy(color = Color(0xFF000000.toInt() or d.color.toInt())))
                val tw = layout.size.width.toFloat()
                val offset = when (p.lane) {
                    FfiLane.SCROLL -> Offset(size.width - age.toFloat() / scrollMs * (size.width + tw), p.track.toFloat() * lineH)
                    FfiLane.TOP -> Offset((size.width - tw) / 2, p.track.toFloat() * lineH)
                    FfiLane.BOTTOM -> Offset((size.width - tw) / 2, size.height - (p.track.toFloat() + 1) * lineH)
                }
                drawText(layout, topLeft = offset, alpha = display.opacity)
            }
        }
    }
}

private fun visible(d: FfiDanmaku, display: DanmakuDisplay): Boolean {
    val laneOk = when (d.mode.toInt()) {
        1, 2, 3, 6 -> display.showScroll
        5 -> display.showTop
        4 -> display.showBottom
        else -> false
    }
    return laneOk && (display.showColored || (d.color and WHITE) == WHITE)
}

/** Index of the first placement whose start is >= [t] (placements are sorted by start). */
private fun firstStartingAfter(list: List<FfiPlacement>, t: Long): Int {
    var lo = 0
    var hi = list.size
    while (lo < hi) {
        val mid = (lo + hi) ushr 1
        if (list[mid].startMs.toLong() < t) lo = mid + 1 else hi = mid
    }
    return lo
}
