package dev.rubi.app.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.rubi.app.ui.theme.LocalMetro
import kotlinx.coroutines.delay

/** How long the hint stays. */
private const val SHOW_SECONDS = 5

/**
 * 继续观看: over the bottom of the video, a mark at where the user stopped last time (on a track
 * of the part's length) and a chip above it, 「上次看到 mm:ss · 点此继续」. Playback itself
 * starts from the beginning; tapping jumps there. Gone after five seconds, or once used.
 */
@Composable
fun ResumeHint(positionMs: Long, durationMs: Long, otherPart: String?, onResume: () -> Unit, onGone: () -> Unit, modifier: Modifier = Modifier) {
    val c = LocalMetro.current
    var left by remember { mutableIntStateOf(SHOW_SECONDS) }
    LaunchedEffect(Unit) {
        while (left > 0) {
            delay(1_000)
            left--
        }
        onGone()
    }
    val fraction = if (durationMs > 0) (positionMs.toFloat() / durationMs).coerceIn(0f, 1f) else 0f
    BoxWithConstraints(modifier.fillMaxWidth().padding(start = 12.dp, end = 12.dp, bottom = 52.dp)) {
        val full = maxWidth
        val x = full * fraction
        var chipWidth by remember { mutableIntStateOf(0) }
        val chipDp = with(LocalDensity.current) { chipWidth.toDp() }
        Column {
            // The chip, centred over the mark but kept inside the video.
            Column(
                Modifier.offset(x = (x - chipDp / 2).coerceIn(0.dp, (full - chipDp).coerceAtLeast(0.dp)))
                    .onSizeChanged { chipWidth = it.width }
                    .clickable { onResume() },
            ) {
                Text(
                    buildString {
                        append("上次看到 ")
                        otherPart?.let { append("$it ") }
                        append(clockText(positionMs))
                        append(" · 点此继续")
                    },
                    fontSize = 12.sp,
                    fontWeight = FontWeight.SemiBold,
                    color = Color.White,
                    modifier = Modifier.background(c.accent).padding(horizontal = 10.dp, vertical = 5.dp),
                )
                Text("${left} 秒后隐藏", fontSize = 10.sp, color = Color.White.copy(alpha = 0.7f), modifier = Modifier.wrapContentWidth())
            }
            Box(Modifier.padding(top = 4.dp).fillMaxWidth().height(11.dp)) {
                Box(Modifier.align(Alignment.Center).fillMaxWidth().height(3.dp).background(Color.White.copy(alpha = 0.3f)))
                Box(Modifier.offset(x = x - 1.5.dp).width(3.dp).height(11.dp).background(c.accent).clickable { onResume() })
            }
        }
    }
}

/** `mm:ss`, or `h:mm:ss` from an hour. */
private fun clockText(ms: Long): String {
    val s = ms.coerceAtLeast(0L) / 1000
    return if (s >= 3600) "%d:%02d:%02d".format(s / 3600, s / 60 % 60, s % 60) else "%02d:%02d".format(s / 60, s % 60)
}
