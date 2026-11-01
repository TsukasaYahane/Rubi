package dev.rubi.app

import android.content.Intent
import android.content.res.Configuration
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import dev.rubi.app.playback.ACTION_LISTEN
import dev.rubi.app.playback.AudioQueue
import dev.rubi.app.playback.Pip
import dev.rubi.app.ui.RubiRoot
import dev.rubi.app.ui.theme.MetroTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        val app = application as RubiApp
        setContent { MetroTheme { RubiRoot(app) } }
        handle(intent)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        handle(intent)
    }

    /** The playback notification: open the 听视频 page when something is being heard. */
    private fun handle(intent: Intent?) {
        if (intent?.action == ACTION_LISTEN && AudioQueue.current.value != null) AudioQueue.pageRequested.value = true
    }

    override fun onUserLeaveHint() {
        super.onUserLeaveHint()
        Pip.onUserLeaveHint(this)
    }

    override fun onPictureInPictureModeChanged(isInPictureInPictureMode: Boolean, newConfig: Configuration) {
        super.onPictureInPictureModeChanged(isInPictureInPictureMode, newConfig)
        Pip.active.value = isInPictureInPictureMode
    }
}
