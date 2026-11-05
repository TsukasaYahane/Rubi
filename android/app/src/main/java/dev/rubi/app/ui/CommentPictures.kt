package dev.rubi.app.ui

import android.content.ClipData
import android.content.ClipboardManager
import android.content.ContentValues
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.os.Build
import android.provider.MediaStore
import android.widget.Toast
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.animateDecay
import androidx.compose.animation.core.FloatExponentialDecaySpec
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.calculateCentroid
import androidx.compose.foundation.gestures.calculatePan
import androidx.compose.foundation.gestures.calculateZoom
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.input.pointer.positionChange
import androidx.compose.ui.input.pointer.util.VelocityTracker
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.core.content.FileProvider
import coil3.SingletonImageLoader
import coil3.compose.AsyncImage
import coil3.request.ImageRequest
import coil3.request.SuccessResult
import coil3.request.allowHardware
import coil3.toBitmap
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.core.FfiPicture
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.ByteArrayOutputStream
import java.io.File
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min
import kotlin.math.sqrt

private val THUMB = 80.dp
private val GAP = 6.dp

/** Height over width from which a picture is a 长图: shown full width and scrolled through. */
private const val LONG_RATIO = 3f

/** Most a picture is enlarged, over its fitted size. */
private const val MAX_SCALE = 4f

/** Where a double tap takes it. */
private const val DOUBLE_TAP_SCALE = 2.5f

/** Most pixels a long picture is decoded at, to keep its bitmap drawable (and memory sane). */
private const val LONG_MAX_PIXELS = 16_000_000f

/** Height over width, or 0 when the size is not known. */
private fun FfiPicture.ratio(): Float = if (width > 0u && height > 0u) height.toFloat() / width.toFloat() else 0f

/**
 * Images attached to a comment or post, in one row of thumbnails. What does not fit is counted
 * on the last one ("+N"); a 长图 shows its top with a mark. Tapping any opens a viewer that
 * swipes through all of them.
 */
@Composable
fun CommentPictures(pictures: List<FfiPicture>) {
    val c = LocalMetro.current
    var open by remember { mutableStateOf<Int?>(null) }
    BoxWithConstraints(Modifier.padding(top = 6.dp)) {
        val fit = ((maxWidth + GAP) / (THUMB + GAP)).toInt().coerceAtLeast(1)
        val shown = pictures.take(fit)
        val more = pictures.size - shown.size
        Row(horizontalArrangement = Arrangement.spacedBy(GAP)) {
            shown.forEachIndexed { i, p ->
                // A lone picture keeps its shape (within limits); several are squares.
                val single = pictures.size == 1 && p.width > 0u && p.height > 0u
                val ratio = if (single) (p.width.toFloat() / p.height.toFloat()).coerceIn(0.5f, 2f) else 1f
                val long = p.ratio() >= LONG_RATIO
                Box(Modifier.height(if (single) 120.dp else THUMB).width(if (single) 120.dp * ratio else THUMB).background(c.bg2).clickable { open = i }) {
                    AsyncImage(
                        model = p.url,
                        contentDescription = null,
                        contentScale = ContentScale.Crop,
                        alignment = if (long) Alignment.TopCenter else Alignment.Center,
                        modifier = Modifier.fillMaxSize(),
                    )
                    if (long) {
                        Text(
                            "长图",
                            fontSize = 11.sp,
                            color = Color.White,
                            modifier = Modifier.align(Alignment.BottomStart).background(c.accent).padding(horizontal = 5.dp, vertical = 1.dp),
                        )
                    }
                    if (i == shown.lastIndex && more > 0) {
                        Box(Modifier.fillMaxSize().background(Color(0x99000000)), contentAlignment = Alignment.Center) {
                            Text("+$more", color = Color.White, fontSize = 18.sp)
                        }
                    }
                }
            }
        }
    }
    open?.let { start -> PictureViewer(pictures, start) { open = null } }
}

/**
 * Full-screen pictures, swiped left and right. Pinch or double-tap to enlarge, drag once
 * enlarged; a 长图 fills the width and scrolls up and down. A tap closes it; a long press
 * offers 保存到相册 / 分享图片 / 复制图片链接.
 */
