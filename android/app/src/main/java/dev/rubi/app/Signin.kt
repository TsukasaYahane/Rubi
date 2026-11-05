package dev.rubi.app

import androidx.compose.runtime.mutableStateOf
import dev.rubi.app.data.SessionStore
import dev.rubi.app.data.UiPrefs
import dev.rubi.core.AccountService
import dev.rubi.core.FfiAccount

/**
 * Who is signed in, for the whole process (pages do not start from 游客 each time), and the
 * stored session behind it. `account` is resolved when first used: the core may still be
 * opening when this is built.
 */
class Signin(
    private val account: () -> AccountService,
    private val sessions: SessionStore,
    private val ui: UiPrefs,
    sessionStored: Boolean,
) {
    /** The signed-in account; seeded from the last known one when a session is stored. */
    val current = mutableStateOf<FfiAccount?>(null)

    /** Bumped whenever someone signs in or out: pages that show the account's content reload. */
    val changes = mutableStateOf(0)

    init {
        if (sessionStored) {
            ui.lastAccount?.let { (name, mid) -> current.value = FfiAccount(true, mid.toULong(), name) }
        }
    }

    /** Re-reads the account; a failed read (offline) keeps what is shown. */
    suspend fun refresh() {
        val a = runCatching { account().account() }.getOrNull() ?: return
        current.value = a
        ui.lastAccount = if (a.loggedIn) a.name to a.mid.toLong() else null
    }

    /** Persists whatever session the core holds now (after device-id activation). */
    fun saveCurrent() = sessions.save(account().sessionJson())

    /** Persists a session a login returned. */
    fun save(sessionJson: String) {
        sessions.save(sessionJson)
        changes.value++
    }

    fun logout() {
        account().logout()
        sessions.clear()
        current.value = null
        ui.lastAccount = null
        changes.value++
    }
}
