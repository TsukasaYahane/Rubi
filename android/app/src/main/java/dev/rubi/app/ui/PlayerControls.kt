package dev.rubi.app.ui

import kotlinx.coroutines.withTimeoutOrNull
import kotlinx.coroutines.delay
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.foundation.gestures.waitForUpOrCancellation
import android.media.AudioManager
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.activity.compose.BackHandler
import android.provider.Settings
import android.view.WindowManager
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.awaitTouchSlopOrCancellation
import androidx.compose.foundation.gestures.drag
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.media3.common.Player
import androidx.media3.ui.PlayerView
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiQuality

/**
 * A panel for player settings. Normally a bottom sheet; in fullscreen (`side`) a panel on the
 * right drawn inside the player, because a sheet is its own window and would bring the system
 * bars back.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PlayerSheet(side: Boolean, onDismiss: () -> Unit, content: @Composable () -> Unit) {
    val c = LocalMetro.current
    if (!side) {
        ModalBottomSheet(onDismissRequest = onDismiss, containerColor = c.bg2, shape = RectangleShape, dragHandle = null) { content() }
        return
    }
    BackHandler(onBack = onDismiss)
    Box(
        Modifier.fillMaxSize().background(Color(0x66000000)).pointerInput(Unit) { detectTapGestures { onDismiss() } },
        contentAlignment = Alignment.CenterEnd,
    ) {
        Box(
            Modifier.fillMaxHeight().width(360.dp).background(c.bg2)
                .pointerInput(Unit) { detectTapGestures { } }
                .verticalScroll(rememberScrollState()),
        ) { content() }
    }
}

/** Picks a quality from what the server granted for this video; the current one is heavier. */
@Composable
fun QualitySheet(options: List<FfiQuality>, current: UInt, onDismiss: () -> Unit, side: Boolean = false, onPick: (FfiQuality) -> Unit) {
    val c = LocalMetro.current
    PlayerSheet(side, onDismiss) {
        Column(Modifier.padding(16.dp)) {
            Text("清晰度", style = MetroType.section, color = c.fg)
            options.forEach { q ->
                val on = q.qn == current
                Text(
                    q.label,
                    fontSize = 17.sp,
                    fontWeight = if (on) FontWeight.SemiBold else FontWeight.Light,
                    color = if (on) c.accent else c.fg,
                    modifier = Modifier.fillMaxWidth().heightIn(min = 48.dp).clickable { onPick(q) }.padding(top = 12.dp),
                )
            }
            Meta("会员专属清晰度不提供；切换后作为默认清晰度", Modifier.padding(top = 8.dp, bottom = 12.dp))
        }
    }
}

private enum class Hold { None, Speed, Seek, Brightness, Volume }

/** A drag across the full width moves this far, or the whole video when it is shorter. */
private const val SEEK_SPAN_MS = 90_000L

/**
 * Fullscreen touch layer over the video. What a touch does is decided by what happens first:
 * - it lifts before moving: a tap, showing / hiding the player controls; two taps in quick
 *   succession play or pause instead;
 * - it stays put for the long-press time: a hold, playing at `speed` until released;
 * - it moves first: a drag. Mostly sideways it scrubs (left back, right forward, applied on
 *   release); mostly up / down it sets the window brightness on the left half or the media
 *   volume on the right half; [restore] puts both back when the video page closes.
 * A badge shows the value while it lasts.
 */
