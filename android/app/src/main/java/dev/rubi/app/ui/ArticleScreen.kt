package dev.rubi.app.ui

import android.content.Intent
import android.net.Uri
import android.widget.Toast
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
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
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import dev.rubi.app.RubiApp
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.core.FfiArticle
import dev.rubi.core.FfiArticleBlock
import dev.rubi.core.FfiPicture
import kotlinx.coroutines.launch

private const val MIN_SP = 13
private const val MAX_SP = 22

/**
 * A 专栏 / 图文 read inside Rubi: cover, title, author, then the body in Rubi's own type
 * (size set with A− / A+ and remembered). Its comments follow the body; the bottom bar jumps
 * there, opens the original in the browser or copies its link.
 */
@Composable
fun ArticleScreen(app: RubiApp, nav: Nav, link: String) {
    val c = LocalMetro.current
    val ctx = LocalContext.current
    val clipboard = LocalClipboardManager.current
    val scope = rememberCoroutineScope()
    var article by remember { mutableStateOf<FfiArticle?>(null) }
    var error by remember { mutableStateOf<Throwable?>(null) }
    var round by remember { mutableStateOf(0) }
    var size by remember { mutableStateOf(app.ui.articleSp.coerceIn(MIN_SP, MAX_SP)) }
    var viewer by remember { mutableStateOf<Pair<List<FfiPicture>, Int>?>(null) }
    val list = rememberLazyListState()

    LaunchedEffect(link, round) {
        error = null
        runCatching { app.core.article(link) }.onSuccess { article = it }.onFailure { error = it }
    }
    fun resize(by: Int) {
        size = (size + by).coerceIn(MIN_SP, MAX_SP)
        app.ui.articleSp = size
    }
    val pageUrl = article?.let { "https://www.bilibili.com/opus/${it.id}" }

    Column(Modifier.fillMaxSize()) {
        Row(Modifier.fillMaxWidth().height(48.dp).padding(end = 8.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(44.dp).clickable { nav.pop() }, contentAlignment = Alignment.Center) { GlyphIcon(Glyph.Back) }
            Text("专栏", fontSize = 13.sp, color = c.fg2, modifier = Modifier.weight(1f))
            Text("A−", fontSize = 14.sp, color = c.fg, modifier = Modifier.clickable { resize(-1) }.padding(horizontal = 10.dp, vertical = 12.dp))
            Text("A+", fontSize = 18.sp, color = c.fg, modifier = Modifier.clickable { resize(1) }.padding(horizontal = 10.dp, vertical = 10.dp))
        }
        Box(Modifier.fillMaxWidth().height(1.dp).background(c.line))
        val a = article
        if (a == null) {
            Column(Modifier.weight(1f).padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                val e = error
                if (e == null) Meta("加载中…") else {
                    Meta(errorText(e))
                    TextAction("重试", onClick = { round++ })
                    TextAction("在浏览器打开", onClick = { open(ctx, link) }, dim = true)
                }
            }
            return@Column
        }
        LazyColumn(Modifier.weight(1f), state = list, contentPadding = PaddingValues(bottom = 24.dp)) {
            if (a.cover.isNotEmpty()) {
                item(key = "cover") {
                    AsyncImage(model = a.cover, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxWidth().aspectRatio(16f / 9f).background(c.bg2))
                }
            }
            item(key = "head") {
                Column(Modifier.padding(start = 18.dp, end = 18.dp, top = 16.dp, bottom = 8.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text(a.title, fontSize = (size + 8).sp, lineHeight = ((size + 8) * 1.35f).sp, fontWeight = FontWeight.SemiBold, color = c.fg)
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                        AsyncImage(model = a.authorFace, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.size(32.dp).background(c.bg2))
                        Column(Modifier.clickable { nav.push(Screen.Up(a.authorMid, a.authorName)) }) {
                            Text(a.authorName, fontSize = 14.sp, color = c.fg)
                            Meta("${formatAge(a.publishedAt)} · 阅读约 ${readingMinutes(a)} 分钟")
                        }
                    }
                }
            }
            itemsIndexed(a.blocks, key = { i, _ -> "b$i" }) { _, block ->
                Box(Modifier.padding(horizontal = 18.dp)) {
                    ArticleBlockView(block, size) { pics, i -> viewer = pics to i }
                }
            }
            if (a.hidden > 0u) item(key = "hidden") { Meta("本文已过滤推广卡片 ${a.hidden} 个", Modifier.padding(horizontal = 18.dp, vertical = 8.dp)) }
            a.comments?.let { area ->
                item(key = "comments-title") { SectionTitle("评论") }
                item(key = "comments") {
                    CommentSection(
                        app,
                        area,
                        focus = null,
                        onOpenVideo = { v -> nav.openVideo(scope, app, v.video, v.title) },
                        onOpenArticle = { nav.push(Screen.Article(it)) },
                        onFocusShown = {},
                    )
                }
            }
        }
        Box(Modifier.fillMaxWidth().height(1.dp).background(c.line))
        Row(Modifier.fillMaxWidth().height(48.dp), verticalAlignment = Alignment.CenterVertically) {
            val commentsAt = a.blocks.size + (if (a.cover.isNotEmpty()) 2 else 1) + (if (a.hidden > 0u) 1 else 0)
            BarAction("评论", Modifier.weight(1f), enabled = a.comments != null) { scope.launch { list.animateScrollToItem(commentsAt) } }
            BarAction("在浏览器打开", Modifier.weight(1f)) { pageUrl?.let { open(ctx, it) } }
            BarAction("复制链接", Modifier.weight(1f)) {
                pageUrl?.let { clipboard.setText(AnnotatedString(it)) }
                Toast.makeText(ctx, "已复制链接", Toast.LENGTH_SHORT).show()
            }
        }
    }
    viewer?.let { (pics, start) -> PictureViewer(pics, start) { viewer = null } }
}

