package dev.rubi.app.ui

import dev.rubi.core.VideosService
import dev.rubi.app.Appearance
import dev.rubi.app.rubiLink
import android.content.ContentValues
import android.content.Context
import android.content.ClipData
import android.content.Intent
import androidx.core.content.FileProvider
import java.io.File
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.Rect
import android.graphics.RectF
import android.graphics.Typeface
import android.net.Uri
import android.os.Build
import android.provider.MediaStore
import android.text.Layout
import android.text.StaticLayout
import android.text.TextPaint
import android.text.TextUtils
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import coil3.SingletonImageLoader
import coil3.request.ImageRequest
import coil3.request.SuccessResult
import coil3.request.allowHardware
import coil3.toBitmap
import com.google.zxing.BarcodeFormat
import com.google.zxing.EncodeHintType
import com.google.zxing.qrcode.QRCodeWriter
import dev.rubi.app.ui.theme.Cobalt
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiVideoDetail
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/** The card's width in pixels; everything else is laid out from it. */
private const val CARD_W = 1080
private const val PAD = 60

/**
 * Rubi 专属转发卡片: the video as a picture — cover, title, UP and plays, the Rubi mark and a QR
 * code of the link — to save to the gallery or hand to another app.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ShareCardSheet(videos: VideosService, appearance: Appearance, d: FfiVideoDetail, onDismiss: () -> Unit, done: (String) -> Unit) {
    val c = LocalMetro.current
    val ctx = LocalContext.current
    val scope = rememberCoroutineScope()
    var card by remember { mutableStateOf<Bitmap?>(null) }
    var note by remember { mutableStateOf("生成中…") }
    LaunchedEffect(d.bvid) {
        card = runCatching {
            val cover = runCatching { videos.video(d.bvid).cover }.getOrDefault("")
            renderCard(ctx, d, cover, appearance.appIcon.value)
        }.onFailure { note = "生成失败：${it.message}" }.getOrNull()
        if (card != null) note = ""
    }
    ModalBottomSheet(onDismissRequest = onDismiss, containerColor = c.bg2, shape = RectangleShape, dragHandle = null) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Text("分享卡片", style = MetroType.section, color = c.fg)
            val shown = card
            if (shown == null) {
                Box(Modifier.fillMaxWidth().heightIn(min = 200.dp), contentAlignment = Alignment.Center) { Meta(note) }
            } else {
                Image(shown.asImageBitmap(), contentDescription = "分享卡片", contentScale = ContentScale.FillWidth, modifier = Modifier.fillMaxWidth().background(c.bg))
            }
            if (shown != null) {
                Row(horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
                    SmallButton("保存到相册", c.accent) {
                        scope.launch {
                            val uri = withContext(Dispatchers.IO) { saveToGallery(ctx, shown, d.bvid) }
                            note = if (uri != null) "已保存到相册 Pictures/Rubi" else "保存失败（需要 Android 10 及以上）"
                            if (uri != null) done("分享卡片已保存到相册")
                        }
                    }
                    SmallButton("分享图片", null) {
                        scope.launch {
                            // Handed over from the cache, not the gallery: nothing is kept.
                            val uri = withContext(Dispatchers.IO) { runCatching { shareFile(ctx, shown, d.bvid) }.getOrNull() }
                            if (uri == null) {
                                note = "分享失败"
                                return@launch
                            }
                            val send = Intent(Intent.ACTION_SEND).setType("image/png")
                                .putExtra(Intent.EXTRA_STREAM, uri)
                                .also { it.clipData = ClipData.newRawUri("", uri) }
                                .putExtra(Intent.EXTRA_TEXT, "【${d.title}】 https://www.bilibili.com/video/${d.bvid}\nRubi 客户端打开：${rubiLink(d.bvid)}")
                                .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                            val chooser = Intent.createChooser(send, "分享卡片").addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                            // The sheet goes first: some systems stall drawing a chooser over one.
                            runCatching { ctx.startActivity(chooser) }
                                .onSuccess { onDismiss() }
                                .onFailure { note = "分享失败：${it.message}" }
                        }
                    }
                }
                if (note.isNotEmpty()) Meta(note)
            }
            Row(Modifier.padding(bottom = 8.dp)) { TextAction("关闭", onClick = onDismiss) }
        }
    }
}

/** Draws the card: white, square-cornered, Cobalt for the mark. */
private suspend fun renderCard(ctx: Context, d: FfiVideoDetail, coverUrl: String, icon: Int): Bitmap = withContext(Dispatchers.Default) {
    val inner = CARD_W - PAD * 2
    val coverH = inner * 9 / 16
    val title = TextPaint(Paint.ANTI_ALIAS_FLAG).apply { color = 0xFF111111.toInt(); textSize = 50f; typeface = Typeface.DEFAULT_BOLD }
    val meta = TextPaint(Paint.ANTI_ALIAS_FLAG).apply { color = 0xFF666666.toInt(); textSize = 34f }
    val brand = TextPaint(Paint.ANTI_ALIAS_FLAG).apply {
        color = Cobalt.toArgb()
        textSize = 72f
        typeface = Typeface.create(Typeface.MONOSPACE, Typeface.BOLD)
        letterSpacing = 0.25f
    }
    val small = TextPaint(Paint.ANTI_ALIAS_FLAG).apply { color = 0xFF777777.toInt(); textSize = 28f }
    val titleLayout = StaticLayout.Builder.obtain(d.title, 0, d.title.length, title, inner)
        .setAlignment(Layout.Alignment.ALIGN_NORMAL).setMaxLines(3).setEllipsize(TextUtils.TruncateAt.END).setLineSpacing(8f, 1f).build()
    val qrSize = 240
    val height = PAD + coverH + 40 + titleLayout.height + 30 + 44 + 50 + 2 + 40 + qrSize + PAD
    val bmp = Bitmap.createBitmap(CARD_W, height, Bitmap.Config.ARGB_8888)
    val canvas = Canvas(bmp)
    canvas.drawColor(0xFFFFFFFF.toInt())

    // Cover, cropped to 16:9.
    val coverRect = Rect(PAD, PAD, PAD + inner, PAD + coverH)
    val cover = if (coverUrl.isNotEmpty()) loadBitmap(ctx, coverUrl) else null
    if (cover != null) {
        val scale = maxOf(inner.toFloat() / cover.width, coverH.toFloat() / cover.height)
        val w = (inner / scale).toInt()
        val h = (coverH / scale).toInt()
        val sx = (cover.width - w) / 2
        val sy = (cover.height - h) / 2
        canvas.drawBitmap(cover, Rect(sx, sy, sx + w, sy + h), coverRect, Paint(Paint.FILTER_BITMAP_FLAG))
    } else {
        canvas.drawRect(RectF(coverRect), Paint().apply { color = 0xFFDFE6EF.toInt() })
    }
    var y = PAD + coverH + 40f
    canvas.save()
    canvas.translate(PAD.toFloat(), y)
    titleLayout.draw(canvas)
    canvas.restore()
    y += titleLayout.height + 30 + 34
    canvas.drawText("${d.upName} · ${formatCount(d.views)} 播放", PAD.toFloat(), y, meta)
    y += 50
    canvas.drawRect(PAD.toFloat(), y, (PAD + inner).toFloat(), y + 2, Paint().apply { color = 0xFFDDDDDD.toInt() })
    y += 2 + 40

    // The mark on the left, the QR code of the link on the right.
    drawRubiLogo(canvas, PAD.toFloat(), y + 12, 84f, icon)
    canvas.drawText("RUBI", PAD + 104f, y + 80, brand)
    canvas.drawText("扫码在 B 站观看", PAD.toFloat(), y + 140, small)
    canvas.drawText(d.bvid, PAD.toFloat(), y + 182, small)
    canvas.drawText("Rubi 客户端：${rubiLink(d.bvid)}", PAD.toFloat(), y + 224, small)
    val qr = QRCodeWriter().encode("https://www.bilibili.com/video/${d.bvid}", BarcodeFormat.QR_CODE, qrSize, qrSize, mapOf(EncodeHintType.MARGIN to 0))
    val dark = Paint().apply { color = 0xFF111111.toInt() }
    val left = PAD + inner - qrSize
    for (qy in 0 until qr.height) for (qx in 0 until qr.width) {
        if (qr.get(qx, qy)) canvas.drawRect((left + qx).toFloat(), y + qy, (left + qx + 1).toFloat(), y + qy + 1, dark)
    }
    bmp
}