@Composable
fun PictureViewer(pictures: List<FfiPicture>, start: Int, onClose: () -> Unit) {
    val ctx = LocalContext.current
    val scope = rememberCoroutineScope()
    val pager = rememberPagerState(initialPage = start) { pictures.size }
    // The page on show is enlarged: its drags move the picture, not the pager.
    var zoomed by remember { mutableStateOf(false) }
    var menu by remember { mutableStateOf<FfiPicture?>(null) }
    Dialog(onDismissRequest = onClose, properties = DialogProperties(usePlatformDefaultWidth = false)) {
        Box(Modifier.fillMaxSize().background(Color.Black)) {
            HorizontalPager(pager, Modifier.fillMaxSize(), userScrollEnabled = !zoomed, beyondViewportPageCount = 1) { page ->
                ZoomPage(
                    pictures[page],
                    active = page == pager.currentPage,
                    onZoomed = { zoomed = it },
                    onTap = onClose,
                    onLongPress = { menu = pictures[page] },
                )
            }
            if (pictures.size > 1) {
                Text(
                    "${pager.currentPage + 1} / ${pictures.size}",
                    color = Color.White,
                    fontSize = 14.sp,
                    modifier = Modifier.align(Alignment.BottomCenter).padding(24.dp).background(Color(0x8C000000)).padding(horizontal = 10.dp, vertical = 4.dp),
                )
            }
            menu?.let { p ->
                PictureMenu(onDismiss = { menu = null }) { action ->
                    menu = null
                    scope.launch {
                        val note = when (action) {
                            PictureAction.Save -> savePicture(ctx, p.url)
                            PictureAction.Share -> sharePicture(ctx, p.url)
                            PictureAction.Copy -> {
                                copyLink(ctx, p.url)
                                "已复制图片链接"
                            }
                        }
                        note?.let { Toast.makeText(ctx, it, Toast.LENGTH_SHORT).show() }
                    }
                }
            }
        }
    }
}

/**
 * One picture, fitted to the screen (a 长图 to its width, from the top) and enlarged up to
 * [MAX_SCALE]. Sizes in px: `bw`×`bh` is the picture at scale 1, drawn from its top-left and
 * moved by `x`/`y`.
 */
