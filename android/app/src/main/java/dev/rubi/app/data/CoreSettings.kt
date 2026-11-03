package dev.rubi.app.data

import dev.rubi.core.RubiCore
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONObject

/**
 * The core's settings as the app reads and changes them. A change is a patch of only the keys
 * it touches (a JSON merge patch), so two pages saving different settings never undo each
 * other; patches are written off the main thread, one after another in the order made.
 */
class CoreSettings(private val core: RubiCore, private val scope: CoroutineScope) {
    @OptIn(ExperimentalCoroutinesApi::class)
    private val writer = Dispatchers.IO.limitedParallelism(1)

    /** The whole settings object as it is now (in memory in the core: no disk read). */
    fun read(): JSONObject = runCatching { JSONObject(core.settingsJson()) }.getOrElse { JSONObject() }

    /** One top-level object, e.g. `preview`; empty when missing. */
    fun section(key: String): JSONObject = read().optJSONObject(key) ?: JSONObject()

    /**
     * Writes `patch`; `done` gets the error (e.g. a bad regex, when nothing changed) or null,
     * on the main thread.
     */
    fun patch(patch: JSONObject, done: (Throwable?) -> Unit = {}) {
        val text = patch.toString()
        scope.launch(writer) {
            val error = runCatching { core.patchSettingsJson(text) }.exceptionOrNull()
            withContext(Dispatchers.Main) { done(error) }
        }
    }
}
