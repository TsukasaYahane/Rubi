package dev.rubi.app

import android.app.Application
import dev.rubi.app.data.SessionStore
import androidx.compose.runtime.mutableStateOf
import dev.rubi.app.data.UiPrefs
import dev.rubi.core.FfiAccount
import dev.rubi.core.RubiCore
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob

/** Holds the single Rust core for the process. */
class RubiApp : Application() {
    lateinit var core: RubiCore
        private set
    lateinit var sessions: SessionStore
        private set

    lateinit var ui: UiPrefs
        private set

    /**
     * The signed-in account, kept for the whole process so pages do not start from 游客 each
     * time; seeded from the last known one when a session is stored.
     */
    val account = mutableStateOf<FfiAccount?>(null)

    /** Re-reads the account; a failed read (offline) keeps what is shown. */
    suspend fun refreshAccount() {
        val a = runCatching { core.account() }.getOrNull() ?: return
        account.value = a
        ui.lastAccount = if (a.loggedIn) a.name to a.mid.toLong() else null
    }

    /** For work that must outlive a page, e.g. the last history report when the player closes. */
    val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    override fun onCreate() {
        super.onCreate()
        sessions = SessionStore(this)
        ui = UiPrefs(this)
        val stored = sessions.load()
        core = RubiCore(filesDir.absolutePath, stored)
        if (stored != null) {
            ui.lastAccount?.let { (name, mid) -> account.value = FfiAccount(true, mid.toULong(), name) }
        }
    }

    /** Persists whatever session the core holds now (after login or device-id activation). */
    fun saveSession() = sessions.save(core.sessionJson())

    fun logout() {
        core.logout()
        sessions.clear()
        account.value = null
        ui.lastAccount = null
    }
}
