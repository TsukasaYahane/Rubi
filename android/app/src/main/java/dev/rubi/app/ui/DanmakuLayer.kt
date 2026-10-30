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
import dev.rubi.app.playback.PlayerHolder
import dev.rubi.core.FfiDanmakuSet
import dev.rubi.core.FfiLane
import dev.rubi.core.FfiPlacement
import dev.rubi.core.RubiCore
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

// Must match rubi_filter::danmaku::layout::LayoutConfig defaults.
private const val SCROLL_MS = 8_000L
private const val FIXED_MS = 4_000L
private const val FONT_SP = 14f
private const val LINE_DP = 20f
/** Danmaku use the top part of the video only, so faces and subtitles stay clear. */
private const val AREA = 0.75f

/**
 * Draws danmaku over the player. Rust assigns tracks once per video ([RubiCore.layoutDanmaku]);
 * this only positions what is on screen at the current playback time.
 */
@Composable
fun DanmakuLayer(core: RubiCore, set: FfiDanmakuSet, modifier: Modifier = Modifier) {
    BoxWithConstraints(modifier) {
        val widthDp = maxWidth.value
        val tracks = ((maxHeight.value * AREA) / LINE_DP).toInt().coerceAtLeast(1)
        val byId = remember(set) { set.items.associateBy { it.id } }
        val placements by produceState(emptyList<FfiPlacement>(), set, widthDp, tracks) {
            value = withContext(Dispatchers.Default) { core.layoutDanmaku(set.items, widthDp, FONT_SP, tracks.toUInt()) }
        }
        val player = PlayerHolder.get(LocalContext.current)
        var pos by remember { mutableLongStateOf(0L) }
        LaunchedEffect(player) { while (true) withFrameMillis { pos = player.currentPosition } }

        val measurer = rememberTextMeasurer(cacheSize = 256)
        val base = remember { TextStyle(fontSize = FONT_SP.sp, shadow = Shadow(Color.Black, blurRadius = 3f)) }

        Canvas(Modifier.fillMaxSize()) {
            val lineH = LINE_DP * density
            val now = pos
            var i = firstStartingAfter(placements, now - SCROLL_MS)
            while (i < placements.size) {
                val p = placements[i++]
                val t0 = p.startMs.toLong()
                if (t0 > now) break
                val age = now - t0
                val d = byId[p.id] ?: continue
                val layout = measurer.measure(d.content, base.copy(color = Color(0xFF000000.toInt() or d.color.toInt())))
                val tw = layout.size.width.toFloat()
                val offset = when (p.lane) {
                    FfiLane.SCROLL -> {
                        if (age >= SCROLL_MS) continue
                        Offset(size.width - age.toFloat() / SCROLL_MS * (size.width + tw), p.track.toFloat() * lineH)
                    }
                    FfiLane.TOP -> {
                        if (age >= FIXED_MS) continue
                        Offset((size.width - tw) / 2, p.track.toFloat() * lineH)
                    }
                    FfiLane.BOTTOM -> {
                        if (age >= FIXED_MS) continue
                        Offset((size.width - tw) / 2, size.height - (p.track.toFloat() + 1) * lineH)
                    }
                }
                drawText(layout, topLeft = offset)
            }
        }
    }
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
