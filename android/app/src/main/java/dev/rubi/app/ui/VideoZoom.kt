package dev.rubi.app.ui

import androidx.annotation.OptIn
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.calculatePan
import androidx.compose.foundation.gestures.calculateZoom
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.media3.common.util.UnstableApi
import androidx.media3.ui.AspectRatioFrameLayout

/** 画面: how the video fills its frame. */
enum class VideoScale(val label: String) {
    /** The whole picture, black bars where the shapes differ. */
    Fit("适应"),
    /** The whole picture pulled to the frame's shape. */
    Stretch("拉伸"),
    /** The frame filled, the picture's edges cut off. */
    Fill("填充"),
}

@OptIn(UnstableApi::class)
fun VideoScale.resizeMode(): Int = when (this) {
    VideoScale.Fit -> AspectRatioFrameLayout.RESIZE_MODE_FIT
    VideoScale.Stretch -> AspectRatioFrameLayout.RESIZE_MODE_FILL
    VideoScale.Fill -> AspectRatioFrameLayout.RESIZE_MODE_ZOOM
}

/** Two-finger zoom and move of the picture, on top of [VideoScale]; [reset] puts it back. */
class VideoZoom {
    var scale by mutableFloatStateOf(1f)
        private set
    var offset by mutableStateOf(Offset.Zero)
        private set
    val zoomed: Boolean get() = scale != 1f || offset != Offset.Zero

    fun reset() {
        scale = 1f
        offset = Offset.Zero
    }

    /** One step of a pinch: scaled by `zoom`, moved by `pan`, kept at least half on screen. */
    internal fun step(zoom: Float, pan: Offset, width: Float, height: Float) {
        scale = (scale * zoom).coerceIn(MIN_SCALE, MAX_SCALE)
        val limitX = width * scale / 2
        val limitY = height * scale / 2
        val moved = offset + pan
        offset = Offset(moved.x.coerceIn(-limitX, limitX), moved.y.coerceIn(-limitY, limitY))
    }

    private companion object {
        const val MIN_SCALE = 0.5f
        const val MAX_SCALE = 4f
    }
}

/**
 * Pinching with two fingers zooms and moves the picture. One finger is left alone (the player's
 * own taps and the swipe gestures); a second finger takes the touch over until both lift.
 */
fun Modifier.pinchToZoom(zoom: VideoZoom): Modifier = pointerInput(zoom) {
    awaitEachGesture {
        awaitFirstDown(requireUnconsumed = false, pass = PointerEventPass.Initial)
        do {
            val event = awaitPointerEvent(PointerEventPass.Initial)
            if (event.changes.count { it.pressed } >= 2) {
                zoom.step(event.calculateZoom(), event.calculatePan(), size.width.toFloat(), size.height.toFloat())
                event.changes.forEach { it.consume() }
            }
        } while (event.changes.any { it.pressed })
    }
}

/** Draws the picture as [VideoZoom] has it. */
fun Modifier.zoomedBy(zoom: VideoZoom): Modifier = graphicsLayer {
    scaleX = zoom.scale
    scaleY = zoom.scale
    translationX = zoom.offset.x
    translationY = zoom.offset.y
}

/** 复原: shown while the picture is zoomed or moved. */
@Composable
fun ZoomReset(zoom: VideoZoom, modifier: Modifier = Modifier) {
    if (!zoom.zoomed) return
    Text(
        "复原",
        fontSize = 12.sp,
        color = Color.White,
        modifier = modifier.padding(8.dp).background(Color(0x99000000)).clickable { zoom.reset() }.padding(horizontal = 10.dp, vertical = 5.dp),
    )
}
