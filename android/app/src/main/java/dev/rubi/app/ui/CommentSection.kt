package dev.rubi.app.ui

import dev.rubi.core.FfiMention
import dev.rubi.core.RubiException
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.foundation.gestures.detectTapGestures
import android.widget.Toast
import dev.rubi.core.FfiCommentArea
import android.os.SystemClock
import android.util.Log
import android.content.Intent
import android.net.Uri
import android.view.LayoutInflater
import android.view.View
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.lazy.rememberLazyListState
import dev.rubi.core.FfiVideoDetail
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.layout.boundsInWindow
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.media3.ui.PlayerView
import dev.rubi.app.R
import dev.rubi.app.RubiApp
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import dev.rubi.app.data.DanmakuDisplay
import dev.rubi.app.data.PlaybackPrefs
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiCommentRef
import dev.rubi.core.FfiCommentRow
import dev.rubi.core.FfiCommentThread
import dev.rubi.core.FfiDanmakuSet
import dev.rubi.core.FfiPlayback
import dev.rubi.core.FfiVideoLink
import dev.rubi.core.FfiStein
import dev.rubi.core.FfiSteinNode
import dev.rubi.core.FfiSignal
import dev.rubi.core.FfiVideoCard
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/** Lines a comment shows before 展开全文. */
private const val COMMENT_LINES = 6

/**
 * The comments of one area (a video's, an article's…): sorted pages of fishbone threads, and
 * a long press on any comment for 点赞 / 回复 / 复制 / 分享.
 */
