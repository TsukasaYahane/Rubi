package dev.rubi.app.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import coil3.compose.AsyncImage
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.core.FfiPicture

private val THUMB = 80.dp
private val GAP = 6.dp

/**
 * Images attached to a comment or post, in one row of thumbnails. What does not fit is counted
 * on the last one ("+N"); tapping any opens a viewer that swipes through all of them.
 */
@Composable
fun CommentPictures(pictures: List<FfiPicture>) {
    val c = LocalMetro.current
    var open by remember { mutableStateOf<Int?>(null) }
    BoxWithConstraints(Modifier.padding(top = 6.dp)) {
        val fit = ((maxWidth + GAP) / (THUMB + GAP)).toInt().coerceAtLeast(1)
        val shown = pictures.take(fit)
        val more = pictures.size - shown.size
        Row(horizontalArrangement = Arrangement.spacedBy(GAP)) {
            shown.forEachIndexed { i, p ->
                // A lone picture keeps its shape (within limits); several are squares.
                val single = pictures.size == 1 && p.width > 0u && p.height > 0u
                val ratio = if (single) (p.width.toFloat() / p.height.toFloat()).coerceIn(0.5f, 2f) else 1f
                Box(Modifier.height(if (single) 120.dp else THUMB).width(if (single) 120.dp * ratio else THUMB).background(c.bg2).clickable { open = i }) {
                    AsyncImage(model = p.url, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
                    if (i == shown.lastIndex && more > 0) {
                        Box(Modifier.fillMaxSize().background(Color(0x99000000)), contentAlignment = Alignment.Center) {
                            Text("+$more", color = Color.White, fontSize = 18.sp)
                        }
                    }
                }
            }
        }
    }
    open?.let { start -> PictureViewer(pictures, start) { open = null } }
}

/** Full-screen pictures, swiped left and right; a tap closes it. */
@Composable
fun PictureViewer(pictures: List<FfiPicture>, start: Int, onClose: () -> Unit) {
    val pager = rememberPagerState(initialPage = start) { pictures.size }
    Dialog(onDismissRequest = onClose, properties = DialogProperties(usePlatformDefaultWidth = false)) {
        Box(Modifier.fillMaxSize().background(Color.Black)) {
            HorizontalPager(pager, Modifier.fillMaxSize()) { page ->
                AsyncImage(
                    model = pictures[page].url,
                    contentDescription = null,
                    contentScale = ContentScale.Fit,
                    modifier = Modifier.fillMaxSize().clickable(onClick = onClose),
                )
            }
            if (pictures.size > 1) {
                Text(
                    "${pager.currentPage + 1} / ${pictures.size}",
                    color = Color.White,
                    fontSize = 14.sp,
                    modifier = Modifier.align(Alignment.BottomCenter).padding(24.dp).background(Color(0x8C000000)).padding(horizontal = 10.dp, vertical = 4.dp),
                )
            }
        }
    }
}