@Composable
private fun BarAction(label: String, modifier: Modifier, enabled: Boolean = true, onClick: () -> Unit) {
    val c = LocalMetro.current
    Box(modifier.height(48.dp).clickable(enabled = enabled, onClick = onClick), contentAlignment = Alignment.Center) {
        Text(label, fontSize = 13.sp, color = if (enabled) c.fg else c.fg2)
    }
}

/** One block of the body at body size `sp`; pictures open the viewer on the one tapped. */
@Composable
private fun ArticleBlockView(block: FfiArticleBlock, sp: Int, onPicture: (List<FfiPicture>, Int) -> Unit) {
    val c = LocalMetro.current
    when (block) {
        is FfiArticleBlock.Heading -> Text(
            block.text,
            fontSize = (sp + 3).sp,
            lineHeight = ((sp + 3) * 1.5f).sp,
            fontWeight = FontWeight.SemiBold,
            color = c.fg,
            modifier = Modifier.padding(top = 18.dp, bottom = 8.dp),
        )
        is FfiArticleBlock.Text -> EmoteText(block.text, color = c.fg, fontSize = sp.sp, lineHeight = (sp * 1.75f).sp, modifier = Modifier.padding(bottom = 14.dp))
        is FfiArticleBlock.Quote -> Row(Modifier.padding(bottom = 14.dp).height(IntrinsicSize.Min)) {
            Box(Modifier.width(3.dp).fillMaxHeight().background(c.accent))
            Text(block.text, fontSize = sp.sp, lineHeight = (sp * 1.75f).sp, color = c.fg2, modifier = Modifier.padding(start = 12.dp))
        }
        is FfiArticleBlock.Pictures -> Column(Modifier.padding(bottom = 14.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            block.pictures.forEachIndexed { i, p ->
                val ratio = if (p.width > 0u && p.height > 0u) (p.width.toFloat() / p.height.toFloat()).coerceIn(0.3f, 3f) else 16f / 9f
                AsyncImage(
                    model = p.url,
                    contentDescription = null,
                    contentScale = ContentScale.Crop,
                    modifier = Modifier.fillMaxWidth().aspectRatio(ratio).background(c.bg2).clickable { onPicture(block.pictures, i) },
                )
            }
        }
        FfiArticleBlock.Line -> Box(Modifier.padding(vertical = 18.dp).fillMaxWidth().height(1.dp).background(c.line))
    }
}

/** About 400 characters a minute, at least one. */
private fun readingMinutes(a: FfiArticle): Int {
    val chars = a.blocks.sumOf { b ->
        when (b) {
            is FfiArticleBlock.Text -> b.text.length
            is FfiArticleBlock.Heading -> b.text.length
            is FfiArticleBlock.Quote -> b.text.length
            else -> 0
        }
    }
    return maxOf(1, chars / 400)
}

private fun open(ctx: android.content.Context, url: String) {
    val full = if (url.startsWith("//")) "https:$url" else url
    runCatching { ctx.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(full))) }
}
