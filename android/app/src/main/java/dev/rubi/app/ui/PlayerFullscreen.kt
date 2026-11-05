package dev.rubi.app.ui

import dev.rubi.app.playback.LocalMedia
import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.content.pm.ActivityInfo
import android.os.Build
import android.view.WindowManager
import dev.rubi.app.ui.theme.LocalMetro
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.runtime.MutableState
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.draw.drawBehind
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiDanmaku
import dev.rubi.core.RubiException
import kotlinx.coroutines.launch

internal fun Context.activity(): Activity? = when (this) {
    is Activity -> this
    is ContextWrapper -> baseContext.activity()
    else -> null
}

/**
 * While `on`: landscape, system bars hidden (swipe to peek), drawing into the cutout area, screen
 * kept on. Undone on exit.
 */
@Composable
fun FullscreenMode(on: Boolean) {
    val media = LocalMedia.current
    val activity = LocalContext.current.activity() ?: return
    val view = LocalView.current
    DisposableEffect(on) {
        val window = activity.window
        val bars = WindowCompat.getInsetsController(window, view)
        val cutoutBefore = if (Build.VERSION.SDK_INT >= 28) window.attributes.layoutInDisplayCutoutMode else 0
        if (on) {
            activity.requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE
            bars.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            bars.hide(WindowInsetsCompat.Type.systemBars())
            if (Build.VERSION.SDK_INT >= 28) {
                window.attributes = window.attributes.apply {
                    layoutInDisplayCutoutMode = WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES
                }
            }
            view.keepScreenOn = true
            media.fullscreen.value = true
        }
        onDispose {
            if (on) {
                // Back to what the device allows: upright on a phone, free on a tablet.
                activity.applyDeviceOrientation()
                bars.show(WindowInsetsCompat.Type.systemBars())
                if (Build.VERSION.SDK_INT >= 28) {
                    window.attributes = window.attributes.apply { layoutInDisplayCutoutMode = cutoutBefore }
                }
                view.keepScreenOn = false
                media.fullscreen.value = false
            }
        }
    }
}

private val MODES = listOf("滚动" to 1.toUByte(), "顶部" to 5.toUByte(), "底部" to 4.toUByte())
private val COLORS = listOf(0xFFFFFFu, 0xFE0302u, 0xFFFF00u, 0x00CD00u, 0x419BF9u, 0xCC0273u)

/** Why a danmaku could not be sent, in words. */
internal fun danmakuSendError(e: Throwable): String = when (e) {
    is RubiException.NotLoggedIn -> "登录后才能发送弹幕"
    is RubiException.RiskControl -> "被风控拦截，稍后再试"
    else -> "发送失败：${e.message}"
}

/**
 * 发弹幕 as part of the picture: light text, an underlined field, 样式 in a thin frame and a
 * send arrow, no filled blocks. `strip` lays it on the black strip under the video (the video
 * page); without it the bar sits over the picture, on whatever shade the caller puts behind
 * (the 竖屏 feed, fullscreen). 样式 unfolds the modes and colours.
 */
@Composable
fun DanmakuSendBar(
    send: suspend (text: String, mode: UByte, color: UInt) -> FfiDanmaku,
    onSent: (FfiDanmaku) -> Unit,
    strip: Boolean = true,
    /** What is typed, when it must outlive the bar (fullscreen closes and reopens it). */
    draft: MutableState<String>? = null,
) {
    val c = LocalMetro.current
    val dim = Color(0x99FFFFFF)
    val scope = rememberCoroutineScope()
    val own = remember { mutableStateOf("") }
    var text by (draft ?: own)
    var mode by remember { mutableStateOf(1.toUByte()) }
    var color by remember { mutableStateOf(0xFFFFFFu) }
    var styling by remember { mutableStateOf(false) }
    var note by remember { mutableStateOf("") }
    var sending by remember { mutableStateOf(false) }
    fun submit() {
        if (sending || text.isBlank()) return
        sending = true
        scope.launch {
            runCatching { send(text, mode, color) }
                .onSuccess { onSent(it); text = ""; note = "已发送" }
                .onFailure { note = danmakuSendError(it) }
            sending = false
        }
    }
    Column(
        Modifier.fillMaxWidth().then(if (strip) Modifier.background(STRIP) else Modifier).padding(horizontal = 12.dp, vertical = 6.dp),
        verticalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            BasicTextField(
                value = text,
                onValueChange = { text = it.take(100); note = "" },
                singleLine = true,
                textStyle = TextStyle(fontSize = 14.sp, color = Color.White),
                cursorBrush = SolidColor(Color.White),
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
                keyboardActions = KeyboardActions(onSend = { submit() }),
                // Only a line under the text: the field belongs to the picture, not on top of it.
                modifier = Modifier.weight(1f).heightIn(min = 32.dp)
                    .drawBehind { drawLine(Color(0x66FFFFFF), Offset(0f, size.height), Offset(size.width, size.height), 1.dp.toPx()) }
                    .padding(vertical = 7.dp),
                decorationBox = { inner ->
                    if (text.isEmpty()) Text(if (sending) "发送中…" else "发个友善的弹幕…", fontSize = 14.sp, color = dim)
                    inner()
                },
            )
            Text(
                if (styling) "收起" else "样式",
                fontSize = 12.sp,
                color = Color.White,
                modifier = Modifier.border(1.dp, Color(0x80FFFFFF)).clickable { styling = !styling }.padding(horizontal = 7.dp, vertical = 3.dp),
            )
            Box(Modifier.size(32.dp).clickable { submit() }, contentAlignment = Alignment.Center) {
                LineIconView(LineIcon.Send, if (text.isBlank()) dim else Color.White, 22.dp)
            }
        }
        if (styling) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(14.dp)) {
                MODES.forEach { (name, m) ->
                    Text(
                        name,
                        fontSize = 13.sp,
                        fontWeight = if (m == mode) FontWeight.SemiBold else FontWeight.Normal,
                        color = if (m == mode) Color.White else dim,
                        modifier = Modifier.clickable { mode = m }.padding(vertical = 6.dp),
                    )
                }
                COLORS.forEach { rgb ->
                    Box(
                        Modifier.size(20.dp)
                            .background(Color(0xFF000000.toInt() or rgb.toInt()))
                            .border(if (rgb == color) 2.dp else 1.dp, if (rgb == color) c.accent else Color(0x66FFFFFF))
                            .clickable { color = rgb },
                    )
                }
            }
        }
        if (note.isNotEmpty()) Text(note, fontSize = 12.sp, color = dim)
    }
}

