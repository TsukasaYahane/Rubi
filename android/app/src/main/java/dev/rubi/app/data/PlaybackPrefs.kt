package dev.rubi.app.data

import dev.rubi.core.RubiCore
import org.json.JSONObject

/** Playback options stored in the core's settings JSON. */
object PlaybackPrefs {
    private const val BACKGROUND_PLAY = "background_play"

    /** Whether to keep playing when the app goes to the background (default: pause). */
    fun backgroundPlay(core: RubiCore): Boolean =
        runCatching { JSONObject(core.settingsJson()).optBoolean(BACKGROUND_PLAY, false) }.getOrDefault(false)
}
