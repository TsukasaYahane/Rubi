package dev.rubi.app.data

import android.content.Context

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
