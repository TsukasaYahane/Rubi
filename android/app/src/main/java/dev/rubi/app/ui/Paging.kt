package dev.rubi.app.ui

import android.os.SystemClock
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.grid.LazyGridState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import dev.rubi.core.RubiException
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch

/**
 * One page from a source: its items, the cursor of the page after it (null: this was the last)
 * and how many items the rules hid.
 */
class Page<T, C>(val items: List<T>, val next: C?, val hidden: UInt = 0u)

/**
 * A list loaded page by page. `fetch(null)` gives the first page, `fetch(cursor)` the ones after;
 * items already listed (by `key`) are not added twice. After an error it waits for [more] again.
 *
 * Loads run in `scope`, which should outlive the effects that ask for pages: those restart as
 * the list grows, and a restart must not cancel a page half way. By default the list has a
 * scope of its own, for state that outlives its screen (a page left for a video and come back
 * to). A [reset] drops any load still under way, so an old answer never lands in the new list.
 *
 * `minGapMs` spaces the pages after the first: scrolling fast then cannot fire a burst of
 * requests (which B站 answers with HTTP 412). A page asked for too soon follows once the gap
 * has passed.
 */
class Paged<T, C>(
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.Main),
    private val key: (T) -> Any,
    private val minGapMs: Long = 0L,
    private val fetch: suspend (cursor: C?) -> Page<T, C>,
) {
    val items = mutableStateListOf<T>()
    var loading by mutableStateOf(false)
        private set
    var ended by mutableStateOf(false)
        private set
    var error by mutableStateOf<Throwable?>(null)
        private set
    var hidden by mutableStateOf(0u)
        private set

    private var cursor: C? = null
    private var generation = 0
    private var lastStart = 0L
    private var waiting = false

    /** Loads the next page unless one is loading, the list ended, or the last one failed. */
    fun more() {
        if (loading || ended || error != null) return
        val wait = minGapMs - (SystemClock.elapsedRealtime() - lastStart)
        if (wait > 0) {
            if (!waiting) {
                waiting = true
                scope.launch {
                    delay(wait)
                    waiting = false
                    more()
                }
            }
            return
        }
        load()
    }

    /** Puts `item` in place of the listed one with the same key (e.g. a thread grown longer). */
    fun replace(item: T) {
        val at = items.indexOfFirst { key(it) == key(item) }
        if (at >= 0) items[at] = item
    }

    /** Tries again after an error. */
    fun retry() {
        error = null
        load()
    }

    /** Starts over from the first page. */
    fun reset() {
        generation++
        items.clear()
        cursor = null
        ended = false
        error = null
        hidden = 0u
        loading = false
        load()
    }

    private fun load() {
        if (loading || ended) return
        loading = true
        lastStart = SystemClock.elapsedRealtime()
        val gen = generation
        val at = cursor
        scope.launch {
            try {
                val page = fetch(at)
                if (gen != generation) return@launch
                items.addAll(page.items.filter { n -> items.none { key(it) == key(n) } })
                hidden += page.hidden
                cursor = page.next
                ended = page.next == null
                error = null
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                if (gen == generation) error = e
            } finally {
                if (gen == generation) loading = false
            }
        }
    }
}

/** Loads the next page whenever the last visible item comes within `ahead` of the end. */
@Composable
fun <T, C> LoadMoreNearEnd(list: LazyListState, paged: Paged<T, C>, ahead: Int = 4) {
    LaunchedEffect(list, paged) {
        snapshotFlow { (list.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0) >= paged.items.size - ahead }
            .collect { near -> if (near) paged.more() }
    }
}

/** The same for a grid. */
@Composable
fun <T, C> LoadMoreNearEnd(grid: LazyGridState, paged: Paged<T, C>, ahead: Int = 4) {
    LaunchedEffect(grid, paged) {
        snapshotFlow { (grid.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0) >= paged.items.size - ahead }
            .collect { near -> if (near) paged.more() }
    }
}

/** What went wrong, in words the user can act on. */
fun errorText(e: Throwable): String = when (e) {
    is RubiException.NotLoggedIn -> "登录后可查看"
    is RubiException.RiskControl -> "被 B 站风控拦截（请求过于频繁），稍等片刻再重试"
    else -> "加载失败：${e.message}"
}
