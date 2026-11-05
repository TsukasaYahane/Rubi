package dev.rubi.app.ui

import dev.rubi.core.VideosService
import dev.rubi.core.FollowingService
import dev.rubi.core.CommentsService
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.core.FfiDynamic

/** A dynamic's comment area: the post in brief, then its comments (and 发表评论). */
@Composable
fun DynamicCommentsScreen(videos: VideosService, commentsService: CommentsService, following: FollowingService, nav: Nav, post: FfiDynamic) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    val area = post.comments ?: return
    Column(Modifier.fillMaxSize()) {
        PageHeader(nav, "动态评论")
        LazyColumn(Modifier.weight(1f), contentPadding = PaddingValues(bottom = 24.dp)) {
            item(key = "post") {
                Column(Modifier.padding(horizontal = 16.dp, vertical = 8.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text(post.body.authorName, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = c.fg)
                        Meta(formatAge(post.publishedAt))
                    }
                    val text = post.body.title.ifBlank { post.body.text.ifBlank { post.orig?.text.orEmpty() } }
                    if (text.isNotBlank()) Text(text, fontSize = 14.sp, lineHeight = 21.sp, color = c.fg, maxLines = 4, overflow = TextOverflow.Ellipsis)
                }
            }
            item(key = "comments") {
                CommentSection(
                    commentsService, following,
                    area,
                    focus = null,
                    onOpenVideo = { v -> nav.openVideo(scope, videos, v.video, v.title) },
                    onOpenArticle = { nav.push(Screen.Article(it)) },
                    onFocusShown = {},
                )
            }
        }
    }
}
