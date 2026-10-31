package dev.rubi.app.data

import dev.rubi.core.RubiCore
import org.json.JSONObject

/** Playback options stored in the core's settings JSON. */
object PlaybackPrefs {
    private const val BACKGROUND_PLAY = "background_play"
    private const val LONG_PRESS_SPEED = "long_press_speed"
    private const val PREFERRED_QUALITY = "preferred_quality"

    /** Speeds offered for holding the fullscreen player. */
    val SPEEDS = listOf(1.25f, 1.5f, 2f, 3f)

    private fun settings(core: RubiCore): JSONObject? = runCatching { JSONObject(core.settingsJson()) }.getOrNull()

    /** Whether to keep playing when the app goes to the background (default: pause). */
    fun backgroundPlay(core: RubiCore): Boolean = settings(core)?.optBoolean(BACKGROUND_PLAY, false) ?: false

    /** Leaving the app from the player shrinks it into picture-in-picture (default off). */
    fun autoPip(core: RubiCore): Boolean = settings(core)?.optBoolean("auto_pip", false) ?: false

    /** Speed while the fullscreen player is held down (default 1.5×). */
    fun longPressSpeed(core: RubiCore): Float =
        settings(core)?.optDouble(LONG_PRESS_SPEED, 1.5)?.toFloat()?.takeIf { it > 0f } ?: 1.5f

    /** Quality (qn) asked for when a video opens (default 80 = 1080P). */
    fun preferredQuality(core: RubiCore): UInt =
        settings(core)?.optInt(PREFERRED_QUALITY, 80)?.takeIf { it > 0 }?.toUInt() ?: 80u

    fun setPreferredQuality(core: RubiCore, qn: UInt) {
        val s = settings(core) ?: return
        s.put(PREFERRED_QUALITY, qn.toInt())
        runCatching { core.setSettingsJson(s.toString()) }
    }
}
