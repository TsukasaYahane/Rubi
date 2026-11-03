package dev.rubi.app.ui

import androidx.compose.ui.graphics.Color
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.border
import androidx.compose.foundation.background
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.rubi.app.RubiApp
import dev.rubi.app.data.PlaybackPrefs
import dev.rubi.app.ui.theme.LocalMetro
import org.json.JSONArray
import org.json.JSONObject

private val FILTER_TOGGLES = listOf(
    Triple("promotion", "必火推广", "隐藏付费加推的视频"),
    Triple("ad", "广告卡片", "推荐流、列表、搜索中的广告"),
    Triple("banner", "首页横幅与活动页", ""),
    Triple("goods", "视频页商品卡片", ""),
    Triple("pinned_comment", "置顶带货评论", ""),
    Triple("commercial_penalty", "商业合作稿件降权", "默认关闭 · 不惩罚 UP 主商单"),
    Triple("notice", "营销私信与推广通知", ""),
    Triple("watched", "推荐中隐藏已看过", ""),
)

private val DANMAKU_TOGGLES = listOf(
    Triple("command_danmaku", "过滤互动指令弹幕", "投票 / 关注 / 三连 / 链接卡片"),
    Triple("show_advanced_danmaku", "显示高级弹幕", "mode 7 定位弹幕"),
    Triple("merge_jump", "合并空降弹幕", "±5s 内只保留最早一条"),
)

private val FACTORS = listOf(
    "interest" to "兴趣匹配", "follow" to "关注 UP", "freshness" to "新鲜度", "topic" to "话题度",
    "quality" to "内容质量", "explore" to "探索", "dislike" to "不感兴趣", "repeat_exposure" to "重复曝光",
)

/**
 * Edits the settings JSON owned by the Rust core. Every change is written back
 * immediately; the core validates it (bad regexes are rejected).
 */