@Composable
internal fun CommentSection(
    app: RubiApp,
    area: FfiCommentArea,
    focus: FfiCommentRef?,
    onOpenVideo: (FfiVideoLink) -> Unit,
    onOpenArticle: (String) -> Unit,
    onFocusShown: () -> Unit,
) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    val ctx = LocalContext.current
    val clipboard = LocalClipboard.current
    var byTime by remember { mutableStateOf(false) }
    var pinned by remember { mutableStateOf(emptyList<FfiCommentRow>()) }
    var hiddenPinned by remember { mutableStateOf(0u) }
    val collapsed = remember { mutableStateMapOf<ULong, Boolean>() }
    // The thread holding the notice's comment, shown first and kept across sort changes.
    var focusThread by remember { mutableStateOf<FfiCommentThread?>(null) }
    var focusNote by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(area, focus) {
        if (focus == null) return@LaunchedEffect
        runCatching { app.core.focusThread(area, focus) }
            .onSuccess { focusThread = it; onFocusShown() }
            .onFailure { focusNote = "无法定位到消息里的评论（可能已被删除或折叠）" }
    }

    // Pages after the first follow the API's cursor, spaced so a fast scroll cannot fire a burst
    // (B站 answers those with HTTP 412). The first page also brings the pinned comments.
    val paged = remember(area) {
        Paged<FfiCommentThread, String>(scope, key = { it.rootRpid }, minGapMs = COMMENT_PAGE_GAP_MS) { at ->
            val page = app.core.comments(area, byTime, at)
            if (at == null) {
                pinned = page.pinned
                hiddenPinned = page.hiddenPinned
            }
            Page(page.threads, page.nextOffset?.takeIf { page.threads.isNotEmpty() })
        }
    }
    // A new sort starts over; so does tapping the active one again.
    LaunchedEffect(paged, byTime) { paged.reset() }
    val error = paged.error?.let(::commentError)

    // Likes changed here, by rpid (liked, count), until the list reloads.
    val likeOverride = remember { mutableStateMapOf<ULong, Pair<Boolean, ULong>>() }
    fun likeOf(row: FfiCommentRow) = likeOverride[row.rpid] ?: (row.liked to row.likes)
    // The comment a long press is acting on, with the root of its thread.
    var menuFor by remember { mutableStateOf<Pair<FfiCommentRow, ULong>?>(null) }
    var replyFor by remember { mutableStateOf<Pair<FfiCommentRow, ULong>?>(null) }
    var sending by remember { mutableStateOf(false) }
    var replyError by remember { mutableStateOf<String?>(null) }
    // 发表评论: a new floor under the content itself.
    var posting by remember { mutableStateOf(false) }
    var postError by remember { mutableStateOf<String?>(null) }
    fun say(msg: String) = Toast.makeText(ctx, msg, Toast.LENGTH_SHORT).show()

    fun replaceThread(t: FfiCommentThread) {
        if (t.rootRpid == focusThread?.rootRpid) focusThread = t else paged.replace(t)
    }

    /** A sent reply goes to the end of its thread at once, with the branch line drawn on to it. */
    fun addReply(root: ULong, reply: FfiCommentRow) {
        val thread = (listOfNotNull(focusThread) + paged.items).firstOrNull { it.rootRpid == root } ?: return
        val lastAtOne = thread.rows.indexOfLast { it.depth == 1u }
        val rows = thread.rows.mapIndexed { i, r -> if (i == lastAtOne) r.copy(hasLaterSibling = true) else r } + reply
        replaceThread(thread.copy(rows = rows, totalReplies = thread.totalReplies + 1u))
        collapsed[root] = false
    }

    fun act(row: FfiCommentRow, root: ULong, action: CommentAction) {
        menuFor = null
        when (action) {
            CommentAction.Like -> {
                val (was, count) = likeOf(row)
                val want = !was
                likeOverride[row.rpid] = want to if (want) count + 1u else count - minOf(count, 1uL)
                scope.launch {
                    runCatching { app.core.likeComment(area, row.rpid, want) }
                        .onSuccess { say(if (want) "已点赞" else "已取消点赞") }
                        .onFailure {
                            likeOverride[row.rpid] = was to count
                            say(if (it is RubiException.NotLoggedIn) "登录后才能点赞" else "点赞失败：${it.message}")
                        }
                }
            }
            CommentAction.Reply -> {
                replyError = null
                replyFor = row to root
            }
            CommentAction.Copy -> {
                scope.launch { clipboard.copyText(row.text) }
                say("已复制评论文字")
            }
            CommentAction.Share -> {
                scope.launch { clipboard.copyText(commentLink(area, row.rpid)) }
                say("已复制评论链接")
            }
        }
    }

    fun send(text: String, mentioned: List<FfiMention>) {
        val (row, root) = replyFor ?: return
        sending = true
        scope.launch {
            runCatching { app.core.replyComment(area, root, row.rpid, text, mentioned) }
                .onSuccess { r ->
                    // Replies to a reply say whom they answer, as the list shows them; B站's own
                    // "回复 @name :" lead-in is dropped so it is not shown twice.
                    addReply(root, if (row.depth > 0u) r.copy(replyTo = row.name, text = r.text.replaceFirst(REPLY_LEAD, "")) else r)
                    replyFor = null
                    say("已发送")
                }
                .onFailure { replyError = if (it is RubiException.NotLoggedIn) "登录后才能回复" else "发送失败：${it.message}" }
            sending = false
        }
    }

    /** A new comment goes on top of the list at once, as a thread of its own. */
    fun post(text: String, mentioned: List<FfiMention>) {
        sending = true
        scope.launch {
            runCatching { app.core.replyComment(area, 0u, 0u, text, mentioned) }
                .onSuccess { r ->
                    paged.items.add(0, FfiCommentThread(r.rpid, listOf(r), 0u, 0u))
                    posting = false
                    say("已发表")
                }
                .onFailure { postError = if (it is RubiException.NotLoggedIn) "登录后才能评论" else "发表失败：${it.message}" }
            sending = false
        }
    }

    if (posting) {
        PostComposer(areaNoun(area), sending, postError, remember { mentionsOf(app) }, onDismiss = { posting = false }, onSend = ::post)
    }
    menuFor?.let { (row, root) ->
        val (liked, likes) = likeOf(row)
        CommentMenu(row, liked, likes, onDismiss = { menuFor = null }) { act(row, root, it) }
    }
    replyFor?.let { (row, _) ->
        ReplyComposer(row.name, sending, replyError, remember(row.rpid) { mentionsOf(app) }, onDismiss = { replyFor = null }, onSend = ::send)
    }

    Column(Modifier.padding(horizontal = 16.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Meta("已隐藏置顶带货评论 $hiddenPinned 条", Modifier.weight(1f))
            listOf("按热度" to false, "按时间" to true).forEach { (label, t) ->
                Text(
                    label,
                    style = MetroType.meta.copy(fontSize = 12.sp),
                    color = if (byTime == t) c.fg else c.fg2,
                    fontWeight = if (byTime == t) FontWeight.SemiBold else FontWeight.Normal,
                    modifier = Modifier.padding(start = 16.dp).size(width = 56.dp, height = 44.dp).clickable {
                        if (byTime == t) paged.reset() else byTime = t
                    },
                )
            }
        }
        PostCommentEntry {
            postError = null
            posting = true
        }
        error?.let {
            Text(it, color = c.fg2, fontSize = 13.sp)
            if (paged.items.isEmpty()) TextAction("重试", onClick = paged::reset)
        }
        focusNote?.let { Meta(it) }
        pinned.forEach { row ->
            val (liked, likes) = likeOf(row)
            CommentRow(
                row,
                open = BooleanArray(0),
                liked = liked,
                likes = likes,
                pinnedLabel = true,
                onLink = onOpenVideo,
                onOpenNote = { onOpenArticle("cv$it") },
                onLongPress = { menuFor = row to row.rpid },
            )
        }
        val focused = focusThread
        val shownThreads = listOfNotNull(focused) + paged.items.filter { it.rootRpid != focused?.rootRpid }
        shownThreads.forEach { thread ->
            val isCollapsed = collapsed[thread.rootRpid] == true
            val rows = if (isCollapsed) thread.rows.take(1) else thread.rows
            Column(Modifier.padding(vertical = 10.dp)) {
                val open = BooleanArray(16)
                rows.forEach { row ->
                    // Snapshot: drawing happens later, after `open` has moved on.
                    val (liked, likes) = likeOf(row)
                    CommentRow(
                        row,
                        open.copyOf(),
                        liked = liked,
                        likes = likes,
                        highlight = thread === focused && row.rpid == focus?.rpid,
                        onLink = onOpenVideo,
                        onOpenNote = { onOpenArticle("cv$it") },
                        onLongPress = { menuFor = row to thread.rootRpid },
                    )
                    if (row.depth.toInt() in 1 until open.size) open[row.depth.toInt()] = row.hasLaterSibling
                }
                val shown = (thread.rows.size - 1).toUInt()
                val actions = buildList {
                    if (isCollapsed) add("展开 ${thread.totalReplies} 条回复" to { collapsed[thread.rootRpid] = false })
                    else {
                        if (thread.hidden > 0u) add("再看 5 条 · 剩 ${thread.hidden}" to {
                            scope.launch {
                                runCatching { app.core.commentThread(area, thread.rootRpid, shown + 5u) }
                                    .onSuccess { replaceThread(it) }
                            }
                            Unit
                        })
                        if (thread.rows.size > 1) add("折叠" to { collapsed[thread.rootRpid] = true })
                    }
                }
                if (actions.isNotEmpty()) {
                    Row(Modifier.padding(start = 22.dp), horizontalArrangement = Arrangement.spacedBy(20.dp)) {
                        actions.forEach { (label, act) -> TextAction(label, onClick = act) }
                    }
                }
            }
        }
        // The whole section is one list item, so the end is noticed by where this row is:
        // once it scrolls into the window, the next page loads.
        val screenH = with(LocalDensity.current) { LocalConfiguration.current.screenHeightDp.dp.toPx() }
        Column(
            Modifier.fillMaxWidth().padding(vertical = 16.dp).onGloballyPositioned { coords ->
                if (coords.boundsInWindow().top < screenH) paged.more()
            },
        ) {
            when {
                paged.loading -> Meta("评论加载中…")
                error != null && paged.items.isNotEmpty() -> TextAction("加载更多评论", onClick = paged::retry)
                paged.ended && paged.items.isNotEmpty() -> Meta("没有更多评论了")
                else -> {}
            }
        }
    }
}

