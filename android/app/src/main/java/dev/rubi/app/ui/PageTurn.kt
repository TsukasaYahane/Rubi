package dev.rubi.app.ui

import android.app.Activity
import android.graphics.Bitmap
import android.os.Handler
import android.os.Looper
import android.view.PixelCopy
import androidx.compose.animation.core.animate
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInWindow

/** How long a whole page takes to turn, at 标准. */
private const val TURN_MS = 520

/**
 * 书页 page turns, always hinged on the left edge of the screen: moving on (a page opened, a
 * pivot to the right) the old page turns up and away over the new one; moving back (going back,
 * a pivot to the left) the new page turns back down over the old one.
 *
 * The old page is gone once the change is made, so the screen is copied first ([switch] makes
 * the change only once the copy is in, about a frame later).
 */
class PageTurns {
    /** The copy of the screen before the change, while a turn runs. */
    var shot by mutableStateOf<ImageBitmap?>(null)
        private set
    /** Moving on (the old page turns up) or back (the new page turns down). */
    var forward by mutableStateOf(true)
        private set
    var round by mutableIntStateOf(0)
        private set
    /** How far the turn has gone, 0 to 1; reset with the copy so no frame shows the end state. */
    var turn by mutableFloatStateOf(0f)
        internal set

    /** Copies the screen; null when no turn should run (another style, or no window). */
    internal var capture: ((done: (Bitmap?) -> Unit) -> Unit)? = null

    /**
     * Makes `change` with a page turn: the screen is copied, then `change` runs and the turn
     * starts. Without a turn to run `change` runs at once.
     */
    fun switch(forward: Boolean, change: () -> Unit) {
        val copy = capture
        if (copy == null) {
            change()
            return
        }
        copy { bmp ->
            if (bmp != null) {
                turn = 0f
                shot = bmp.asImageBitmap()
                this.forward = forward
                round++
            }
            change()
        }
    }

    internal fun done() {
        shot = null
    }
}

val LocalPageTurns = staticCompositionLocalOf { PageTurns() }

/** Copies what the window shows now (videos included) into a bitmap, then calls `done`. */
fun copyWindow(activity: Activity, done: (Bitmap?) -> Unit) {
    val view = activity.window?.decorView
    if (view == null || view.width == 0 || view.height == 0) {
        done(null)
        return
    }
    val bmp = runCatching { Bitmap.createBitmap(view.width, view.height, Bitmap.Config.ARGB_8888) }.getOrNull()
    if (bmp == null) {
        done(null)
        return
    }
    runCatching {
        PixelCopy.request(activity.window, bmp, { result -> done(if (result == PixelCopy.SUCCESS) bmp else null) }, Handler(Looper.getMainLooper()))
    }.onFailure { done(null) }
}

/** Runs each turn to its end, then lets the copy go. */
@Composable
fun PageTurnClock(turns: PageTurns, motion: Motion) {
    LaunchedEffect(turns.round) {
        if (turns.shot == null) return@LaunchedEffect
        runFixed {
            animate(0f, 1f, animationSpec = tween(motion.ms(TURN_MS), easing = TurnEasing)) { v, _ -> turns.turn = v }
        }
        turns.done()
    }
}

/**
 * The copy of the old page. Moving back it lies still under the new page (`under`); moving on it
 * turns up about the left edge over the new page.
 */
@Composable
fun PageTurnShot(turns: PageTurns, under: Boolean) {
    val shot = turns.shot ?: return
    if (turns.forward == under) return
    var at by remember { mutableStateOf(Offset.Zero) }
    Canvas(
        Modifier.fillMaxSize()
            .onGloballyPositioned { at = it.positionInWindow() }
            .graphicsLayer {
                if (!under) {
                    rotationY = -90f * turns.turn
                    transformOrigin = TransformOrigin(0f, 0.5f)
                    cameraDistance = 14f * density
                }
            },
    ) {
        drawImage(shot, topLeft = Offset(-at.x, -at.y))
    }
}

/** Moving back, the new page turns down about the left edge onto the old one. */
fun Modifier.pageTurnIn(turns: PageTurns): Modifier = graphicsLayer {
    if (turns.shot != null && !turns.forward) {
        rotationY = -90f * (1f - turns.turn)
        transformOrigin = TransformOrigin(0f, 0.5f)
        cameraDistance = 14f * density
    }
}
