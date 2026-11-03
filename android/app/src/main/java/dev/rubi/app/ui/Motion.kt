package dev.rubi.app.ui

import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.foundation.Canvas
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.waitForUpOrCancellation
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.MotionDurationScale
import androidx.compose.ui.composed
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.boundsInWindow
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInWindow
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.IntOffset
import coil3.compose.AsyncImage
import dev.rubi.app.ui.theme.LocalMetro
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * UI 动效: speed `level` (0 off, then 慢 / 标准 / 快) and `style`, each the same for every page
 * opened or gone back to and every pivot switched:
 * - 0 默认: pages and pivots slide in from the side moved to while fading in; cards tilt;
 * - 1 卡牌: a tapped card spins about its own centre while growing to the screen; pages and pivots
 *   flip about the screen's centre line;
 * - 2 书页: a tapped card grows to the screen, then turns like a book page onto the video page;
 *   pages and pivots turn about the edge on the side being moved to.
 * Every motion is a non-linear curve; none run when off.
 */
data class Motion(val level: Int, val style: Int = 0) {
    val on: Boolean get() = level > 0

    /** `ms` at the 标准 speed, scaled for this one. */
    fun ms(ms: Int): Int = when (level) {
        1 -> ms * 2
        3 -> ms / 2
        else -> ms
    }

    companion object {
        val LABELS = listOf("无动效", "慢", "标准", "快")
        val STYLES = listOf("默认", "卡牌", "书页")
        const val DEFAULT = 0
        const val CARD = 1
        const val BOOK = 2
    }
}

val LocalMotion = staticCompositionLocalOf { Motion(2) }

/**
 * Animations here run at the speed chosen in 设置 whatever the system's animation scale: some
 * systems shorten every animation (on high refresh rate screens, or by the developer setting),
 * which made these faster than set. They are timed by the clock, not by frames.
 */
private object FixedSpeed : MotionDurationScale {
    override val scaleFactor: Float get() = 1f
}

/** Runs `block` at the speed set in Rubi, not the system's animation scale. */
suspend fun runFixed(block: suspend () -> Unit) = withContext(FixedSpeed) { block() }

/** Fast out, slow in: the Metro curve for anything arriving. */
val EnterEasing = CubicBezierEasing(0.2f, 0.8f, 0.25f, 1f)

/**
 * For a card flying to its page: eases in and out evenly, so the whole flight can be seen. (The
 * arriving curve above spends nearly all its movement in the first fifth, so a flight on it looked
 * equally quick at every speed.)
 */
val FlightEasing = CubicBezierEasing(0.45f, 0f, 0.15f, 1f)

/** Slow, fast, slow: for a page turning over. */
val TurnEasing = CubicBezierEasing(0.6f, 0f, 0.2f, 1f)

/** How long a page or pivot takes to arrive at 标准, in 默认 and 卡牌. */
private const val ARRIVE_MS = 340
/** How far 默认 slides a page or pivot in from. */
private const val SLIDE_DP = 60f

/**
 * A page arriving each time `key` changes: opened (`depth` grew) or gone back to. Only the
 * arriving page moves; the one left is gone at once (never two pages at a time, so a video page
 * leaving cannot stop the next one's player). Skipped while a card flight covers the change.
 */
@Composable
fun Modifier.turnstileIn(key: Any?, depth: Int, covered: () -> Boolean = { false }): Modifier {
    val motion = LocalMotion.current
    val turn = remember { Animatable(0f) }
    val last = remember { intArrayOf(depth) }
    val forward = remember { booleanArrayOf(true) }
    LaunchedEffect(key) {
        forward[0] = depth >= last[0]
        last[0] = depth
        if (!motion.on || covered() || motion.style == Motion.BOOK) return@LaunchedEffect
        withContext(FixedSpeed) {
            turn.snapTo(1f)
            turn.animateTo(0f, tween(motion.ms(ARRIVE_MS), easing = EnterEasing))
        }
    }
    return graphicsLayer {
        val sign = if (forward[0]) 1f else -1f
        when (motion.style) {
            Motion.CARD -> {
                rotationY = -90f * sign * turn.value
                transformOrigin = TransformOrigin.Center
            }
            // 书页 turns are drawn by PageTurns.
            Motion.BOOK -> {}
            // The same slide as a pivot: from the right going on, from the left going back.
            else -> translationX = sign * turn.value * SLIDE_DP * density
        }
        alpha = 1f - turn.value
        cameraDistance = 12f * density
    }
}

/** The arrival of pivot content, kept outside it so content composed anew still moves in. */
class PivotSlide internal constructor(internal val shift: Animatable<Float, *>, internal val motion: Motion)

/**
 * Pivot content arriving when `index` changes: 默认 slides in from the side picked; 卡牌 flips
 * in about the centre line; 书页 turns in about the edge on the side being moved to (drawn by
 * [PageTurns]; the switch goes through it).
 */
