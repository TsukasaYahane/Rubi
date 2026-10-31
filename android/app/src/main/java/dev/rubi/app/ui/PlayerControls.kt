package dev.rubi.app.ui

import android.media.AudioManager
import android.provider.Settings
import android.view.WindowManager
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectDragGesturesAfterLongPress
import androidx.compose.foundation.gestures.detectTapGestures
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

private enum class Hold { None, Speed, Brightness, Volume }

/**
 * Fullscreen touch layer over the video. Tap: show / hide the player controls. Hold: play at
 * `speed` until released. Hold, then drag up / down on the left third: window brightness
 * (restored when fullscreen ends); on the right third: media volume. A value badge shows while
 * holding.
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
        val widthPx = constraints.maxWidth.toFloat()
        val heightPx = constraints.maxHeight.toFloat().coerceAtLeast(1f)
        var start by remember { mutableStateOf(0f) }
        var startX by remember { mutableStateOf(0f) }
        var restoreSpeed by remember { mutableStateOf(1f) }
        Box(
            Modifier.fillMaxSize()
                .pointerInput(view) {
                    detectTapGestures(onTap = {
                        val v = view ?: return@detectTapGestures
                        if (v.isControllerFullyVisible) v.hideController() else v.showController()
                    })
                }
                .pointerInput(speed) {
                    detectDragGesturesAfterLongPress(
                        onDragStart = { at ->
                            startX = at.x
                            restoreSpeed = player.playbackParameters.speed
                            player.setPlaybackSpeed(speed)
                            hold = Hold.Speed
                            badge = "${speed}× 播放中"
                        },
                        onDrag = { change, _ ->
                            change.consume()
                            if (hold == Hold.Speed && change.position.y != change.previousPosition.y) {
                                // A vertical drag on a side third turns the hold into a slider.
                                val side = when {
                                    startX < widthPx / 3 -> Hold.Brightness
                                    startX > widthPx * 2 / 3 -> Hold.Volume
                                    else -> Hold.Speed
                                }
                                if (side != Hold.Speed) {
                                    player.setPlaybackSpeed(restoreSpeed)
                                    hold = side
                                    start = when (side) {
                                        Hold.Brightness -> currentBrightness(ctx, window)
                                        else -> audio.getStreamVolume(AudioManager.STREAM_MUSIC).toFloat() /
                                            audio.getStreamMaxVolume(AudioManager.STREAM_MUSIC).coerceAtLeast(1)
                                    }
                                }
                            }
                            val delta = -(change.position.y - change.previousPosition.y) / heightPx
                            when (hold) {
                                Hold.Brightness -> {
                                    start = (start + delta).coerceIn(0.01f, 1f)
                                    window?.let { w -> w.attributes = w.attributes.apply { screenBrightness = start } }
                                    badge = "亮度 ${(start * 100).toInt()}%"
                                }
                                Hold.Volume -> {
                                    start = (start + delta).coerceIn(0f, 1f)
                                    val max = audio.getStreamMaxVolume(AudioManager.STREAM_MUSIC)
                                    audio.setStreamVolume(AudioManager.STREAM_MUSIC, (start * max).toInt(), 0)
                                    badge = "音量 ${(start * 100).toInt()}%"
                                }
                                else -> {}
                            }
                        },
                        onDragEnd = {
                            if (hold == Hold.Speed) player.setPlaybackSpeed(restoreSpeed)
                            hold = Hold.None
                        },
                        onDragCancel = {
                            if (hold == Hold.Speed) player.setPlaybackSpeed(restoreSpeed)
                            hold = Hold.None
                        },
                    )
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

/** The window's own brightness, or the system's when the window does not override it. */
private fun currentBrightness(ctx: android.content.Context, window: android.view.Window?): Float {
    val own = window?.attributes?.screenBrightness ?: -1f
    if (own >= 0f) return own
    return runCatching { Settings.System.getInt(ctx.contentResolver, Settings.System.SCREEN_BRIGHTNESS) / 255f }.getOrDefault(0.5f)
}