@Composable
fun SettingsScreen(app: RubiApp, nav: Nav) {
    val c = LocalMetro.current
    val settings = remember { runCatching { JSONObject(app.core.settingsJson()) }.getOrElse { JSONObject() } }
    var version by remember { mutableIntStateOf(0) }
    var note by remember { mutableStateOf("") }

    fun save() {
        note = runCatching { app.core.setSettingsJson(settings.toString()); "" }.getOrElse { "未保存：${it.message}" }
        version++
    }

    val rules = settings.optJSONObject("rules") ?: JSONObject().also { settings.put("rules", it) }
    val toggles = rules.optJSONObject("toggles") ?: JSONObject().also { rules.put("toggles", it) }
    val weights = settings.optJSONObject("weights") ?: JSONObject().also { settings.put("weights", it) }
    val keywords = rules.optJSONArray("keywords") ?: JSONArray().also { rules.put("keywords", it) }

    Column(Modifier.fillMaxSize()) {
        PageHeader(nav, "设置")
        if (note.isNotEmpty()) Meta(note, Modifier.padding(horizontal = 16.dp))
        // `version` forces recomposition after in-place JSON edits.
        LazyColumn(Modifier.weight(1f)) {
            item(key = "playback-$version") {
                SectionTitle("播放")
                ToggleRow("后台播放", "切到后台后继续播放；关闭时自动暂停", settings.optBoolean("background_play", false)) {
                    settings.put("background_play", it)
                    save()
                }
                ToggleRow("切到后台自动小窗", "在播放页回到桌面时以小窗继续播放；关闭小窗后按「后台播放」决定继续或暂停", settings.optBoolean("auto_pip", false)) {
                    settings.put("auto_pip", it)
                    save()
                }
                val speed = settings.optDouble("long_press_speed", 1.5).toFloat()
                Column(Modifier.padding(horizontal = 16.dp, vertical = 8.dp)) {
                    Text("长按倍速", fontSize = 16.sp, color = c.fg)
                    Meta("全屏时按住画面临时倍速播放，松手恢复")
                    Row(horizontalArrangement = Arrangement.spacedBy(20.dp)) {
                        PlaybackPrefs.SPEEDS.forEach { v ->
                            val on = kotlin.math.abs(v - speed) < 0.01f
                            Text(
                                "${v}×",
                                fontSize = 16.sp,
                                color = if (on) c.fg else c.fg2,
                                fontWeight = if (on) FontWeight.SemiBold else FontWeight.Normal,
                                modifier = Modifier.heightIn(min = 44.dp).clickable { settings.put("long_press_speed", v.toDouble()); save() }.padding(top = 12.dp),
                            )
                        }
                    }
                    Meta("默认清晰度：在播放页右上角切换后记住")
                }
            }
            item(key = "preview-$version") {
                val preview = settings.optJSONObject("preview") ?: JSONObject().also { settings.put("preview", it) }
                SectionTitle("大图预览")
                Meta("首页单列模式下，停在屏幕中间的视频自动播放预览（360P，循环）", Modifier.padding(horizontal = 16.dp))
                ToggleRow("启用预览", "关闭后只显示封面", preview.optBoolean("enabled", true)) { preview.put("enabled", it); save() }
                ToggleRow("仅 Wi-Fi", "使用移动数据时不预览", preview.optBoolean("wifi_only", true)) { preview.put("wifi_only", it); save() }
                ToggleRow("预览弹幕", "在预览画面上显示弹幕", preview.optBoolean("danmaku", false)) { preview.put("danmaku", it); save() }
                var volume by remember(version) { mutableStateOf(preview.optDouble("volume", 0.0).toFloat()) }
                Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically) {
                    Text("预览音量", fontSize = 14.sp, color = c.fg, modifier = Modifier.width(72.dp))
                    Slider(
                        value = volume,
                        onValueChange = { volume = it },
                        onValueChangeFinished = { preview.put("volume", volume.toDouble()); save() },
                        valueRange = 0f..1f,
                        colors = SliderDefaults.colors(thumbColor = c.accent, activeTrackColor = c.accent, inactiveTrackColor = c.line),
                        modifier = Modifier.weight(1f),
                    )
                    Meta(if (volume < 0.01f) "静音" else "${(volume * 100).toInt()}%", Modifier.width(40.dp), c.fg)
                }
            }
            item(key = "filters-$version") {
                SectionTitle("内容过滤")
                FILTER_TOGGLES.forEach { (key, title, desc) ->
                    ToggleRow(title, desc, toggles.optBoolean(key)) { toggles.put(key, it); save() }
                }
                SectionTitle("弹幕")
                DANMAKU_TOGGLES.forEach { (key, title, desc) ->
                    ToggleRow(title, desc, toggles.optBoolean(key)) { toggles.put(key, it); save() }
                }
            }
            item(key = "keywords-$version") {
                SectionTitle("屏蔽关键词")
                Meta("作用于标题、标签和简介", Modifier.padding(horizontal = 16.dp))
                for (i in 0 until keywords.length()) {
                    val pattern = keywords.optJSONObject(i)?.optString("pattern").orEmpty()
                    Row(Modifier.fillMaxWidth().heightIn(min = 48.dp).padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically) {
                        Text(pattern, fontSize = 16.sp, color = c.fg, modifier = Modifier.weight(1f))
                        TextAction("移除", dim = true, onClick = { keywords.remove(i); save() })
                    }
                }
                var draft by remember { mutableStateOf("") }
                Row(Modifier.padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    KeywordField(draft, { draft = it }, Modifier.weight(1f))
                    TextAction("添加", onClick = {
                        val p = draft.trim()
                        if (p.isNotEmpty()) {
                            keywords.put(JSONObject().put("pattern", p).put("regex", false).put("fields", JSONArray(listOf("title", "tags", "desc"))))
                            draft = ""
                            save()
                        }
                    })
                }
            }
            item(key = "weights-$version") {
                SectionTitle("推荐权重")
                Meta("调整后下一次刷新首页即生效", Modifier.padding(horizontal = 16.dp))
                FACTORS.forEach { (key, label) ->
                    var v by remember(version) { mutableStateOf(weights.optDouble(key, 0.0).toFloat()) }
                    Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically) {
                        Text(label, fontSize = 14.sp, color = c.fg, modifier = Modifier.width(72.dp))
                        Slider(
                            value = v,
                            onValueChange = { v = it },
                            onValueChangeFinished = { weights.put(key, v.toDouble()); save() },
                            valueRange = 0f..0.6f,
                            colors = SliderDefaults.colors(thumbColor = c.accent, activeTrackColor = c.accent, inactiveTrackColor = c.line),
                            modifier = Modifier.weight(1f),
                        )
                        Meta("%.2f".format(v), Modifier.width(40.dp), c.fg)
                    }
                }
            }
            item {
                SectionTitle("外观")
                Text("应用图标", fontSize = 16.sp, color = c.fg, modifier = Modifier.padding(start = 16.dp, top = 4.dp))
                Row(Modifier.padding(horizontal = 16.dp, vertical = 6.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    AppIcon.LABELS.forEachIndexed { i, label ->
                        val on = app.appIcon.value == i
                        Column(
                            Modifier.then(if (on) Modifier.border(2.dp, c.accent) else Modifier.border(1.dp, c.line))
                                .clickable {
                                    if (!on) {
                                        app.appIcon.value = i
                                        app.ui.appIcon = i
                                    }
                                }
                                .padding(10.dp),
                            horizontalAlignment = Alignment.CenterHorizontally,
                            verticalArrangement = Arrangement.spacedBy(6.dp),
                        ) {
                            RubiLogo(48.dp, i)
                            Text(if (i == AppIcon.TV) "$label（默认）" else label, fontSize = 12.sp, color = c.fg)
                        }
                    }
                }
                Meta("关于页和分享卡片立即换；桌面图标在退出 Rubi 后（返回退出或下次启动时）再换，部分桌面要几秒才刷新，有的需要重新把图标放到主屏幕。", Modifier.padding(horizontal = 16.dp))
                Segmented(listOf("跟随系统", "浅色", "深色"), app.themeMode.value) {
                    app.themeMode.value = it
                    app.ui.themeMode = it
                }
                Text("动效风格", fontSize = 16.sp, color = c.fg, modifier = Modifier.padding(start = 16.dp, top = 12.dp))
                Segmented(Motion.STYLES, app.motionStyle.value) {
                    app.motionStyle.value = it
                    app.ui.motionStyle = it
                }
                Meta(
                    listOf(
                        "页面绕边缘翻入，栏目滑入，按住卡片倾斜",
                        "点卡片以卡片中心为轴旋转放大，封面渐隐露出播放页；切页以画面中心为轴翻转",
                        "点卡片放大后从左侧翻开，封面渐隐露出播放页，返回时封面从左侧翻回再缩小；切页以屏幕左侧为轴翻页",
                    )[app.motionStyle.value.coerceIn(0, 2)],
                    Modifier.padding(horizontal = 16.dp),
                )
                Text("动效速度", fontSize = 16.sp, color = c.fg, modifier = Modifier.padding(start = 16.dp, top = 12.dp))
                Segmented(Motion.LABELS, app.motion.value) {
                    app.motion.value = it
                    app.ui.motion = it
                }
                Meta("非线性缓动：进入先快后慢；「无动效」立即切换。强调色 Cobalt。", Modifier.padding(16.dp))
            }
        }
    }
}

