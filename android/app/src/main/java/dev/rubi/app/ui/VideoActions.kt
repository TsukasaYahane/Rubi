package dev.rubi.app.ui

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.rubi.app.RubiApp
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiFavFolder
import dev.rubi.core.FfiVideoDetail
import dev.rubi.core.FfiVideoRelation
import dev.rubi.core.RubiException
import kotlinx.coroutines.launch

private enum class ActionSheet { None, Coin, Favorite }

/**
 * 赞 / 币 / 藏 / 稍后 / 关注 under the video title. Done actions are drawn in the accent colour;
 * every action needs a login, so a guest tap opens the login page instead. Long-pressing 赞 is
 * 一键三连 (like, two coins, default folder).
 */
@Composable
fun VideoActions(app: RubiApp, nav: Nav, d: FfiVideoDetail) {
    val scope = rememberCoroutineScope()
    val haptic = LocalHapticFeedback.current
    var rel by remember { mutableStateOf<FfiVideoRelation?>(null) }
    var loggedIn by remember { mutableStateOf(false) }
    var likes by remember { mutableStateOf(d.likes) }
    var coins by remember { mutableStateOf(d.coins) }
    var favorites by remember { mutableStateOf(d.favorites) }
    var later by remember { mutableStateOf(false) }
    var confirmUnfollow by remember { mutableStateOf(false) }
    var note by remember { mutableStateOf("") }
    var sheet by remember { mutableStateOf(ActionSheet.None) }

    // Re-checked when coming back from the login page.
    LaunchedEffect(d.bvid, nav.stack.size) {
        loggedIn = runCatching { app.core.account().loggedIn }.getOrDefault(false)
        if (loggedIn) rel = runCatching { app.core.relation(d.bvid) }.getOrNull()
    }

    fun failed(e: Throwable) {
        note = if (e is RubiException.NotLoggedIn) "登录已失效，请重新登录" else "操作失败：${e.message}"
    }

    /** Runs `block` only for a logged-in user whose relation is known. */
    fun act(block: (FfiVideoRelation) -> Unit) {
        val r = rel
        when {
            !loggedIn -> nav.push(Screen.Login)
            r == null -> note = "互动状态加载中…"
            else -> { note = ""; block(r) }
        }
    }

    Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
        Row(Modifier.fillMaxWidth()) {
            val r = rel
            val triple = {
                act { cur ->
                    if (cur.liked && cur.coins >= 2u && cur.favorited) {
                        note = "已三连"
                        return@act
                    }
                    haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                    note = "一键三连中…"
                    scope.launch {
                        runCatching { app.core.triple(d.bvid) }
                            .onSuccess { t ->
                                val was = rel ?: cur
                                if (t.liked && !was.liked) likes += 1u
                                if (t.favorited && !was.favorited) favorites += 1u
                                coins += t.coins.toULong()
                                rel = was.copy(
                                    liked = was.liked || t.liked,
                                    coins = was.coins + t.coins,
                                    favorited = was.favorited || t.favorited,
                                )
                                val missed = listOfNotNull(
                                    "点赞".takeUnless { t.liked },
                                    "投币".takeUnless { t.coined },
                                    "收藏".takeUnless { t.favorited },
                                )
                                note = if (missed.isEmpty()) {
                                    if (t.coins > 0u) "三连成功 · 投币 ${t.coins} 枚" else "三连成功（之前已投过币，这次未再投币）"
                                }
                                else "三连部分完成，未成功：" + missed.joinToString("、")
                            }
                            .onFailure { failed(it) }
                    }
                }
            }
            Action("赞", formatCount(likes), r?.liked == true, Modifier.weight(1f), onLongClick = triple) {
                act { cur ->
                    val on = !cur.liked
                    rel = cur.copy(liked = on)
                    likes = if (on) likes + 1u else likes - minOf(likes, 1uL)
                    scope.launch {
                        runCatching { app.core.like(d.bvid, on) }.onFailure {
                            rel = rel?.copy(liked = !on)
                            likes = d.likes
                            failed(it)
                        }
                    }
                }
            }
            Action("币", formatCount(coins), (r?.coins ?: 0u) > 0u, Modifier.weight(1f)) {
                act { cur -> if (cur.coins >= 2u) note = "已投满 2 枚硬币" else sheet = ActionSheet.Coin }
            }
            Action("藏", formatCount(favorites), r?.favorited == true, Modifier.weight(1f)) {
                act { sheet = ActionSheet.Favorite }
            }
            Action("稍后", if (later) "已加入" else "再看", later, Modifier.weight(1f)) {
                act {
                    if (later) return@act
                    later = true
                    scope.launch { runCatching { app.core.watchLater(d.bvid) }.onFailure { later = false; failed(it) } }
                }
            }
            Action("关注", if (r?.followingUp == true) "已关注" else "UP", r?.followingUp == true, Modifier.weight(1f)) {
                act { cur ->
                    // Unfollowing takes a second tap so a stray touch does not drop the UP.
                    if (cur.followingUp && !confirmUnfollow) {
                        confirmUnfollow = true
                        note = "再点一次取消关注 ${d.upName}"
                        return@act
                    }
                    confirmUnfollow = false
                    val on = !cur.followingUp
                    rel = cur.copy(followingUp = on)
                    scope.launch {
                        runCatching { app.core.followUp(d.bvid, on) }
                            .onSuccess { note = if (on) "已关注 ${d.upName}" else "已取消关注" }
                            .onFailure { rel = rel?.copy(followingUp = !on); failed(it) }
                    }
                }
            }
        }
        if (note.isNotEmpty()) Meta(note)
    }

    when (sheet) {
        ActionSheet.Coin -> CoinSheet(given = rel?.coins ?: 0u, liked = rel?.liked == true, onDismiss = { sheet = ActionSheet.None }) { n, alsoLike ->
            sheet = ActionSheet.None
            scope.launch {
                runCatching { app.core.coin(d.bvid, n, alsoLike) }
                    .onSuccess {
                        val likeNow = alsoLike && rel?.liked != true
                        rel = rel?.let { it.copy(coins = it.coins + n, liked = it.liked || alsoLike) }
                        coins += n.toULong()
                        if (likeNow) likes += 1u
                        note = "已投 $n 枚硬币"
                    }
                    .onFailure { failed(it) }
            }
        }
        ActionSheet.Favorite -> FavoriteSheet(app, d.bvid, onDismiss = { sheet = ActionSheet.None }) { add, remove, nowIn ->
            sheet = ActionSheet.None
            if (add.isEmpty() && remove.isEmpty()) return@FavoriteSheet
            scope.launch {
                runCatching { app.core.setFavorite(d.bvid, add, remove) }
                    .onSuccess {
                        val was = rel?.favorited == true
                        rel = rel?.copy(favorited = nowIn)
                        if (nowIn && !was) favorites += 1u
                        if (!nowIn && was) favorites -= minOf(favorites, 1uL)
                        note = if (nowIn) "已收藏" else "已取消收藏"
                    }
                    .onFailure { failed(it) }
            }
        }
        ActionSheet.None -> Unit
    }
}

