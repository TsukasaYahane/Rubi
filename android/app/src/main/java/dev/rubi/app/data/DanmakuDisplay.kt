package dev.rubi.app.data

import dev.rubi.core.RubiCore
import org.json.JSONObject

/** Mirror of `rubi_core::DanmakuDisplay`, stored inside the core's settings JSON. */
data class DanmakuDisplay(
    val enabled: Boolean = true,
    /** Share of the video height danmaku may use: 0.25, 0.5, 0.75 or 1.0. */
    val area: Float = 0.75f,
    val fontScale: Float = 1f,
    val opacity: Float = 0.9f,
    val speed: Float = 1f,
    val showScroll: Boolean = true,
    val showTop: Boolean = true,
    val showBottom: Boolean = true,
    val showColored: Boolean = true,
) {
    /** Time a scrolling danmaku takes to cross the screen. */
    val scrollMs: Long get() = (BASE_SCROLL_MS / speed.coerceIn(0.5f, 2f)).toLong()

    companion object {
        const val BASE_SCROLL_MS = 8_000f
        const val FIXED_MS = 4_000L
        private const val KEY = "danmaku_display"

        fun load(core: RubiCore): DanmakuDisplay = runCatching {
            val o = JSONObject(core.settingsJson()).optJSONObject(KEY) ?: return DanmakuDisplay()
            DanmakuDisplay(
                enabled = o.optBoolean("enabled", true),
                area = o.optDouble("area", 0.75).toFloat(),
                fontScale = o.optDouble("font_scale", 1.0).toFloat(),
                opacity = o.optDouble("opacity", 0.9).toFloat(),
                speed = o.optDouble("speed", 1.0).toFloat(),
                showScroll = o.optBoolean("show_scroll", true),
                showTop = o.optBoolean("show_top", true),
                showBottom = o.optBoolean("show_bottom", true),
                showColored = o.optBoolean("show_colored", true),
            )
        }.getOrDefault(DanmakuDisplay())

        fun save(core: RubiCore, d: DanmakuDisplay) {
            val settings = JSONObject(core.settingsJson())
            settings.put(
                KEY,
                JSONObject()
                    .put("enabled", d.enabled)
                    .put("area", d.area.toDouble())
                    .put("font_scale", d.fontScale.toDouble())
                    .put("opacity", d.opacity.toDouble())
                    .put("speed", d.speed.toDouble())
                    .put("show_scroll", d.showScroll)
                    .put("show_top", d.showTop)
                    .put("show_bottom", d.showBottom)
                    .put("show_colored", d.showColored),
            )
            core.setSettingsJson(settings.toString())
        }
    }
}
