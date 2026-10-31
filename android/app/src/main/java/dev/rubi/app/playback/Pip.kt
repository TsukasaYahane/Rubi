package dev.rubi.app.playback

import android.app.Activity
import android.app.PictureInPictureParams
import android.content.pm.PackageManager
import android.os.Build
import android.util.Rational
import androidx.compose.runtime.mutableStateOf

/**
 * Picture-in-picture for the player page. The activity reports mode changes here; the player
 * page says when it is showing and whether leaving the app should shrink it into a window.
 */
object Pip {
    /** Whether the activity is in picture-in-picture right now (drives the compact layout). */
    val active = mutableStateOf(false)

    /** The player page is on screen. */
    @Volatile
    var onPlayer = false

    /** 切到后台自动小窗 is on. */
    @Volatile
    var autoEnter = false

    fun supported(activity: Activity): Boolean =
        activity.packageManager.hasSystemFeature(PackageManager.FEATURE_PICTURE_IN_PICTURE)

    private fun params(auto: Boolean): PictureInPictureParams =
        PictureInPictureParams.Builder()
            .setAspectRatio(Rational(16, 9))
            .apply { if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) setAutoEnterEnabled(auto) }
            .build()

    /** Shrinks into a window now (the 小窗 button). */
    fun enter(activity: Activity): Boolean =
        supported(activity) && runCatching { activity.enterPictureInPictureMode(params(false)) }.getOrDefault(false)

    /**
     * Keeps the activity's params in step with the page: on Android 12+ the system then enters
     * picture-in-picture by itself when the user leaves; older versions use [onUserLeaveHint].
     */
    fun sync(activity: Activity) {
        if (!supported(activity)) return
        runCatching { activity.setPictureInPictureParams(params(onPlayer && autoEnter)) }
    }

    /** Called from the activity's `onUserLeaveHint` (Home / Recents) before Android 12. */
    fun onUserLeaveHint(activity: Activity) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.S && onPlayer && autoEnter) enter(activity)
    }
}