private suspend fun loadBitmap(ctx: Context, url: String): Bitmap? {
    val req = ImageRequest.Builder(ctx).data(url).allowHardware(false).build()
    return (SingletonImageLoader.get(ctx).execute(req) as? SuccessResult)?.image?.toBitmap()
}

/**
 * Writes the card to the cache for another app to read, replacing the last one shared; the
 * folder is emptied again at the next start ([clearSharedCards]).
 */
private fun shareFile(ctx: Context, bmp: Bitmap, bvid: String): Uri {
    val dir = File(ctx.cacheDir, SHARED_DIR).apply { mkdirs() }
    val file = File(dir, "rubi_$bvid.png")
    // The same card again keeps its file: an app may still be reading it from the last share.
    if (file.length() == 0L) file.outputStream().use { bmp.compress(Bitmap.CompressFormat.PNG, 100, it) }
    return FileProvider.getUriForFile(ctx, "${ctx.packageName}.share", file)
}

private const val SHARED_DIR = "shared"

/** Removes cards left in the cache by earlier shares. */
fun clearSharedCards(ctx: Context) {
    File(ctx.cacheDir, SHARED_DIR).deleteRecursively()
}

/** Writes the card to Pictures/Rubi; null when it cannot be written (before Android 10). */
private fun saveToGallery(ctx: Context, bmp: Bitmap, bvid: String): Uri? {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) return null
    val values = ContentValues().apply {
        put(MediaStore.Images.Media.DISPLAY_NAME, "rubi_${bvid}_${System.currentTimeMillis() / 1000}.png")
        put(MediaStore.Images.Media.MIME_TYPE, "image/png")
        put(MediaStore.Images.Media.RELATIVE_PATH, "Pictures/Rubi")
        put(MediaStore.Images.Media.IS_PENDING, 1)
    }
    val resolver = ctx.contentResolver
    val uri = resolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, values) ?: return null
    val ok = runCatching { resolver.openOutputStream(uri)?.use { bmp.compress(Bitmap.CompressFormat.PNG, 100, it) } == true }.getOrDefault(false)
    if (!ok) {
        resolver.delete(uri, null, null)
        return null
    }
    values.clear()
    values.put(MediaStore.Images.Media.IS_PENDING, 0)
    resolver.update(uri, values, null, null)
    return uri
}