@Composable
private fun ToggleRow(title: String, desc: String, on: Boolean, onChange: (Boolean) -> Unit) {
    val c = LocalMetro.current
    Row(
        Modifier.fillMaxWidth().heightIn(min = 56.dp).clickable { onChange(!on) }.padding(horizontal = 16.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f).padding(end = 12.dp)) {
            Text(title, fontSize = 16.sp, color = c.fg)
            if (desc.isNotEmpty()) Meta(desc)
        }
        MetroToggle(on)
    }
}

@Composable
private fun KeywordField(value: String, onChange: (String) -> Unit, modifier: Modifier) {
    val c = LocalMetro.current
    androidx.compose.foundation.layout.Box(modifier.heightIn(min = 44.dp), contentAlignment = Alignment.CenterStart) {
        if (value.isEmpty()) Text("关键词", color = c.fg2, fontSize = 16.sp)
        androidx.compose.foundation.text.BasicTextField(
            value = value,
            onValueChange = onChange,
            singleLine = true,
            textStyle = androidx.compose.ui.text.TextStyle(color = c.fg, fontSize = 16.sp),
            cursorBrush = androidx.compose.ui.graphics.SolidColor(c.accent),
            modifier = Modifier.fillMaxWidth(),
        )
    }
}

/** A row of square segments, the chosen one filled with accent. */
@Composable
private fun Segmented(labels: List<String>, selected: Int, onSelect: (Int) -> Unit) {
    val c = LocalMetro.current
    Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 6.dp), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        labels.forEachIndexed { i, label ->
            val on = i == selected
            Box(
                Modifier.weight(1f).height(36.dp)
                    .then(if (on) Modifier.background(c.accent) else Modifier.border(1.dp, c.line))
                    .clickable { onSelect(i) },
                contentAlignment = Alignment.Center,
            ) { Text(label, fontSize = 13.sp, color = if (on) Color.White else c.fg) }
        }
    }
}
