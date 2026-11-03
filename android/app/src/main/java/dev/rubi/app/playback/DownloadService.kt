package dev.rubi.app.playback

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import dev.rubi.app.MainActivity
import dev.rubi.app.RubiApp
import dev.rubi.core.FfiDownloadState
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

/**
 * Keeps 离线缓存 going while Rubi is in the background: a foreground service with one progress
 * notification, alive only while something is downloading. The downloads themselves run in
 * the core; this only keeps the process from being reclaimed and shows how far they are.
 */
class DownloadService : Service() {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private var watch: Job? = null

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        channel()
        val first = notification("准备缓存…", 0, 0)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            startForeground(NOTIFICATION_ID, first, ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC)
        } else {
            startForeground(NOTIFICATION_ID, first)
        }
        if (watch?.isActive != true) watch = scope.launch { follow() }
        return START_NOT_STICKY
    }

    /** Updates the notification until nothing is downloading, then leaves. */
    private suspend fun follow() {
        val core = (application as RubiApp).core
        val manager = getSystemService(NotificationManager::class.java)
        while (true) {
            val active = runCatching { core.downloads() }.getOrDefault(emptyList())
                .filter { it.state == FfiDownloadState.RUNNING || it.state == FfiDownloadState.QUEUED }
            if (active.isEmpty()) break
            val done = active.sumOf { it.doneBytes }
            val total = active.sumOf { maxOf(it.totalBytes, it.doneBytes) }
            val title = if (active.size == 1) "正在缓存：${active[0].card.title}" else "正在缓存 ${active.size} 个视频"
            manager.notify(NOTIFICATION_ID, notification(title, done.toLong(), total.toLong()))
            delay(1_000)
        }
        stopForeground(STOP_FOREGROUND_REMOVE)
        stopSelf()
    }

    private fun notification(title: String, done: Long, total: Long): Notification {
        val open = PendingIntent.getActivity(
            this,
            0,
            Intent(this, MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )
        // Progress in per-mille so sizes above Int range still fit.
        val permille = if (total > 0) (done * 1000 / total).toInt() else 0
        return Notification.Builder(this, CHANNEL)
            .setSmallIcon(android.R.drawable.stat_sys_download)
            .setContentTitle(title)
            .setContentText(if (total > 0) "${permille / 10}%" else "")
            .setProgress(1000, permille, total <= 0)
            .setOnlyAlertOnce(true)
            .setOngoing(true)
            .setContentIntent(open)
            .build()
    }

    private fun channel() {
        val manager = getSystemService(NotificationManager::class.java)
        if (manager.getNotificationChannel(CHANNEL) == null) {
            manager.createNotificationChannel(NotificationChannel(CHANNEL, "离线缓存", NotificationManager.IMPORTANCE_LOW))
        }
    }

    override fun onDestroy() {
        scope.cancel()
        super.onDestroy()
    }

    companion object {
        private const val CHANNEL = "downloads"
        private const val NOTIFICATION_ID = 2

        /** Call after a download starts or resumes. */
        fun start(context: Context) {
            runCatching { context.startForegroundService(Intent(context, DownloadService::class.java)) }
        }
    }
}
