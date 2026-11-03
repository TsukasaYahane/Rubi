package dev.rubi.app.ui

import dev.rubi.app.LocalFrozen
import dev.rubi.app.playback.LocalMedia
import android.graphics.Paint
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameMillis
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.drawText
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.sp
import androidx.media3.common.Player
import coil3.SingletonImageLoader
import coil3.request.ImageRequest
import coil3.request.SuccessResult
import coil3.request.allowHardware
import coil3.toBitmap
import dev.rubi.app.data.DanmakuDisplay
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
/** Emote images are drawn this many times the font size, square. */
private const val EMOTE_EM = 1.3f

/** A run of a danmaku's text: plain text, or an emote token with a known image. */
private sealed interface Seg {
    data class Text(val text: String) : Seg
    data class Emote(val url: String) : Seg
}

/** Splits `text` at emote tokens found in `emotes`; null when there are none (plain path). */
private fun segments(text: String, emotes: Map<String, String>): List<Seg>? {
    if (emotes.isEmpty() || '[' !in text) return null
    val out = mutableListOf<Seg>()
    var last = 0
    for (m in EMOTE_TOKEN.findAll(text)) {
        val url = emotes[m.value] ?: continue
        if (m.range.first > last) out += Seg.Text(text.substring(last, m.range.first))
        out += Seg.Emote(url)
        last = m.range.last + 1
    }
    if (out.none { it is Seg.Emote }) return null
    if (last < text.length) out += Seg.Text(text.substring(last))
    return out
}

/**
 * Draws danmaku over the player. Rust assigns tracks once per video and display setting
 * ([RubiCore.layoutDanmaku], with widths measured here by [Paint]); this only positions what
 * is on screen at the current time. `[xxx]` tokens of the global emote set are drawn as images.
 */
@Composable
fun DanmakuLayer(
    core: RubiCore,
    set: FfiDanmakuSet,
    display: DanmakuDisplay,
    modifier: Modifier = Modifier,
    /** The player whose clock drives the danmaku; the main player by default. */
    source: Player? = null,
) {
    BoxWithConstraints(modifier) {
        val fontSp = BASE_FONT_SP * display.fontScale
        val lineDp = BASE_LINE_DP * display.fontScale
        val tracks = ((maxHeight.value * display.area) / lineDp).toInt().coerceAtLeast(1)
        val widthDp = maxWidth.value
        val scrollMs = display.scrollMs
        val emotes = LocalEmotes.current
        val ctx = LocalContext.current

        val shown = remember(set, display) { set.items.filter { visible(it, display) } }
        val byId = remember(shown) { shown.associateBy { it.id } }
        val segs = remember(shown, emotes) { shown.mapNotNull { d -> segments(d.content, emotes)?.let { d.id to it } }.toMap() }
        val screen = LocalDensity.current
        val fontPx = with(screen) { fontSp.sp.toPx() }
        val emotePx = fontPx * EMOTE_EM
        val placements by produceState(emptyList<FfiPlacement>(), shown, segs, widthDp, fontSp, tracks, scrollMs) {
            value = withContext(Dispatchers.Default) {
                // Real widths for track spacing, measured like the text is drawn (same size,
                // default typeface, emotes as squares), in dp like the viewport.
                val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply { textSize = fontPx }
                val widths = shown.map { d ->
                    val px = segs[d.id]?.sumOf { s ->
                        when (s) {
                            is Seg.Text -> paint.measureText(s.text).toDouble()
                            is Seg.Emote -> emotePx.toDouble()
                        }
                    }?.toFloat() ?: paint.measureText(d.content)
                    px / screen.density
                }
                core.layoutDanmaku(shown, widths, widthDp, fontSp, tracks.toUInt(), scrollMs.toUInt())
            }
        }

        // Emote images used by this set, decoded once into bitmaps the canvas can draw.
        val bitmaps = remember { mutableStateMapOf<String, ImageBitmap>() }
        LaunchedEffect(segs) {
            val loader = SingletonImageLoader.get(ctx)
            val urls = segs.values.flatten().filterIsInstance<Seg.Emote>().map { it.url }.distinct()
            for (url in urls) {
                if (url in bitmaps) continue
                val req = ImageRequest.Builder(ctx).data(url).size(emotePx.toInt().coerceAtLeast(1)).allowHardware(false).build()
                (loader.execute(req) as? SuccessResult)?.let { bitmaps[url] = it.image.toBitmap().asImageBitmap() }
            }
        }

        val player = source ?: LocalMedia.current.player.get()
        var pos by remember { mutableLongStateOf(0L) }
        // Every frame while awake; frozen (节电) the danmaku hold still.
        val frozen = LocalFrozen.current
        LaunchedEffect(player, frozen) { if (!frozen) while (true) withFrameMillis { pos = player.currentPosition } }

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
                val style = base.copy(color = Color(0xFF000000.toInt() or d.color.toInt()))
                val parts = segs[d.id]
                if (parts == null) {
                    val layout = measurer.measure(d.content, style)
                    drawText(layout, topLeft = origin(p, age, scrollMs, layout.size.width.toFloat(), lineH), alpha = display.opacity)
                    continue
                }
                // Mixed text and emotes: text runs measured here, emotes as fixed squares.
                val runs = parts.map { s -> s to (s as? Seg.Text)?.let { measurer.measure(it.text, style) } }
                val tw = runs.sumOf { (_, l) -> (l?.size?.width?.toFloat() ?: emotePx).toDouble() }.toFloat()
                val o = origin(p, age, scrollMs, tw, lineH)
                var x = o.x
                for ((s, l) in runs) {
                    if (l != null) {
                        drawText(l, topLeft = Offset(x, o.y), alpha = display.opacity)
                        x += l.size.width
                        continue
                    }
                    bitmaps[(s as Seg.Emote).url]?.let { img ->
                        val side = emotePx.toInt()
                        val top = o.y + (lineH - emotePx) / 2
                        drawImage(img, dstOffset = IntOffset(x.toInt(), top.toInt()), dstSize = IntSize(side, side), alpha = display.opacity)
                    }
                    x += emotePx
                }
            }
        }
    }
}

/** Top-left of a danmaku `tw` wide, `age` ms into its life, in canvas pixels. */
private fun DrawScope.origin(p: FfiPlacement, age: Long, scrollMs: Long, tw: Float, lineH: Float): Offset =
    when (p.lane) {
        FfiLane.SCROLL -> Offset(size.width - age.toFloat() / scrollMs * (size.width + tw), p.track.toFloat() * lineH)
        FfiLane.TOP -> Offset((size.width - tw) / 2, p.track.toFloat() * lineH)
        FfiLane.BOTTOM -> Offset((size.width - tw) / 2, size.height - (p.track.toFloat() + 1) * lineH)
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
