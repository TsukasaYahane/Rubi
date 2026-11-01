package dev.rubi.app.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
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
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import coil3.compose.AsyncImage
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.core.FfiPicture

/** Images attached to a comment: one keeps its shape, several become a grid of squares. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun CommentPictures(pictures: List<FfiPicture>) {
    val c = LocalMetro.current
    var open by remember { mutableStateOf<FfiPicture?>(null) }
    FlowRow(Modifier.padding(top = 6.dp), horizontalArrangement = Arrangement.spacedBy(6.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
        pictures.forEach { p ->
            val single = pictures.size == 1 && p.width > 0u && p.height > 0u
            val shape = if (single) {
                Modifier.widthIn(max = 220.dp).aspectRatio((p.width.toFloat() / p.height.toFloat()).coerceIn(0.5f, 2f))
            } else {
                Modifier.size(96.dp)
            }
            AsyncImage(
                model = p.url,
                contentDescription = null,
                contentScale = ContentScale.Crop,
                modifier = shape.background(c.bg2).clickable { open = p },
            )
        }
    }
    open?.let { p ->
        Dialog(onDismissRequest = { open = null }, properties = DialogProperties(usePlatformDefaultWidth = false)) {
            Box(Modifier.fillMaxSize().background(Color.Black).clickable { open = null }, contentAlignment = Alignment.Center) {
                AsyncImage(model = p.url, contentDescription = null, contentScale = ContentScale.Fit, modifier = Modifier.fillMaxSize())
            }
        }
    }
}
