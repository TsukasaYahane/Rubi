package dev.rubi.app.ui

import android.media.AudioManager
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

/** Picks a quality from what the server granted for this video; the current one is heavier. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun QualitySheet(options: List<FfiQuality>, current: UInt, onDismiss: () -> Unit, onPick: (FfiQuality) -> Unit) {
    val c = LocalMetro.current
    ModalBottomSheet(onDismissRequest = onDismiss, containerColor = c.bg2, shape = RectangleShape, dragHandle = null) {
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
 * - it lifts before moving: a tap, showing / hiding the player controls;
 * - it stays put for the long-press time: a hold, playing at `speed` until released;
 * - it moves first: a drag. Mostly sideways it scrubs (left back, right forward, applied on
 *   release); mostly up / down it sets the window brightness on the left half (restored when
 *   fullscreen ends) or the media volume on the right half.
 * A badge shows the value while it lasts.
 */
@Composable
fun FullscreenGestures(player: Player, view: PlayerView?, speed: Float) {
    val ctx = LocalContext.current
    val window = remember(ctx) { ctx.activity()?.window }
    val audio = remember(ctx) { ctx.getSystemService(AudioManager::class.java) }
    var hold by remember { mutableStateOf(Hold.None) }
    var badge by remember { mutableStateOf("") }

    // Brightness is changed for this window only and handed back when fullscreen ends.
    DisposableEffect(window) {
        onDispose {
            window?.let { w ->
                w.attributes = w.attributes.apply { screenBrightness = WindowManager.LayoutParams.BRIGHTNESS_OVERRIDE_NONE }
            }
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
                            // Lifted before moving: a tap.
                            view?.let { v -> if (v.isControllerFullyVisible) v.hideController() else v.showController() }
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
        if (hold != Hold.None) {
            Text(
                badge,
                fontSize = 16.sp,
                fontWeight = FontWeight.SemiBold,
                color = Color.White,
                modifier = Modifier.align(Alignment.Center).background(Color(0xB3000000)).padding(horizontal = 16.dp, vertical = 8.dp),
            )
        }
    }
}

/** `mm:ss`, or `h:mm:ss` from an hour. */
private fun clock(ms: Long): String {
    val s = ms.coerceAtLeast(0L) / 1000
    return if (s >= 3600) "%d:%02d:%02d".format(s / 3600, s / 60 % 60, s % 60) else "%02d:%02d".format(s / 60, s % 60)
}

/** The window's own brightness, or the system's when the window does not override it. */
private fun currentBrightness(ctx: android.content.Context, window: android.view.Window?): Float {
    val own = window?.attributes?.screenBrightness ?: -1f
    if (own >= 0f) return own
    return runCatching { Settings.System.getInt(ctx.contentResolver, Settings.System.SCREEN_BRIGHTNESS) / 255f }.getOrDefault(0.5f)
}
