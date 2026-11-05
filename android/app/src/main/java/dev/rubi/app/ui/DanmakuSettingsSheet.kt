package dev.rubi.app.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.rubi.app.data.DanmakuDisplay
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType

/**
 * The player's settings: 播放设置 (when `play` is given) and 弹幕设置 (unless `danmaku` is off).
 * The video page shows both under 设置; fullscreen has a button for each. Changes apply at once
 * and are saved by the caller.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DanmakuSettingsSheet(
    display: DanmakuDisplay,
    onChange: (DanmakuDisplay) -> Unit,
    onDismiss: () -> Unit,
    side: Boolean = false,
    play: PlayControls? = null,
    danmaku: Boolean = true,
) {
    val c = LocalMetro.current
    PlayerSheet(side, onDismiss) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            play?.let { PlaySettings(it) }
            if (!danmaku) {
                Box(Modifier.padding(bottom = 8.dp))
                return@Column
            }
            Text("弹幕设置", style = MetroType.section, color = c.fg, modifier = Modifier.padding(top = if (play != null) 10.dp else 0.dp))

            Choice(
                "显示区域",
                listOf("1/4" to 0.25f, "半屏" to 0.5f, "3/4" to 0.75f, "满屏" to 1f),
                display.area,
            ) { onChange(display.copy(area = it)) }
            Choice(
                "滚动速度",
                listOf("慢" to 0.7f, "适中" to 1f, "快" to 1.4f, "极快" to 1.8f),
                display.speed,
            ) { onChange(display.copy(speed = it)) }

            Percent("字号", display.fontScale, 0.5f..2f) { onChange(display.copy(fontScale = it)) }
            Percent("不透明度", display.opacity, 0.2f..1f) { onChange(display.copy(opacity = it)) }

            Text("显示类型", fontSize = 14.sp, color = c.fg2, modifier = Modifier.padding(top = 8.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(20.dp)) {
                Flag("滚动", display.showScroll) { onChange(display.copy(showScroll = it)) }
                Flag("顶部", display.showTop) { onChange(display.copy(showTop = it)) }
                Flag("底部", display.showBottom) { onChange(display.copy(showBottom = it)) }
                Flag("彩色", display.showColored) { onChange(display.copy(showColored = it)) }
            }
            Meta("关键词 / 互动弹幕 / 空降合并等过滤在「我的 → 设置」中", Modifier.padding(top = 8.dp, bottom = 12.dp))
        }
    }
}

/** A row of mutually exclusive text options; the selected one is heavier. */
/** 倍速 offered in 播放设置. */
private val SPEEDS = listOf("0.5" to 0.5f, "0.75" to 0.75f, "1.0" to 1f, "1.25" to 1.25f, "1.5" to 1.5f, "2.0" to 2f)

/** 播放设置: the speed and how the picture fills its frame, with what changes them. */
class PlayControls(
    val speed: Float,
    val onSpeed: (Float) -> Unit,
    val scale: VideoScale,
    val onScale: (VideoScale) -> Unit,
)

/** 倍速 and 画面比例 (适应 / 拉伸 / 填充); pinching the picture zooms it further. */
@Composable
private fun PlaySettings(play: PlayControls) {
    val c = LocalMetro.current
    Text("播放设置", style = MetroType.section, color = c.fg)
    Choice("倍速", SPEEDS, SPEEDS.minBy { kotlin.math.abs(it.second - play.speed) }.second, play.onSpeed)
    Choice(
        "画面比例",
        VideoScale.entries.map { it.label to it.ordinal.toFloat() },
        play.scale.ordinal.toFloat(),
    ) { play.onScale(VideoScale.entries[it.toInt()]) }
    Meta("双指可自由缩放、拖动画面，点「复原」恢复")
}

@Composable
private fun Choice(label: String, options: List<Pair<String, Float>>, value: Float, onPick: (Float) -> Unit) {
    val c = LocalMetro.current
    Row(Modifier.fillMaxWidth().heightIn(min = 44.dp), verticalAlignment = Alignment.CenterVertically) {
        Text(label, fontSize = 14.sp, color = c.fg2, modifier = Modifier.width(80.dp))
        Row(horizontalArrangement = Arrangement.spacedBy(18.dp)) {
            options.forEach { (text, v) ->
                val on = kotlin.math.abs(v - value) < 0.01f
                Text(
                    text,
                    fontSize = 16.sp,
                    color = if (on) c.fg else c.fg2,
                    fontWeight = if (on) FontWeight.SemiBold else FontWeight.Normal,
                    modifier = Modifier.heightIn(min = 44.dp).clickable { onPick(v) }.padding(top = 10.dp),
                )
            }
        }
    }
}

/** Slider shown as a percentage; saves when the drag ends. */
@Composable
private fun Percent(label: String, value: Float, range: ClosedFloatingPointRange<Float>, onDone: (Float) -> Unit) {
    val c = LocalMetro.current
    var v by remember(value) { mutableFloatStateOf(value) }
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        Text(label, fontSize = 14.sp, color = c.fg2, modifier = Modifier.width(80.dp))
        Slider(
            value = v,
            onValueChange = { v = it },
            onValueChangeFinished = { onDone(v) },
            valueRange = range,
            colors = SliderDefaults.colors(thumbColor = c.accent, activeTrackColor = c.accent, inactiveTrackColor = c.line),
            modifier = Modifier.weight(1f),
        )
        Meta("${(v * 100).toInt()}%", Modifier.width(44.dp), c.fg)
    }
}

@Composable
private fun Flag(label: String, on: Boolean, onChange: (Boolean) -> Unit) {
    val c = LocalMetro.current
    Row(
        Modifier.heightIn(min = 44.dp).clickable { onChange(!on) },
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        Text(if (on) "■" else "□", fontSize = 14.sp, color = if (on) c.accent else c.fg2)
        Text(label, fontSize = 15.sp, color = if (on) c.fg else c.fg2)
    }
}