/** A label over its count; heavier and in the accent colour once done. */
@OptIn(ExperimentalFoundationApi::class)
@Composable
private fun Action(
    label: String,
    value: String,
    done: Boolean,
    modifier: Modifier,
    onLongClick: (() -> Unit)? = null,
    onClick: () -> Unit,
) {
    val c = LocalMetro.current
    Column(
        modifier.heightIn(min = 52.dp).combinedClickable(onLongClick = onLongClick, onClick = onClick).padding(vertical = 6.dp),
        horizontalAlignment = Alignment.Start,
    ) {
        Text(
            label,
            fontSize = 17.sp,
            fontWeight = if (done) FontWeight.SemiBold else FontWeight.Light,
            color = if (done) c.accent else c.fg,
        )
        Meta(value, color = if (done) c.accent else c.fg2)
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun CoinSheet(given: UInt, liked: Boolean, onDismiss: () -> Unit, onConfirm: (UInt, Boolean) -> Unit) {
    val c = LocalMetro.current
    val max = 2u - given
    var count by remember { mutableStateOf(max) }
    var alsoLike by remember { mutableStateOf(!liked) }
    ModalBottomSheet(onDismissRequest = onDismiss, containerColor = c.bg2, shape = RectangleShape, dragHandle = null) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Text("投币", style = MetroType.section, color = c.fg)
            if (given > 0u) Meta("已投 $given 枚，还可投 $max 枚")
            Row(horizontalArrangement = Arrangement.spacedBy(28.dp)) {
                (1u..max).forEach { n ->
                    val on = n == count
                    Text(
                        "$n 枚",
                        fontSize = 28.sp,
                        fontWeight = if (on) FontWeight.SemiBold else FontWeight.Light,
                        color = if (on) c.accent else c.fg2,
                        modifier = Modifier.heightIn(min = 52.dp).clickable { count = n },
                    )
                }
            }
            if (!liked) Check("同时点赞", alsoLike) { alsoLike = it }
            Row(Modifier.padding(top = 8.dp, bottom = 12.dp)) { AccentButton("投 $count 枚") { onConfirm(count, alsoLike && !liked) } }
        }
    }
}

