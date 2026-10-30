package dev.rubi.app

import android.app.Application
import dev.rubi.app.data.SessionStore
import dev.rubi.core.RubiCore

/** Holds the single Rust core for the process. */
class RubiApp : Application() {
    lateinit var core: RubiCore
        private set
    lateinit var sessions: SessionStore
        private set

    override fun onCreate() {
        super.onCreate()
        sessions = SessionStore(this)
        core = RubiCore(filesDir.absolutePath, sessions.load())
    }

    /** Persists whatever session the core holds now (after login or device-id activation). */
    fun saveSession() = sessions.save(core.sessionJson())

    fun logout() {
        core.logout()
        sessions.clear()
    }
}
