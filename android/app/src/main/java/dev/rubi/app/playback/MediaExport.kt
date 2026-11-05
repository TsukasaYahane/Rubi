package dev.rubi.app.playback

import dev.rubi.core.DownloadsService
import dev.rubi.app.data.UiPrefs
import android.content.ContentValues
import android.content.Context
import android.media.MediaCodec
import android.media.MediaExtractor
import android.media.MediaFormat
import android.media.MediaMuxer
import android.net.Uri
import android.os.Build
import android.provider.MediaStore
import dev.rubi.core.FfiDownloadState
import dev.rubi.core.FfiLocalCopy
import java.io.File
import java.nio.ByteBuffer
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/** What an offline copy is also saved as, outside Rubi. */
enum class ExportFormat(val label: String) {
    /** Kept inside Rubi only (plays in 离线缓存). */
    None("仅 App 内"),
    /** Picture and sound in one MP4, in 影片/Rubi. */
    Mp4("MP4 视频"),
    /** The sound alone as M4A, in 音乐/Rubi. */
    Audio("音频 M4A"),
}

/**
 * Saves a finished offline copy as an MP4 or as audio, by moving its streams into a new file
 * as they are (no re-encoding, so it takes seconds and loses nothing). Needs Android 10 for the
 * shared folders.
 */
object MediaExport {
    /** Writes `copy` as `format` and returns where it went; throws with a reason when it cannot. */
    suspend fun export(context: Context, copy: FfiLocalCopy, title: String, format: ExportFormat): Uri = withContext(Dispatchers.IO) {
        require(format != ExportFormat.None)
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) throw IllegalStateException("导出需要 Android 10 及以上")
        val audio = format == ExportFormat.Audio
        val temp = File(context.cacheDir, "export-${System.nanoTime()}.${if (audio) "m4a" else "mp4"}")
        try {
            val muxer = MediaMuxer(temp.path, MediaMuxer.OutputFormat.MUXER_OUTPUT_MPEG_4)
            // Tracks to carry over: the picture (unless audio only) and the sound, wherever each is.
            val sources = buildList {
                if (!audio) add(copy.videoPath to "video/")
                add((copy.audioPath ?: copy.videoPath) to "audio/")
            }
            val extractors = sources.mapNotNull { (path, kind) -> openTrack(path, kind) }
            if (extractors.isEmpty()) throw IllegalStateException("缓存文件里没有可导出的内容")
            val tracks = extractors.map { (ex, fmt) -> ex to muxer.addTrack(fmt) }
            muxer.start()
            val buffer = ByteBuffer.allocate(BUFFER_BYTES)
            val info = MediaCodec.BufferInfo()
            // Interleaved by time: always the track whose next sample comes first.
            val open = tracks.toMutableList()
            while (open.isNotEmpty()) {
                val next = open.minBy { (ex, _) -> ex.sampleTime }
                val (ex, track) = next
                info.size = ex.readSampleData(buffer, 0)
                if (info.size < 0) {
                    ex.release()
                    open.remove(next)
                    continue
                }
                info.offset = 0
                info.presentationTimeUs = ex.sampleTime
                info.flags = if (ex.sampleFlags and MediaExtractor.SAMPLE_FLAG_SYNC != 0) MediaCodec.BUFFER_FLAG_KEY_FRAME else 0
                muxer.writeSampleData(track, buffer, info)
                ex.advance()
            }
            muxer.stop()
            muxer.release()
            publish(context, temp, title, audio)
        } finally {
            temp.delete()
        }
    }

    /** The first track of `kind` ("video/", "audio/") in the file at `path`, selected. */
    private fun openTrack(path: String, kind: String): Pair<MediaExtractor, MediaFormat>? {
        val ex = MediaExtractor()
        ex.setDataSource(path)
        for (i in 0 until ex.trackCount) {
            val fmt = ex.getTrackFormat(i)
            if (fmt.getString(MediaFormat.KEY_MIME)?.startsWith(kind) == true) {
                ex.selectTrack(i)
                return ex to fmt
            }
        }
        ex.release()
        return null
    }

    /** Copies the finished file into 影片/Rubi or 音乐/Rubi. */
    private fun publish(context: Context, file: File, title: String, audio: Boolean): Uri {
        val resolver = context.contentResolver
        val name = title.replace(Regex("""[\\/:*?"<>|]"""), "_").take(80)
        val values = ContentValues().apply {
            put(MediaStore.MediaColumns.DISPLAY_NAME, "$name.${if (audio) "m4a" else "mp4"}")
            put(MediaStore.MediaColumns.MIME_TYPE, if (audio) "audio/mp4" else "video/mp4")
            put(MediaStore.MediaColumns.RELATIVE_PATH, if (audio) "Music/Rubi" else "Movies/Rubi")
            put(MediaStore.MediaColumns.IS_PENDING, 1)
        }
        val collection = if (audio) MediaStore.Audio.Media.EXTERNAL_CONTENT_URI else MediaStore.Video.Media.EXTERNAL_CONTENT_URI
        val uri = resolver.insert(collection, values) ?: throw IllegalStateException("无法写入共享文件夹")
        runCatching {
            resolver.openOutputStream(uri)?.use { out -> file.inputStream().use { it.copyTo(out) } } ?: throw IllegalStateException("无法写入")
        }.onFailure {
            resolver.delete(uri, null, null)
            throw it
        }
        values.clear()
        values.put(MediaStore.MediaColumns.IS_PENDING, 0)
        resolver.update(uri, values, null, null)
        return uri
    }

    /**
     * Saves the copies that were asked to be exported once they finish. Called while downloads
     * run; returns what was saved, for a message.
     */
    suspend fun exportFinished(downloads: DownloadsService, uiPrefs: UiPrefs, context: Context): List<String> {
        val pending = uiPrefs.pendingExports
        if (pending.isEmpty()) return emptyList()
        val done = runCatching { downloads.downloads() }.getOrDefault(emptyList()).filter { it.state == FfiDownloadState.DONE }
        val said = mutableListOf<String>()
        for (d in done) {
            val format = pending[d.card.bvid] ?: continue
            uiPrefs.pendingExports = uiPrefs.pendingExports - d.card.bvid
            val copy = runCatching { downloads.downloadLocal(d.card.bvid) }.getOrNull() ?: continue
            runCatching { export(context, copy, d.card.title, format) }
                .onSuccess { said += "已导出${format.label}：${d.card.title}" }
                .onFailure { said += "导出失败：${it.message}" }
        }
        return said
    }

    private const val BUFFER_BYTES = 2 * 1024 * 1024
}
