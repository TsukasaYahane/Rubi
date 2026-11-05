package dev.rubi.app.ui

import androidx.compose.foundation.border
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.Box
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.scale
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.rubi.app.ui.theme.LocalMetro

/**
 * Line icons for actions, on a 24-unit grid: square line ends and sharp corners, as Metro draws
 * them (no rounding). A coin and a clock keep their round outline; what is inside them is still
 * straight lines.
 */
enum class LineIcon(d: String, val ring: Boolean = false) {
    Like("M3 10h4v10H3z M7 10l4-7h1.5v6H20l-2 11H7"),
    Coin("M8.5 7.5l3.5 4.5 3.5-4.5 M12 12v6 M8.5 13.5h7 M8.5 16h7", ring = true),
    Star("M12 3l2.7 6 6.3.5-4.8 4.2 1.5 6.3L12 16.6 6.3 20l1.5-6.3L3 9.5l6.3-.5z"),
    Later("M12 7v5h4.5", ring = true),
    List("M4 6h15 M4 11h15 M4 16h8 M17 14v7 M13.5 17.5h7"),
    Listen("M4 18v-7l4-6h8l4 6v7 M4 14h4v6H4z M16 14h4v6h-4z"),
    Cache("M12 4v11 M7 10l5 5 5-5 M4 20h16"),
    Share("M14 4h6v6 M20 4l-9 9 M18 14v6H4V6h6"),
    Cast("M3 9V5h18v14h-7 M3 13h8v6 M3 16h5v3 M3 19h2"),
    Edit("M4 20h4L19 9l-4-4L4 16z M13 7l4 4"),
    Check("M4 4h16v16H4z M8 12l3 3 5-6"),
    Clear("M4 12l5 5 11-11 M4 20h16"),
    Delete("M5 7h14 M9 7V4h6v3 M7 7l1 13h8l1-13"),
    Move("M3 6h7l2 2h9v11H3z M10 13h7 M14 10l3 3-3 3"),
    Copy("M8 8h12v12H8z M4 16V4h12"),
    Comment("M4 5h16v11H10l-5 4v-4H4z"),
    More("M5 11h2v2H5z M11 11h2v2h-2z M17 11h2v2h-2z"),
    Eye("M2 12l5-6h10l5 6-5 6H7z M10 10h4v4h-4z"),
    Send("M3 12l18-8-6 16-3-7z"),
    Danmaku("M3 5h18v14H3z M7 10h6 M7 14h10"),
    DanmakuOff("M3 5h18v14H3z M7 10h6 M7 14h10 M4 20L20 4"),
    Pip("M3 5h18v14H3z M12 12h7v5h-7z");

    internal val path: Path = PathParser().parsePathString(d).toPath()
}

/** [dev.rubi.app.data.UiPrefs.uiStyle] value for 文字. */
const val UI_WORDS = 1

/**
 * 界面风格 文字: actions as words alone (the earlier look) instead of line icons with words.
 * 图形 is the default. 发弹幕 looks the same in both.
 */
val LocalWordsUi = staticCompositionLocalOf { false }

@Composable
fun LineIconView(icon: LineIcon, color: Color = LocalMetro.current.fg, size: Dp = 24.dp) {
    Canvas(Modifier.size(size)) {
        val k = this.size.width / 24f
        // The stroke stays 1.6dp whatever the size: drawn on the grid, then scaled.
        val stroke = Stroke(width = 1.6.dp.toPx() / k, cap = StrokeCap.Square, join = StrokeJoin.Miter)
        scale(k, pivot = Offset.Zero) {
            if (icon.ring) drawCircle(color, 8.5f, Offset(12f, 12f), style = stroke)
            drawPath(icon.path, color, style = stroke)
        }
    }
}

/**
 * An action as icon and words, without a frame: the quieter alternative to a filled button,
 * which stays for the main action of a page. `color` marks a destructive one (red) or a
 * disabled one (grey).
 */
@Composable
fun IconAction(icon: LineIcon, text: String, color: Color = LocalMetro.current.fg, onClick: () -> Unit) {
    if (LocalWordsUi.current) {
        // 文字: a thin frame round the words, in the action's colour.
        Box(
            Modifier.height(30.dp).border(1.dp, color).clickable(onClick = onClick).padding(horizontal = 12.dp),
            contentAlignment = Alignment.Center,
        ) { Text(text, fontSize = 13.sp, color = color) }
        return
    }
    Row(
        Modifier.heightIn(min = 36.dp).clickable(onClick = onClick).padding(horizontal = 4.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        LineIconView(icon, color, 18.dp)
        Text(text, fontSize = 13.sp, color = color)
    }
}
