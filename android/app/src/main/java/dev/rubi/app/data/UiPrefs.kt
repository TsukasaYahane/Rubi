package dev.rubi.app.data

import android.content.Context

/** Per-device layout choices; unlike settings they are not part of the export. */
class UiPrefs(context: Context) {
    private val prefs = context.getSharedPreferences("rubi_ui", Context.MODE_PRIVATE)

    /** Home feed in two columns (true) or one large column. */
    var homeDual: Boolean
        get() = prefs.getBoolean("home_dual", true)
        set(value) = prefs.edit().putBoolean("home_dual", value).apply()
}
