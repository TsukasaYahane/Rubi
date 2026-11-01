package dev.rubi.app.ui

import dev.rubi.app.playback.LocalMedia
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.media3.common.Player
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.core.FfiSteinChoice
import dev.rubi.core.FfiSteinNode
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

/**
 * 互动视频 choices over the player. When the node's part nears its end the question shows;
 * a question without visible options (type 0) or one whose time runs out takes its default
 * choice. An ending offers to start the story again. These come from the story graph, not from
 * danmaku, so no danmaku filter can hide them.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun SteinOverlay(
    node: FfiSteinNode,
    playingCid: ULong?,
    nodeCid: ULong?,
    onChosen: (FfiSteinNode, ULong) -> Unit,
    onRestart: () -> Unit,
    choose: suspend (ULong) -> FfiSteinNode,
    onError: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val c = LocalMetro.current
    val media = LocalMedia.current
    val scope = rememberCoroutineScope()
    var showing by remember(node) { mutableStateOf(false) }
    var ended by remember(node) { mutableStateOf(false) }
    var busy by remember(node) { mutableStateOf(false) }
    val q = node.question

    fun pick(choice: FfiSteinChoice) {
        if (busy) return
        busy = true
        showing = false
        scope.launch {
            runCatching { choose(choice.edgeId) }
                .onSuccess { onChosen(it, choice.cid) }
                .onFailure { busy = false; onError("互动视频跳转失败：${it.message}") }
        }
    }

    fun default(): FfiSteinChoice? = q?.choices?.firstOrNull { it.isDefault } ?: q?.choices?.firstOrNull()

    // Wait for this node's part to be the one loaded, then for its end to come near.
    LaunchedEffect(node, playingCid, nodeCid) {
        if (playingCid == null || playingCid != nodeCid) return@LaunchedEffect
        val player = media.player.get()
        while (true) {
            val state = player.playbackState
            val dur = player.duration
            val near = q != null && dur > 0 && dur - player.currentPosition <= q.beforeEndMs.toLong()
            if (state == Player.STATE_ENDED || (state == Player.STATE_READY && near)) break
            delay(200)
        }
        if (q == null) {
            ended = true
            return@LaunchedEffect
        }
        if (q.kind.toInt() == 0) {
            // Nothing to show: move on once the part is over.
            while (player.playbackState != Player.STATE_ENDED) delay(200)
            default()?.let(::pick)
            return@LaunchedEffect
        }
        showing = true
        q.timeLimitMs?.let { limit ->
            delay(limit.toLong())
            if (showing) default()?.let(::pick)
        }
    }

    // A question that does not wait takes its default once the part is over unanswered; one
    // that waits keeps the last frame (the part has ended) until a choice is made.
    LaunchedEffect(showing) {
        if (!showing || q == null || q.pause) return@LaunchedEffect
        val player = media.player.get()
        while (showing && player.playbackState != Player.STATE_ENDED) delay(200)
        if (showing) default()?.let(::pick)
    }

    if (!showing && !ended) return
    // Centred over the picture, like the official player.
    Column(
        modifier.padding(horizontal = 24.dp).background(Color(0xB3000000)).padding(horizontal = 16.dp, vertical = 12.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        if (node.vars.isNotEmpty()) {
            Text(node.vars.joinToString("  ") { "${it.name} ${formatVar(it.value)}" }, fontSize = 12.sp, color = Color(0xCCFFFFFF))
        }
        if (showing && q != null) {
            if (q.title.isNotBlank()) Text(q.title, fontSize = 15.sp, fontWeight = FontWeight.SemiBold, color = Color.White)
            FlowRow(
                horizontalArrangement = Arrangement.spacedBy(10.dp, Alignment.CenterHorizontally),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                q.choices.forEach { choice ->
                    Text(
                        choice.option,
                        fontSize = 15.sp,
                        color = Color.White,
                        modifier = Modifier.background(c.accent).clickable { pick(choice) }
                            .heightIn(min = 40.dp).padding(horizontal = 16.dp, vertical = 10.dp),
                    )
                }
            }
        } else if (ended) {
            Text("已到达结局：${node.title}", fontSize = 15.sp, color = Color.White)
            Text(
                "从头再来",
                fontSize = 15.sp,
                color = Color.White,
                modifier = Modifier.background(c.accent).clickable(onClick = onRestart)
                    .heightIn(min = 40.dp).padding(horizontal = 16.dp, vertical = 10.dp),
            )
        }
    }
}

private fun formatVar(v: Double): String = if (v % 1.0 == 0.0) v.toLong().toString() else "%.1f".format(v)