@Composable
fun FullscreenGestures(player: Player, view: PlayerView?, speed: Float, restore: GestureRestore) {
    val ctx = LocalContext.current
    val window = remember(ctx) { ctx.activity()?.window }
    val audio = remember(ctx) { ctx.getSystemService(AudioManager::class.java) }
    var hold by remember { mutableStateOf(Hold.None) }
    var badge by remember { mutableStateOf("") }
    // 暂停 / 继续 after a double tap, shown for a moment.
    var flash by remember { mutableStateOf("") }
    LaunchedEffect(flash) {
        if (flash.isNotEmpty()) {
            delay(FLASH_MS)
            flash = ""
        }
    }

    BoxWithConstraints(Modifier.fillMaxSize()) {
        val widthPx = constraints.maxWidth.toFloat().coerceAtLeast(1f)
        val heightPx = constraints.maxHeight.toFloat().coerceAtLeast(1f)
        Box(
            Modifier.fillMaxSize().pointerInput(view, speed) {
                awaitEachGesture {
                    val down = awaitFirstDown(requireUnconsumed = false)
                    var held = true
                    val slop = withTimeoutOrNull(viewConfiguration.longPressTimeoutMillis) {
                        awaitTouchSlopOrCancellation(down.id) { change, _ -> change.consume() }.also { held = false }
                    }
                    when {
                        held -> {
                            // Hold: speed up until every finger is up; movement is ignored.
                            val before = player.playbackParameters.speed
                            player.setPlaybackSpeed(speed)
                            hold = Hold.Speed
                            badge = "${speed}× 播放中"
                            do {
                                val event = awaitPointerEvent()
                                event.changes.forEach { it.consume() }
                            } while (event.changes.any { it.pressed })
                            player.setPlaybackSpeed(before)
                        }
                        slop == null -> {
                            // Lifted before moving: a tap. Another one soon after makes a double
                            // tap, which plays or pauses; a single tap shows the controls.
                            val second = withTimeoutOrNull(viewConfiguration.doubleTapTimeoutMillis) {
                                awaitFirstDown(requireUnconsumed = false)
                            }
                            if (second != null) {
                                val play = !player.playWhenReady
                                if (play) player.play() else player.pause()
                                flash = if (play) "继续播放" else "已暂停"
                                waitForUpOrCancellation()
                            } else {
                                view?.let { v -> if (v.isControllerFullyVisible) v.hideController() else v.showController() }
                            }
                        }
                        else -> {
                            val first = slop.position - down.position
                            hold = when {
                                kotlin.math.abs(first.x) > kotlin.math.abs(first.y) -> Hold.Seek
                                down.position.x < widthPx / 2 -> Hold.Brightness
                                else -> Hold.Volume
                            }
                            val duration = player.duration.coerceAtLeast(0L)
                            val from = player.currentPosition
                            var value = when (hold) {
                                Hold.Brightness -> currentBrightness(ctx, window)
                                Hold.Volume -> audio.getStreamVolume(AudioManager.STREAM_MUSIC).toFloat() /
                                    audio.getStreamMaxVolume(AudioManager.STREAM_MUSIC).coerceAtLeast(1)
                                else -> 0f
                            }
                            var target = from
                            fun apply(totalX: Float, stepY: Float) {
                                when (hold) {
                                    Hold.Seek -> {
                                        val span = if (duration > 0) minOf(SEEK_SPAN_MS, duration) else SEEK_SPAN_MS
                                        val offset = (totalX / widthPx * span).toLong()
                                        target = (from + offset).coerceIn(0L, if (duration > 0) duration else Long.MAX_VALUE)
                                        val sign = if (offset >= 0) "快进 +" else "后退 -"
                                        badge = "$sign${kotlin.math.abs(offset) / 1000} 秒 · ${clock(target)} / ${clock(duration)}"
                                    }
                                    Hold.Brightness -> {
                                        value = (value - stepY / heightPx).coerceIn(0.01f, 1f)
                                        window?.let { w -> w.attributes = w.attributes.apply { screenBrightness = value } }
                                        badge = "亮度 ${(value * 100).toInt()}%"
                                    }
                                    Hold.Volume -> {
                                        value = (value - stepY / heightPx).coerceIn(0f, 1f)
                                        val max = audio.getStreamMaxVolume(AudioManager.STREAM_MUSIC)
                                        if (restore.volumeBefore == null) restore.volumeBefore = audio.getStreamVolume(AudioManager.STREAM_MUSIC)
                                        audio.setStreamVolume(AudioManager.STREAM_MUSIC, (value * max).toInt(), 0)
                                        badge = "音量 ${(value * 100).toInt()}%"
                                    }
                                    else -> {}
                                }
                            }
                            apply(first.x, first.y)
                            drag(slop.id) { change ->
                                change.consume()
                                apply(change.position.x - down.position.x, change.position.y - change.previousPosition.y)
                            }
                            if (hold == Hold.Seek) player.seekTo(target)
                        }
                    }
                    hold = Hold.None
                }
            },
        )
        if (hold != Hold.None || flash.isNotEmpty()) {
            Text(
                if (hold != Hold.None) badge else flash,
                fontSize = 16.sp,
                fontWeight = FontWeight.SemiBold,
                color = Color.White,
                modifier = Modifier.align(Alignment.Center).background(Color(0xB3000000)).padding(horizontal = 16.dp, vertical = 8.dp),
            )
        }
    }
}

/** How long 暂停 / 继续 stays up after a double tap. */
private const val FLASH_MS = 800L

/** `mm:ss`, or `h:mm:ss` from an hour. */
private fun clock(ms: Long): String {
    val s = ms.coerceAtLeast(0L) / 1000
    return if (s >= 3600) "%d:%02d:%02d".format(s / 3600, s / 60 % 60, s % 60) else "%02d:%02d".format(s / 60, s % 60)
}

/**
 * What the fullscreen gestures changed: the window brightness and the media volume. Lives as
 * long as the video page, not the gesture layer (which leaves whenever the controls show), and
 * puts both back when the page closes.
 */
class GestureRestore {
    /** The media volume before the first drag changed it. */
    var volumeBefore: Int? = null
}

@Composable
fun rememberGestureRestore(): GestureRestore {
    val ctx = LocalContext.current
    val restore = remember { GestureRestore() }
    DisposableEffect(ctx) {
        onDispose {
            ctx.activity()?.window?.let { w ->
                w.attributes = w.attributes.apply { screenBrightness = WindowManager.LayoutParams.BRIGHTNESS_OVERRIDE_NONE }
            }
            restore.volumeBefore?.let { ctx.getSystemService(AudioManager::class.java).setStreamVolume(AudioManager.STREAM_MUSIC, it, 0) }
            restore.volumeBefore = null
        }
    }
    return restore
}

/** The window's own brightness, or the system's when the window does not override it. */
internal fun currentBrightness(ctx: android.content.Context, window: android.view.Window?): Float {
    val own = window?.attributes?.screenBrightness ?: -1f
    if (own >= 0f) return own
    return runCatching { Settings.System.getInt(ctx.contentResolver, Settings.System.SCREEN_BRIGHTNESS) / 255f }.getOrDefault(0.5f)
}