/** The strip under the video that 发弹幕 sits on: the picture's own black, just lifted. */
private val STRIP = Color(0xFF111111)

/**
 * Over the fullscreen player: the top bar (exit, title, 弹 toggle, 弹幕设置, 清晰度, 发弹幕) shown
 * with the player's own controls, and while composing the 发弹幕 bar at the bottom of the
 * picture, which stays up regardless. `send` posts the text at the current position.
 */
@Composable
fun FullscreenOverlay(
    title: String,
    controlsVisible: Boolean,
    danmakuOn: Boolean,
    onExit: () -> Unit,
    onToggleDanmaku: () -> Unit,
    onSettings: () -> Unit,
    onPlaySettings: () -> Unit,
    quality: String,
    onQuality: () -> Unit,
    send: suspend (text: String, mode: UByte, color: UInt) -> FfiDanmaku,
    onSent: (FfiDanmaku) -> Unit,
    modifier: Modifier = Modifier,
) {
    var composing by remember { mutableStateOf(false) }
    // Kept while the bar is closed: a tap on the picture closes it, the text waits for its return.
    val draft = remember { mutableStateOf("") }
    if (!controlsVisible && !composing) return

    val label = TextStyle(fontSize = 13.sp, color = Color.White)
    Box(modifier.fillMaxSize()) {
        // While composing, a tap anywhere on the picture closes the bar (and nothing else).
        if (composing) Box(Modifier.fillMaxSize().pointerInput(Unit) { detectTapGestures { composing = false } })
        if (controlsVisible || composing) {
            Row(
                Modifier.fillMaxWidth()
                    .background(Brush.verticalGradient(listOf(Color(0xCC000000), Color.Transparent)))
                    .padding(start = 4.dp, end = 16.dp, bottom = 24.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(14.dp),
            ) {
                Box(Modifier.size(44.dp).clickable(onClick = onExit), contentAlignment = Alignment.Center) {
                    GlyphIcon(Glyph.Back, color = Color.White)
                }
                Text(title, style = MetroType.body, color = Color.White, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f))
                Text(if (danmakuOn) "弹 开" else "弹 关", style = label, modifier = Modifier.clickable(onClick = onToggleDanmaku).padding(8.dp))
                Text("播放设置", style = label, modifier = Modifier.clickable(onClick = onPlaySettings).padding(8.dp))
                Text("弹幕设置", style = label, modifier = Modifier.clickable(onClick = onSettings).padding(8.dp))
                if (quality.isNotEmpty()) Text(quality, style = label, modifier = Modifier.clickable(onClick = onQuality).padding(8.dp))
                Text(
                    if (composing) "收起" else "发弹幕",
                    style = label.copy(fontWeight = FontWeight.SemiBold),
                    modifier = Modifier.clickable { composing = !composing }.padding(8.dp),
                )
            }
        }
        if (composing) {
            // Above the player's own seek bar while that shows.
            Box(
                Modifier.align(Alignment.BottomCenter).fillMaxWidth()
                    .background(Brush.verticalGradient(listOf(Color.Transparent, Color(0xB3000000))))
                    .padding(top = 24.dp, bottom = if (controlsVisible) 56.dp else 8.dp, start = 32.dp, end = 32.dp),
            ) {
                DanmakuSendBar(send, onSent = { onSent(it); composing = false }, strip = false, draft = draft)
            }
        }
    }
}
