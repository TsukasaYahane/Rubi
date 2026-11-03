package dev.rubi.app.data

import android.content.Context
import dev.rubi.core.FfiLiveFilter

/** Per-device layout choices; unlike settings they are not part of the export. */
class UiPrefs(context: Context) {
    private val prefs = context.getSharedPreferences("rubi_ui", Context.MODE_PRIVATE)

    /** Home feed in two columns (true) or one large column. */
    var homeDual: Boolean
        get() = prefs.getBoolean("home_dual", true)
        set(value) = prefs.edit().putBoolean("home_dual", value).apply()

    /** 播完自动连播下一个: the next part, then the next video of the 合集. */
    var autoNext: Boolean
        get() = prefs.getBoolean("auto_next", true)
        set(value) = prefs.edit().putBoolean("auto_next", value).apply()

    /** 离线缓存 only on unmetered networks (Wi-Fi). */
    var downloadWifiOnly: Boolean
        get() = prefs.getBoolean("download_wifi_only", true)
        set(value) = prefs.edit().putBoolean("download_wifi_only", value).apply()

    /** 直播间过滤: what a live room holds back (gift banners, popups, effects); all by default. */
    var liveFilter: FfiLiveFilter
        get() = FfiLiveFilter(prefs.getBoolean("live_gifts", true), prefs.getBoolean("live_popups", true), prefs.getBoolean("live_effects", true))
        set(value) = prefs.edit().putBoolean("live_gifts", value.gifts).putBoolean("live_popups", value.popups).putBoolean("live_effects", value.effects).apply()

    /** How a live room draws its danmaku. */
    var liveDanmaku: LiveDanmakuPrefs
        get() = LiveDanmakuPrefs(
            enabled = prefs.getBoolean("live_dm_on", true),
            opacity = prefs.getFloat("live_dm_opacity", 1f),
            size = prefs.getInt("live_dm_size", 1),
            area = prefs.getInt("live_dm_area", 2),
        )
        set(value) = prefs.edit()
            .putBoolean("live_dm_on", value.enabled)
            .putFloat("live_dm_opacity", value.opacity)
            .putInt("live_dm_size", value.size)
            .putInt("live_dm_area", value.area)
            .apply()

    /** The clipboard text last offered as a video link, so it is offered once. */
    var lastClipboard: String
        get() = prefs.getString("last_clipboard", "").orEmpty()
        set(value) = prefs.edit().putString("last_clipboard", value).apply()

    /** 外观: 0 跟随系统, 1 浅色, 2 深色. */
    var themeMode: Int
        get() = prefs.getInt("theme_mode", 0)
        set(value) = prefs.edit().putInt("theme_mode", value).apply()

    /** 应用图标: 0 小电视 (default), 1 R 字. */
    var appIcon: Int
        get() = prefs.getInt("app_icon", 0)
        set(value) = prefs.edit().putInt("app_icon", value).apply()

    /** 动效风格: 0 默认, 1 卡牌, 2 书页. */
    var motionStyle: Int
        get() = prefs.getInt("motion_style", 0)
        set(value) = prefs.edit().putInt("motion_style", value).apply()

    /** 动效速度: 0 无动效, 1 慢, 2 标准, 3 快. */
    var motion: Int
        get() = prefs.getInt("motion", 2)
        set(value) = prefs.edit().putInt("motion", value).apply()

    /** Body text size of the 专栏 reader, in sp. */
    var articleSp: Int
        get() = prefs.getInt("article_sp", 16)
        set(value) = prefs.edit().putInt("article_sp", value).apply()

    /** Who was signed in last time (name, mid), shown at once while the account is re-read. */
    var lastAccount: Pair<String, Long>?
        get() = prefs.getString("account_name", null)?.let { it to prefs.getLong("account_mid", 0L) }
        set(value) {
            prefs.edit().apply {
                if (value == null) remove("account_name").remove("account_mid")
                else putString("account_name", value.first).putLong("account_mid", value.second)
            }.apply()
        }
}

/**
 * A live room's danmaku look: shown or not, opacity (0.2–1), size (0 small, 1 normal, 2 large)
 * and how much of the picture they may cover (0 a quarter, 1 half, 2 all of it).
 */
data class LiveDanmakuPrefs(val enabled: Boolean, val opacity: Float, val size: Int, val area: Int)