@Composable
fun rememberPivotSlide(index: Int): PivotSlide {
    val motion = LocalMotion.current
    val shift = remember { Animatable(0f) }
    val last = remember { intArrayOf(index) }
    LaunchedEffect(index) {
        val from = last[0]
        last[0] = index
        if (!motion.on || from == index || motion.style == Motion.BOOK) return@LaunchedEffect
        withContext(FixedSpeed) {
            shift.snapTo(if (index > from) 1f else -1f)
            shift.animateTo(0f, tween(motion.ms(ARRIVE_MS), easing = EnterEasing))
        }
    }
    return remember(motion) { PivotSlide(shift, motion) }
}

@Composable
fun Modifier.pivotSlide(index: Int): Modifier = pivotSlide(rememberPivotSlide(index))

fun Modifier.pivotSlide(slide: PivotSlide): Modifier {
    val motion = slide.motion
    val shift = slide.shift
    return graphicsLayer {
        val v = shift.value
        when (motion.style) {
            Motion.CARD -> {
                rotationY = 90f * v
                transformOrigin = TransformOrigin.Center
            }
            // 书页 turns are drawn by PageTurns.
            Motion.BOOK -> {}
            else -> translationX = v * SLIDE_DP * density
        }
        alpha = 1f - kotlin.math.abs(v)
        cameraDistance = 12f * density
    }
}

/** 磁贴倾斜: pressed, the card leans toward the finger and shrinks a little; let go, it settles. */
fun Modifier.pressTilt(): Modifier = composed {
    val motion = LocalMotion.current
    if (!motion.on) return@composed this
    val scope = rememberCoroutineScope()
    val tiltX = remember { Animatable(0f) }
    val tiltY = remember { Animatable(0f) }
    val press = remember { Animatable(0f) }
    this
        .pointerInput(Unit) {
            awaitEachGesture {
                val down = awaitFirstDown(requireUnconsumed = false)
                val fx = (down.position.x / size.width.coerceAtLeast(1) - 0.5f).coerceIn(-0.5f, 0.5f)
                val fy = (down.position.y / size.height.coerceAtLeast(1) - 0.5f).coerceIn(-0.5f, 0.5f)
                val inMs = motion.ms(120)
                scope.launch(FixedSpeed) { tiltY.animateTo(fx * 10f, tween(inMs, easing = EnterEasing)) }
                scope.launch(FixedSpeed) { tiltX.animateTo(-fy * 10f, tween(inMs, easing = EnterEasing)) }
                scope.launch(FixedSpeed) { press.animateTo(1f, tween(inMs, easing = EnterEasing)) }
                waitForUpOrCancellation()
                val outMs = motion.ms(220)
                scope.launch(FixedSpeed) { tiltY.animateTo(0f, tween(outMs, easing = EnterEasing)) }
                scope.launch(FixedSpeed) { tiltX.animateTo(0f, tween(outMs, easing = EnterEasing)) }
                scope.launch(FixedSpeed) { press.animateTo(0f, tween(outMs, easing = EnterEasing)) }
            }
        }
        .graphicsLayer {
            rotationX = tiltX.value
            rotationY = tiltY.value
            val s = 1f - 0.03f * press.value
            scaleX = s
            scaleY = s
            cameraDistance = 12f * density
        }
}

/** Where a tapped card's cover was on screen, the cover, and the video it opened (`key`), for the flight. */
data class CardOrigin(val bounds: Rect, val cover: String, val key: String = "")

/**
 * 卡牌 / 书页: the cover of the card that opened each page, by stack depth, so the page can grow
 * out of it and shrink back into it. A card sets [pending] right before it opens its page.
 */
class CardFlights {
    var pending: CardOrigin? = null
    private val origins = mutableMapOf<Int, CardOrigin>()
    /** The flight running: its origin, and whether it goes out to the page (true) or back. */
    var flight by mutableStateOf<Pair<CardOrigin, Boolean>?>(null)
    /**
     * Going out: how far the cover has grown (0 to 1). The page it opens fades in as the cover
     * fades out, and is all that shows once the cover fills the screen.
     */
    var progress by mutableFloatStateOf(0f)

    /** Where the video page's video frame is on screen, for 书页's cover to land on (and leave from). */
    var videoFrame by mutableStateOf<Rect?>(null)

    /** 书页 going back: a copy of the page being left, kept under the cover while it turns back in. */
    var backShot: ImageBitmap? = null

    /** The card a flight back is landing on; it keeps its cover hidden until the flight arrives. */
    fun landingOn(key: String): Boolean = flight?.let { (o, out) -> !out && o.key == key && key.isNotEmpty() } == true

    /**
     * Starts the flight back from `depth` pages before the page is gone, so no frame shows the
     * list with the card's cover still in place.
     */
    fun startBack(depth: Int) {
        val o = origins.remove(depth) ?: return
        flight = o to false
    }

    /** Whether going back from `depth` pages lands on a card that opened the top one. */
    fun hasOrigin(depth: Int): Boolean = origins.containsKey(depth)

    /** The stack went from `from` to `to` pages: start a flight out or back when a card is involved. */
    fun moved(from: Int, to: Int) {
        if (to > from) {
            val o = pending
            pending = null
            if (o != null) {
                origins[to] = o
                flight = o to true
            }
        } else if (to < from) {
            val o = origins[from]
            (to + 1..from).forEach { origins.remove(it) }
            if (o != null) flight = o to false
        }
    }
}