/** Least time between two comment page requests. */
private const val COMMENT_PAGE_GAP_MS = 1_500L

private fun commentError(e: Throwable): String =
    if (e is dev.rubi.core.RubiException.RiskControl) "评论被 B 站风控拦截（请求过于频繁），稍等片刻再重试" else "评论加载失败：${e.message}"

/**
 * One fishbone row. `open[l]` tells whether the ancestor branch at depth `l` continues
 * below this row, so its vertical line is drawn through.
 */
@Composable
private fun CommentRow(
    row: FfiCommentRow,
    open: BooleanArray,
    liked: Boolean,
    likes: ULong,
    pinnedLabel: Boolean = false,
    highlight: Boolean = false,
    onLink: (FfiVideoLink) -> Unit = {},
    onOpenNote: (ULong) -> Unit = {},
    onLongPress: () -> Unit = {},
) {
    val c = LocalMetro.current
    val depth = row.depth.toInt()
    val indent = if (depth == 0) 0.dp else ((depth - 1) * 20 + 22).dp
    Column(
        Modifier.fillMaxWidth()
            .drawBehind {
                if (depth == 0) return@drawBehind
                val stroke = 1.dp.toPx()
                val tick = 16.dp.toPx()
                fun x(l: Int) = ((l - 1) * 20 + 6).dp.toPx()
                for (l in 1 until depth) {
                    if (l < open.size && open[l]) drawLine(c.spine, Offset(x(l), 0f), Offset(x(l), size.height), stroke)
                }
                val xd = x(depth)
                drawLine(c.spine, Offset(xd, 0f), Offset(xd, if (row.hasLaterSibling) size.height else tick), stroke)
                drawLine(c.spine, Offset(xd, tick), Offset(xd + 10.dp.toPx(), tick), stroke)
            }
            // Taps go to links and 展开全文 inside; a long press anywhere opens the actions.
            .pointerInput(row.rpid) { detectTapGestures(onLongPress = { onLongPress() }) }
            .padding(start = indent, top = 6.dp, bottom = 6.dp),
        verticalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            if (pinnedLabel) Tag("置顶")
            if (highlight) Tag("消息")
            Text(
                row.name,
                fontSize = if (depth == 0) 14.sp else 13.sp,
                fontWeight = FontWeight.SemiBold,
                color = if (highlight) c.accent else c.fg,
            )
            if (row.isUp) Tag("UP")
            if (row.noteCvid > 0u) Tag("笔记")
            Meta(formatAge(row.ctime))
        }
        // Long comments fold to six lines.
        var expanded by remember(row.rpid) { mutableStateOf(false) }
        var overflows by remember(row.rpid) { mutableStateOf(false) }
        EmoteText(
            (row.replyTo?.let { "回复 @$it：" } ?: "") + row.text,
            color = c.fg,
            fontSize = 14.sp,
            lineHeight = 21.sp,
            maxLines = if (expanded) Int.MAX_VALUE else COMMENT_LINES,
            extra = row.emotes,
            links = row.videoLinks,
            onLink = onLink,
            onTextLayout = { if (!expanded) overflows = it.hasVisualOverflow },
        )
        if (overflows || expanded) {
            Text(
                if (expanded) "收起" else "展开全文",
                fontSize = 13.sp,
                color = c.accent,
                modifier = Modifier.clickable { expanded = !expanded }.padding(vertical = 2.dp),
            )
        }
        if (row.pictures.isNotEmpty()) CommentPictures(row.pictures)
        if (row.noteCvid > 0u) TextAction("查看完整笔记", onClick = { onOpenNote(row.noteCvid) })
        Meta(if (liked) "已赞 $likes" else "赞 $likes", color = if (liked) c.accent else c.fg2)
    }
}

/** Who a comment can @: the people the account follows (friends included), by name. */
private fun mentionsOf(app: RubiApp) = Mentions { app.core.allFollowings().ups.map { FfiMention(it.name, it.mid) } }

private val REPLY_LEAD = Regex("""^回复 @[^:：]+[:：]\s*""")