@Composable
private fun ZoomPage(pic: FfiPicture, active: Boolean, onZoomed: (Boolean) -> Unit, onTap: () -> Unit, onLongPress: () -> Unit) {
    val ctx = LocalContext.current
    val density = LocalDensity.current
    val scope = rememberCoroutineScope()
    // Known from the comment, or from the picture once it has loaded (a post's pictures).
    var ratio by remember(pic.url) { mutableFloatStateOf(pic.ratio()) }
    BoxWithConstraints(Modifier.fillMaxSize().clipToBounds()) {
        val vw = constraints.maxWidth.toFloat()
        val vh = constraints.maxHeight.toFloat()
        val long = ratio >= LONG_RATIO
        val bw = when {
            ratio <= 0f -> vw
            long -> vw
            else -> min(vw, vh / ratio)
        }
        val bh = if (ratio <= 0f) vh else bw * ratio

        var s by remember { mutableFloatStateOf(1f) }
        var x by remember { mutableFloatStateOf(0f) }
        var y by remember { mutableFloatStateOf(0f) }
        fun clamp() {
            val w = bw * s
            val h = bh * s
            x = if (w <= vw) (vw - w) / 2 else x.coerceIn(vw - w, 0f)
            y = if (h <= vh) (vh - h) / 2 else y.coerceIn(vh - h, 0f)
        }
        // Enlarges to `to` keeping the point under `at` where it is.
        fun zoomAt(to: Float, at: Offset) {
            val ns = to.coerceIn(1f, MAX_SCALE)
            x = at.x - (at.x - x) * ns / s
            y = at.y - (at.y - y) * ns / s
            s = ns
            clamp()
        }
        // Back to fitted, a 长图 from its top, whenever the size changes or the page is left.
        LaunchedEffect(bw, bh, active) {
            if (!active || s <= 1f) {
                s = 1f
                x = 0f
                y = 0f
                clamp()
            }
        }
        LaunchedEffect(active, s > 1.01f) { if (active) onZoomed(s > 1.01f) }

        var fling by remember { mutableStateOf<Job?>(null) }
        val decoded = remember(pic.url, long, bw, bh) {
            val b = ImageRequest.Builder(ctx).data(pic.url)
            if (long) {
                // Decoded at screen width, fewer pixels for a very long one.
                val k = min(1f, sqrt(LONG_MAX_PIXELS / (bw * bh)))
                b.size((bw * k).toInt().coerceAtLeast(1), (bh * k).toInt().coerceAtLeast(1)).allowHardware(false)
            }
            b.build()
        }
        AsyncImage(
            model = decoded,
            contentDescription = null,
            contentScale = ContentScale.Fit,
            onSuccess = { r ->
                val size = r.painter.intrinsicSize
                if (ratio <= 0f && size.width > 0f && size.height > 0f) ratio = size.height / size.width
            },
            modifier = Modifier
                .size(with(density) { bw.toDp() }, with(density) { bh.toDp() })
                .graphicsLayer {
                    transformOrigin = TransformOrigin(0f, 0f)
                    scaleX = s
                    scaleY = s
                    translationX = x
                    translationY = y
                },
        )
        Box(
            Modifier.fillMaxSize()
                .pointerInput(pic.url) {
                    detectTapGestures(
                        onTap = { onTap() },
                        onLongPress = { onLongPress() },
                        onDoubleTap = { at ->
                            fling?.cancel()
                            scope.launch {
                                val from = s
                                val to = if (s > 1.01f) 1f else DOUBLE_TAP_SCALE
                                val t = Animatable(0f)
                                t.animateTo(1f, tween(220)) { zoomAt(from + (to - from) * value, at) }
                            }
                        },
                    )
                }
                .pointerInput(pic.url, long, bw, bh) {
                    awaitEachGesture {
                        awaitFirstDown(requireUnconsumed = false)
                        fling?.cancel()
                        val tracker = VelocityTracker()
                        var travelled = Offset.Zero
                        var decided = false
                        var panning = false
                        do {
                            val ev = awaitPointerEvent()
                            val pressed = ev.changes.filter { it.pressed }
                            if (pressed.size >= 2) {
                                // Pinch: always the picture's, never the pager's.
                                zoomAt(s * ev.calculateZoom(), ev.calculateCentroid())
                                val pan = ev.calculatePan()
                                x += pan.x
                                y += pan.y
                                clamp()
                                ev.changes.forEach { it.consume() }
                                decided = true
                                panning = true
                            } else if (pressed.size == 1) {
                                val ch = pressed[0]
                                val d = ch.positionChange()
                                tracker.addPosition(ch.uptimeMillis, ch.position)
                                if (!decided) {
                                    travelled += d
                                    if (travelled.getDistance() > viewConfiguration.touchSlop) {
                                        decided = true
                                        // Enlarged, or a 长图 dragged up and down: the picture
                                        // moves. Otherwise the drag is left to the pager.
                                        panning = s > 1.01f || (long && abs(travelled.y) > abs(travelled.x))
                                    }
                                }
                                if (decided && panning) {
                                    x += d.x
                                    y += d.y
                                    clamp()
                                    ch.consume()
                                }
                            }
                        } while (ev.changes.any { it.pressed })
                        if (panning) {
                            val v = tracker.calculateVelocity()
                            fling = scope.launch {
                                coroutineScope {
                                    val decay = FloatExponentialDecaySpec(frictionMultiplier = 1.4f)
                                    launch { animateDecay(x, v.x, decay) { value, _ -> x = value; clamp() } }
                                    launch { animateDecay(y, v.y, decay) { value, _ -> y = value; clamp() } }
                                }
                            }
                        }
                    }
                },
        )
        // 长图: what it is, and where in it the screen is.
        if (long) {
            Text(
                if (s > 1.01f) "长图 · ${(s * 100).toInt()}%" else "长图 · 上下滑动浏览",
                fontSize = 12.sp,
                color = Color.White,
                modifier = Modifier.align(Alignment.TopStart).padding(12.dp).background(Color(0x8C000000)).padding(horizontal = 8.dp, vertical = 3.dp),
            )
            val h = bh * s
            if (h > vh) {
                val railH = vh * 0.7f
                val knobH = max(railH * vh / h, with(density) { 16.dp.toPx() })
                val at = (railH - knobH) * (-y / (h - vh)).coerceIn(0f, 1f)
                Box(
                    Modifier.align(Alignment.CenterEnd).padding(end = 3.dp)
                        .size(3.dp, with(density) { railH.toDp() })
                        .background(Color(0x33FFFFFF)),
                ) {
                    Box(Modifier.offset(y = with(density) { at.toDp() }).size(3.dp, with(density) { knobH.toDp() }).background(Color.White))
                }
            }
        } else if (s > 1.01f) {
            Text(
                "${(s * 100).toInt()}%",
                fontSize = 12.sp,
                color = Color.White,
                modifier = Modifier.align(Alignment.TopEnd).padding(12.dp).background(Color(0x8C000000)).padding(horizontal = 8.dp, vertical = 3.dp),
            )
        }
    }
}

private enum class PictureAction(val label: String) { Save("保存到相册"), Share("分享图片…"), Copy("复制图片链接") }

/** The long-press menu, a dark sheet along the foot of the viewer. */
@Composable
private fun PictureMenu(onDismiss: () -> Unit, onPick: (PictureAction) -> Unit) {
    Box(Modifier.fillMaxSize().background(Color(0x66000000)).clickable(onClick = onDismiss)) {
        Column(Modifier.align(Alignment.BottomCenter).fillMaxWidth().background(Color(0xFF1F1F1F)).clickable(enabled = false) {}.padding(vertical = 6.dp)) {
            PictureAction.entries.forEach { a ->
                Text(a.label, fontSize = 16.sp, color = Color.White, modifier = Modifier.fillMaxWidth().clickable { onPick(a) }.padding(horizontal = 20.dp, vertical = 14.dp))
            }
            Text("取消", fontSize = 16.sp, color = Color(0xFFAAAAAA), modifier = Modifier.fillMaxWidth().clickable(onClick = onDismiss).padding(horizontal = 20.dp, vertical = 14.dp))
        }
    }
}