val LocalCardFlights = staticCompositionLocalOf { CardFlights() }

/** Remembers where this card's cover is, so tapping it can start a flight from there. */
fun Modifier.flightSource(onBounds: (Rect) -> Unit): Modifier = onGloballyPositioned { onBounds(it.boundsInWindow()) }

/**
 * The flying cover, over everything.
 * - 卡牌: the cover spins about its own centre while growing from the card to the screen, fading
 *   into the page as it goes (back: spins the other way while shrinking into the card).
 * - 书页: the page grows from the card to the screen, its cover moving to where the video sits on
 *   the video page, the rest background; then it turns up and away about the left edge of the
 *   screen, showing the video page. Back: the page turns in from the left edge, then shrinks
 *   into the card.
 * Going back the list shows beneath from the start, so the cover shrinks onto it.
 */
@Composable
fun CardFlightLayer(flights: CardFlights) {
    val motion = LocalMotion.current
    val run = flights.flight ?: return
    if (!motion.on || motion.style == Motion.DEFAULT) {
        LaunchedEffect(run) { flights.flight = null }
        return
    }
    val (origin, out) = run
    val c = LocalMetro.current
    val book = motion.style == Motion.BOOK
    val grow = remember(run) { Animatable(if (out) 0f else 1f) }
    val turn = remember(run) { Animatable(if (out) 0f else 1f) }
    var at by remember { mutableStateOf(Offset.Zero) }
    // 书页 going back: the page being left stays under the cover until the cover has turned in.
    val leftPage = remember(run) { if (out) null else flights.backShot.also { flights.backShot = null } }
    LaunchedEffect(run) {
        withContext(FixedSpeed) {
            when {
                !book ->
                    grow.animateTo(if (out) 1f else 0f, tween(motion.ms(if (out) 560 else 480), easing = FlightEasing)) {
                        if (out) flights.progress = value
                    }
                out -> {
                    grow.animateTo(1f, tween(motion.ms(380), easing = FlightEasing))
                    // The video page shows once the page starts turning away.
                    flights.progress = 1f
                    turn.animateTo(1f, tween(motion.ms(520), easing = TurnEasing))
                }
                else -> {
                    turn.animateTo(0f, tween(motion.ms(520), easing = TurnEasing))
                    grow.animateTo(0f, tween(motion.ms(380), easing = FlightEasing))
                }
            }
        }
        flights.progress = 0f
        flights.flight = null
    }
    BoxWithConstraints(Modifier.fillMaxSize().onGloballyPositioned { at = it.positionInWindow() }) {
        val density = LocalDensity.current
        val fullW = constraints.maxWidth.toFloat()
        val fullH = constraints.maxHeight.toFloat()
        val g = grow.value
        fun lerp(a: Float, b: Float) = a + (b - a) * g
        // The sheet: from the card to the whole screen.
        val x = lerp(origin.bounds.left - at.x, 0f)
        val y = lerp(origin.bounds.top - at.y, 0f)
        val w = lerp(origin.bounds.width, fullW)
        val h = lerp(origin.bounds.height, fullH)
        // 书页: the cover, from the card to the video frame of the video page (or the top, full width).
        val frame = flights.videoFrame
        val endX = frame?.left?.minus(at.x) ?: 0f
        val endY = frame?.top?.minus(at.y) ?: 0f
        val endW = frame?.width ?: fullW
        val endH = frame?.height ?: (fullW * origin.bounds.height / origin.bounds.width.coerceAtLeast(1f))
        val coverX = lerp(origin.bounds.left - at.x, endX) - x
        val coverY = lerp(origin.bounds.top - at.y, endY) - y
        val coverW = lerp(origin.bounds.width, endW)
        val coverH = lerp(origin.bounds.height, endH)
        if (book && !out && leftPage != null && turn.value > 0f) {
            Canvas(Modifier.fillMaxSize()) { drawImage(leftPage, topLeft = Offset(-at.x, -at.y)) }
        }
        Box(
            Modifier
                .offset { IntOffset(x.toInt(), y.toInt()) }
                .size(with(density) { w.toDp() }, with(density) { h.toDp() })
                .graphicsLayer {
                    if (book) {
                        // Hinged on the left edge of the screen, turned up toward the viewer.
                        rotationY = -90f * turn.value
                        transformOrigin = TransformOrigin(0f, 0.5f)
                    } else {
                        rotationY = -360f * g
                        transformOrigin = TransformOrigin.Center
                        if (out) alpha = 1f - flights.progress
                    }
                    cameraDistance = 16f * density.density
                }
                .background(if (book) c.bg else c.bg2),
        ) {
            if (book) {
                AsyncImage(
                    model = origin.cover,
                    contentDescription = null,
                    contentScale = ContentScale.Crop,
                    modifier = Modifier
                        .offset { IntOffset(coverX.toInt(), coverY.toInt()) }
                        .size(with(density) { coverW.toDp() }, with(density) { coverH.toDp() }),
                )
            } else {
                AsyncImage(model = origin.cover, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
            }
        }
    }
}
