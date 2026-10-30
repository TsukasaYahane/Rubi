package dev.rubi.app.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.sizeIn
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.horizontalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType

/** Big lowercase-style pivot headers; the active one is brighter and heavier. */
@Composable
fun Pivot(labels: List<String>, selected: Int, onSelect: (Int) -> Unit, style: TextStyle = MetroType.pivot) {
    val c = LocalMetro.current
    Row(
        Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(horizontal = 16.dp),
        horizontalArrangement = Arrangement.spacedBy(22.dp),
    ) {
        labels.forEachIndexed { i, label ->
            Text(
                label,
                style = style.copy(fontWeight = if (i == selected) FontWeight.Normal else FontWeight.Light),
                color = if (i == selected) c.fg else c.fg2,
                modifier = Modifier.sizeIn(minHeight = 44.dp).wrapContentSize(Alignment.CenterStart).clickable { onSelect(i) },
            )
        }
    }
}

@Composable
fun Meta(text: String, modifier: Modifier = Modifier, color: Color = LocalMetro.current.fg2) {
    Text(text, style = MetroType.meta, color = color, modifier = modifier)
}

@Composable
fun SectionTitle(text: String) {
    Text(text, style = MetroType.section, color = LocalMetro.current.fg, modifier = Modifier.padding(start = 16.dp, top = 28.dp, bottom = 4.dp))
}

/** Solid accent block with white text: the only "button" shape Metro uses. */
@Composable
fun AccentButton(text: String, onClick: () -> Unit) {
    Box(
        Modifier.height(44.dp).background(LocalMetro.current.accent).clickable(onClick = onClick).padding(horizontal = 22.dp),
        contentAlignment = Alignment.Center,
    ) { Text(text, color = Color.White, fontSize = 15.sp, fontWeight = FontWeight.SemiBold) }
}

/** Underlined text action. */
@Composable
fun TextAction(text: String, onClick: () -> Unit, dim: Boolean = false) {
    val c = LocalMetro.current
    Text(
        text,
        color = if (dim) c.fg2 else c.fg,
        fontSize = 15.sp,
        style = TextStyle(textDecoration = androidx.compose.ui.text.style.TextDecoration.Underline),
        modifier = Modifier.sizeIn(minHeight = 44.dp).wrapContentSize(Alignment.CenterStart).clickable(onClick = onClick),
    )
}

/** Small accent tag (推荐理由, UP). */
@Composable
fun Tag(text: String, modifier: Modifier = Modifier) {
    Text(
        text,
        style = MetroType.meta.copy(fontSize = 10.sp),
        color = Color.White,
        modifier = modifier.background(LocalMetro.current.accent).padding(horizontal = 6.dp, vertical = 1.dp),
    )
}

/** Rectangular Metro toggle: square thumb, accent fill when on. */
@Composable
fun MetroToggle(on: Boolean) {
    val c = LocalMetro.current
    Canvas(Modifier.size(44.dp, 22.dp)) {
        val stroke = 2.dp.toPx()
        if (on) drawRect(c.accent) else drawRect(c.fg, style = androidx.compose.ui.graphics.drawscope.Stroke(stroke))
        val tw = 10.dp.toPx()
        val inset = stroke + 2.dp.toPx()
        val x = if (on) size.width - inset - tw else inset
        drawRect(if (on) Color.White else c.fg, topLeft = Offset(x, inset), size = androidx.compose.ui.geometry.Size(tw, size.height - inset * 2))
    }
}

/** Geometric stroke icons drawn in code (no icon font, no emoji). */
enum class Glyph { Search, Mail, Back, More }

@Composable
fun GlyphIcon(glyph: Glyph, color: Color = LocalMetro.current.fg, size: Dp = 20.dp) {
    Canvas(Modifier.size(size)) {
        val s = this.size.width / 24f
        val w = 1.5.dp.toPx()
        fun line(x1: Float, y1: Float, x2: Float, y2: Float) =
            drawLine(color, Offset(x1 * s, y1 * s), Offset(x2 * s, y2 * s), w, StrokeCap.Square)
        when (glyph) {
            Glyph.Search -> {
                drawCircle(color, 6.5f * s, Offset(10.5f * s, 10.5f * s), style = androidx.compose.ui.graphics.drawscope.Stroke(w))
                line(15.5f, 15.5f, 21f, 21f)
            }
            Glyph.Mail -> {
                drawRect(color, Offset(3 * s, 5.5f * s), androidx.compose.ui.geometry.Size(18 * s, 13 * s), style = androidx.compose.ui.graphics.drawscope.Stroke(w))
                line(3f, 6f, 12f, 13f); line(12f, 13f, 21f, 6f)
            }
            Glyph.Back -> { line(13f, 5f, 6f, 12f); line(6f, 12f, 13f, 19f); line(6f, 12f, 20f, 12f) }
            Glyph.More -> listOf(4f, 10.75f, 17.5f).forEach {
                drawRect(color, Offset(it * s, 11 * s), androidx.compose.ui.geometry.Size(2.5f * s, 2.5f * s))
            }
        }
    }
}

fun formatCount(n: ULong): String = when {
    n >= 100_000_000uL -> "%.1f亿".format(n.toDouble() / 1e8)
    n >= 10_000uL -> "%.1f万".format(n.toDouble() / 1e4)
    else -> n.toString()
}

fun formatDuration(s: UInt): String {
    val t = s.toInt()
    return if (t >= 3600) "%d:%02d:%02d".format(t / 3600, t / 60 % 60, t % 60) else "%02d:%02d".format(t / 60, t % 60)
}

fun formatAge(unixSecs: Long): String {
    val d = System.currentTimeMillis() / 1000 - unixSecs
    return when {
        unixSecs <= 0 -> ""
        d < 3600 -> "${(d / 60).coerceAtLeast(1)}分钟前"
        d < 86_400 -> "${d / 3600}小时前"
        d < 7 * 86_400 -> "${d / 86_400}天前"
        else -> java.text.SimpleDateFormat("yyyy-MM-dd", java.util.Locale.CHINA).format(java.util.Date(unixSecs * 1000))
    }
}