/**
 * Folder picker. `onConfirm` gets the folders to add to, to remove from, and whether the video
 * ends up in any folder.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun FavoriteSheet(
    app: RubiApp,
    bvid: String,
    onDismiss: () -> Unit,
    onConfirm: (List<ULong>, List<ULong>, Boolean) -> Unit,
) {
    val c = LocalMetro.current
    var folders by remember { mutableStateOf<List<FfiFavFolder>?>(null) }
    var error by remember { mutableStateOf<String?>(null) }
    val picked = remember { mutableStateMapOf<ULong, Boolean>() }
    LaunchedEffect(bvid) {
        runCatching { app.core.favFolders(bvid) }
            .onSuccess { list -> folders = list; list.forEach { picked[it.id] = it.contains } }
            .onFailure { error = "收藏夹加载失败：${it.message}" }
    }
    ModalBottomSheet(onDismissRequest = onDismiss, containerColor = c.bg2, shape = RectangleShape, dragHandle = null) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Text("收藏到", style = MetroType.section, color = c.fg)
            val list = folders
            when {
                error != null -> Meta(error!!)
                list == null -> Meta("加载中…")
                list.isEmpty() -> Meta("还没有收藏夹")
                else -> LazyColumn(Modifier.heightIn(max = 360.dp)) {
                    items(list, key = { it.id.toString() }) { f ->
                        Check("${f.title}  ·  ${f.count}", picked[f.id] == true) { picked[f.id] = it }
                    }
                }
            }
            Row(Modifier.padding(top = 8.dp, bottom = 12.dp)) {
                AccentButton("完成") {
                    val all = folders.orEmpty()
                    val add = all.filter { picked[it.id] == true && !it.contains }.map { it.id }
                    val remove = all.filter { picked[it.id] != true && it.contains }.map { it.id }
                    onConfirm(add, remove, all.any { picked[it.id] == true })
                }
            }
        }
    }
}

@Composable
private fun Check(label: String, on: Boolean, onChange: (Boolean) -> Unit) {
    val c = LocalMetro.current
    Row(
        Modifier.fillMaxWidth().heightIn(min = 44.dp).clickable { onChange(!on) },
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Text(if (on) "■" else "□", fontSize = 15.sp, color = if (on) c.accent else c.fg2)
        Text(label, fontSize = 15.sp, color = if (on) c.fg else c.fg2)
    }
}