/** A picture's file as it was sent (from the image cache when there), with its type and extension. */
private class PictureFile(val bytes: ByteArray, val mime: String, val ext: String)

private suspend fun pictureFile(ctx: Context, url: String): PictureFile? = withContext(Dispatchers.IO) {
    val loader = SingletonImageLoader.get(ctx)
    // Loaded small: enough to have the original in the disk cache.
    val small = loader.execute(ImageRequest.Builder(ctx).data(url).size(64, 64).build())
    val cached = loader.diskCache?.let { cache ->
        runCatching { cache.openSnapshot(url)?.use { snap -> cache.fileSystem.read(snap.data) { readByteArray() } } }.getOrNull()
    }
    if (cached != null && cached.isNotEmpty()) {
        val (mime, ext) = typeOf(cached)
        return@withContext PictureFile(cached, mime, ext)
    }
    // Not in the disk cache: the decoded picture, as PNG.
    if (small !is SuccessResult) return@withContext null
    val full = loader.execute(ImageRequest.Builder(ctx).data(url).allowHardware(false).build()) as? SuccessResult ?: return@withContext null
    val out = ByteArrayOutputStream()
    full.image.toBitmap().compress(Bitmap.CompressFormat.PNG, 100, out)
    PictureFile(out.toByteArray(), "image/png", "png")
}

/** The picture's type from its first bytes. */
private fun typeOf(b: ByteArray): Pair<String, String> = when {
    b.size > 3 && b[0] == 0xFF.toByte() && b[1] == 0xD8.toByte() -> "image/jpeg" to "jpg"
    b.size > 4 && b[0] == 0x89.toByte() && b[1] == 'P'.code.toByte() -> "image/png" to "png"
    b.size > 3 && b[0] == 'G'.code.toByte() && b[1] == 'I'.code.toByte() && b[2] == 'F'.code.toByte() -> "image/gif" to "gif"
    b.size > 12 && String(b, 8, 4, Charsets.US_ASCII) == "WEBP" -> "image/webp" to "webp"
    else -> "image/jpeg" to "jpg"
}

/** Saves it to Pictures/Rubi; answers what to tell. */
private suspend fun savePicture(ctx: Context, url: String): String = withContext(Dispatchers.IO) {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) return@withContext "Android 10 以下暂不支持保存到相册"
    val f = pictureFile(ctx, url) ?: return@withContext "图片读取失败"
    val values = ContentValues().apply {
        put(MediaStore.Images.Media.DISPLAY_NAME, "rubi_${System.currentTimeMillis()}.${f.ext}")
        put(MediaStore.Images.Media.MIME_TYPE, f.mime)
        put(MediaStore.Images.Media.RELATIVE_PATH, "Pictures/Rubi")
        put(MediaStore.Images.Media.IS_PENDING, 1)
    }
    val resolver = ctx.contentResolver
    val uri = resolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, values) ?: return@withContext "保存失败"
    val ok = runCatching { resolver.openOutputStream(uri)?.use { it.write(f.bytes) } != null }.getOrDefault(false)
    if (!ok) {
        resolver.delete(uri, null, null)
        return@withContext "保存失败"
    }
    values.clear()
    values.put(MediaStore.Images.Media.IS_PENDING, 0)
    resolver.update(uri, values, null, null)
    "已保存到 相册 / Rubi"
}

/**
 * Hands the picture file to another app through the system share sheet. The copy sits in the
 * cache's shared folder, replaced by the next share and emptied at the next start.
 */
private suspend fun sharePicture(ctx: Context, url: String): String? {
    val f = pictureFile(ctx, url) ?: return "图片读取失败"
    val uri = withContext(Dispatchers.IO) {
        val dir = File(ctx.cacheDir, "shared/picture").apply { deleteRecursively(); mkdirs() }
        val file = File(dir, "rubi_${System.currentTimeMillis()}.${f.ext}")
        file.writeBytes(f.bytes)
        FileProvider.getUriForFile(ctx, "${ctx.packageName}.share", file)
    }
    val send = Intent(Intent.ACTION_SEND).apply {
        type = f.mime
        putExtra(Intent.EXTRA_STREAM, uri)
        addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
    }
    ctx.startActivity(Intent.createChooser(send, "分享图片").addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
    return null
}

private fun copyLink(ctx: Context, url: String) {
    val link = if (url.startsWith("//")) "https:$url" else url.replaceFirst("http://", "https://")
    val cm = ctx.getSystemService(ClipboardManager::class.java) ?: return
    cm.setPrimaryClip(ClipData.newPlainText("图片链接", link))
}
