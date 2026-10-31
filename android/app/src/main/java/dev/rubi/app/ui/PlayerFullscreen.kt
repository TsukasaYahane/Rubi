package dev.rubi.app.ui

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.content.pm.ActivityInfo
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

private fun Context.activity(): Activity? = when (this) {
    is Activity -> this
    is ContextWrapper -> baseContext.activity()
    else -> null
}

/** While `on`: landscape, system bars hidden (swipe to peek), screen kept on. Undone on exit. */
@Composable
fun FullscreenMode(on: Boolean) {
    val activity = LocalContext.current.activity() ?: return
    val view = LocalView.current
    DisposableEffect(on) {
        val bars = WindowCompat.getInsetsController(activity.window, view)
        if (on) {
            activity.requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE
            bars.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            bars.hide(WindowInsetsCompat.Type.systemBars())
            view.keepScreenOn = true
        }
        onDispose {
            if (on) {
                activity.requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_PORTRAIT
                bars.show(WindowInsetsCompat.Type.systemBars())
                view.keepScreenOn = false
            }
        }
    }
}

private val MODES = listOf("滚动" to 1.toUByte(), "顶部" to 5.toUByte(), "底部" to 4.toUByte())
private val COLORS = listOf(0xFFFFFFu, 0xFE0302u, 0xFFFF00u, 0x00CD00u, 0x419BF9u, 0xCC0273u)

/**
 * Top bar over the fullscreen player: exit, title, 弹 toggle, 弹幕设置 and the danmaku composer.
 * Shown with the player's own controls; while composing it stays up regardless.
 * `send` posts the text at the current position and returns the new danmaku.
 */
@Composable
fun FullscreenOverlay(
    title: String,
    controlsVisible: Boolean,
    danmakuOn: Boolean,
    onExit: () -> Unit,
    onToggleDanmaku: () -> Unit,
    onSettings: () -> Unit,
    send: suspend (text: String, mode: UByte, color: UInt) -> FfiDanmaku,
    onSent: (FfiDanmaku) -> Unit,
    modifier: Modifier = Modifier,
) {
    val scope = rememberCoroutineScope()
    var composing by remember { mutableStateOf(false) }
    var text by remember { mutableStateOf("") }
    var mode by remember { mutableStateOf(1.toUByte()) }
    var color by remember { mutableStateOf(0xFFFFFFu) }
    var note by remember { mutableStateOf("") }
    var sending by remember { mutableStateOf(false) }
    if (!controlsVisible && !composing) return

    fun submit() {
        if (sending || text.isBlank()) return
        sending = true
        scope.launch {
            runCatching { send(text, mode, color) }
                .onSuccess { onSent(it); text = ""; note = "已发送"; composing = false }
                .onFailure {
                    note = when (it) {
                        is RubiException.NotLoggedIn -> "登录后才能发送弹幕"
                        is RubiException.RiskControl -> "被风控拦截，稍后再试"
                        else -> "发送失败：${it.message}"
                    }
                }
            sending = false
        }
    }

    val label = TextStyle(fontSize = 13.sp, color = Color.White)
    Column(
        modifier.fillMaxWidth()
            .background(Brush.verticalGradient(listOf(Color(0xCC000000), Color.Transparent)))
            .padding(start = 4.dp, end = 16.dp, bottom = 24.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(14.dp)) {
            Box(Modifier.size(44.dp).clickable(onClick = onExit), contentAlignment = Alignment.Center) {
                GlyphIcon(Glyph.Back, color = Color.White)
            }
            Text(title, style = MetroType.body, color = Color.White, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f))
            Text(if (danmakuOn) "弹 开" else "弹 关", style = label, modifier = Modifier.clickable(onClick = onToggleDanmaku).padding(8.dp))
            Text("弹幕设置", style = label, modifier = Modifier.clickable(onClick = onSettings).padding(8.dp))
            Text(
                if (composing) "收起" else "发弹幕",
                style = label.copy(fontWeight = FontWeight.SemiBold),
                modifier = Modifier.clickable { composing = !composing; note = "" }.padding(8.dp),
            )
        }
        if (composing) {
            Column(Modifier.padding(start = 14.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    BasicTextField(
                        value = text,
                        onValueChange = { text = it.take(100) },
                        singleLine = true,
                        textStyle = TextStyle(fontSize = 15.sp, color = Color.White),
                        cursorBrush = SolidColor(Color.White),
                        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
                        keyboardActions = KeyboardActions(onSend = { submit() }),
                        modifier = Modifier.weight(1f).heightIn(min = 40.dp).background(Color(0x66000000)).padding(horizontal = 10.dp, vertical = 10.dp),
                        decorationBox = { inner ->
                            if (text.isEmpty()) Text("发个友善的弹幕见证当下", fontSize = 15.sp, color = Color(0x99FFFFFF))
                            inner()
                        },
                    )
                    AccentButton(if (sending) "发送中" else "发送") { submit() }
                }
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(14.dp)) {
                    MODES.forEach { (name, m) ->
                        Text(
                            name,
                            style = label.copy(fontWeight = if (m == mode) FontWeight.SemiBold else FontWeight.Normal),
                            color = if (m == mode) Color.White else Color(0x99FFFFFF),
                            modifier = Modifier.clickable { mode = m }.padding(vertical = 8.dp),
                        )
                    }
                    COLORS.forEach { rgb ->
                        Box(
                            Modifier.size(22.dp)
                                .background(Color(0xFF000000.toInt() or rgb.toInt()))
                                .then(if (rgb == color) Modifier.border(2.dp, Color.White) else Modifier)
                                .clickable { color = rgb },
                        )
                    }
                    if (note.isNotEmpty()) Text(note, style = label.copy(fontSize = 12.sp))
                }
            }
        } else if (note.isNotEmpty()) {
            Text(note, style = label.copy(fontSize = 12.sp), modifier = Modifier.padding(start = 14.dp))
        }
    }
}
