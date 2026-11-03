package dev.rubi.app.data

import android.content.Context
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import org.json.JSONObject

/** 大图预览 options, stored in the core's settings JSON under `preview`. */
data class PreviewPrefs(
    val enabled: Boolean = true,
    val wifiOnly: Boolean = true,
    /** 0 = muted. */
    val volume: Float = 0f,
    val danmaku: Boolean = false,
) {
    /** Whether a preview may play right now (setting on, and on Wi-Fi when asked). */
    fun allowed(context: Context): Boolean = enabled && (!wifiOnly || context.onUnmeteredNetwork())

    companion object {
        private const val KEY = "preview"

        fun load(settings: CoreSettings): PreviewPrefs = runCatching {
            val o = settings.read().optJSONObject(KEY) ?: return PreviewPrefs()
            PreviewPrefs(
                enabled = o.optBoolean("enabled", true),
                wifiOnly = o.optBoolean("wifi_only", true),
                volume = o.optDouble("volume", 0.0).toFloat(),
                danmaku = o.optBoolean("danmaku", false),
            )
        }.getOrDefault(PreviewPrefs())

        /** Writes only this section; other settings stay as they are. */
        fun save(settings: CoreSettings, p: PreviewPrefs) {
            val patch = JSONObject()
            patch.put(
                KEY,
                JSONObject()
                    .put("enabled", p.enabled)
                    .put("wifi_only", p.wifiOnly)
                    .put("volume", p.volume.toDouble())
                    .put("danmaku", p.danmaku),
            )
            settings.patch(patch)
        }
    }
}
