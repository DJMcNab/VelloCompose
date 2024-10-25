package org.linebender.vello

import android.view.Surface
import androidx.compose.runtime.MonotonicFrameClock
import androidx.compose.ui.platform.AndroidUiDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

/**
 * The base class for a Surface controlled by Vello.
 *
 * Note that in the current version of this library, the scenes it is possible to create are
 * limited.
 * This is because we do not have an FFI stable interface to Vello, so for efficiency we limit the
 * supported kinds of scenes.
 *
 * This type has manual memory management. [cleanUp] must be called when you are finished with this.
 */
class VelloSurface internal constructor(
    val surface: Surface,
    val vello: Vello,
    internal val id: Long,
    width: Int,
    height: Int
) {
    var width: Int = width
        private set
    var height: Int = height
        private set

    private var onResize: ((width: Int, height: Int) -> Unit)? = null

    fun onResize(func: (width: Int, height: Int) -> Unit) {
        onResize = func
    }

    fun resize(width: Int, height: Int) {
        this.width = width;
        this.height = height;
        onResize?.invoke(width, height)
        // TODO: Reconfigure the underlying wgpu `Surface`
        // TODO: Rerender this surface immediately
        // This does mean that work can't be shared, but that's OK
    }

    fun cleanUp() {
        // TODO: Schedule this surface for cleanup
    }

    fun onRender(onFrame: (Long) -> Unit) {
        vello.callbacks.add(Callback(onFrame, id))
    }

    // TODO: Consider a Debug finalise to ensure that cleanup is called?
}

internal data class Callback(val callback: (Long) -> Unit, val surfaceId: Long)

/**
 * A [VelloSurface] supporting variable font input.
 *
 * Currently only supports Roboto Flex.
 */
class VariableFontSurface(
    val surface: VelloSurface,
    text: String,
    fontSize: Float,
    fontWeight: Float = 400f
) {
    var fontSize = fontSize
        private set
    var fontWeight = fontWeight
        private set
    var text: String = text
        private set

    private var textChanged = false
    private var renderScheduled = false

    fun setWeight(weight: Float) {
        fontWeight = weight
        scheduleRender()
    }

    fun setFontSize(size: Float) {
        fontSize = size
        scheduleRender()
    }

    fun setText(newText: String) {
        text = newText;
        textChanged = true
        scheduleRender()
    }

    init {
        surface.vello.makeVariableFontSurface(surface.id, text, fontSize, fontWeight);
    }

    private fun scheduleRender() {
        if (renderScheduled) return
        surface.onRender { _ ->
            renderScheduled = false
            // Marshalling text is potentially expensive, so only do it if needed.
            if (textChanged) {
                surface.vello.updateVariableFontText(surface.id, text)
            }
            surface.vello.updateVariableFontParameters(surface.id, fontSize, fontWeight)
        }
        renderScheduled = true
    }

}

/**
 * A global instance of a Vello renderer.
 */
// TODO: The rules of thread safety in Java are a bit unclear to me. This might need a whole
// sprinkling of synchronised(lock)
class Vello(private val coroutineScope: CoroutineScope) {
    /** A pointer to the state of this renderer in Rust */
    private var state: Long = 0

    // We reserve the zero id for "not a surface"
    private var nextSurfaceId: Long = 1;

    internal var callbacks = mutableListOf<Callback>()
    internal var scratchCallbacks = mutableListOf<Callback>()

    fun createSurface(surface: Surface, width: Int, height: Int): VelloSurface {
        val id = nextSurfaceId
        // Overflow handling: long, so overflow implausible
        nextSurfaceId += 1
        newSurface(state, surface, id, width, height)
        return VelloSurface(surface, this, nextSurfaceId, width, height)
    }

    init {
        state = initialise()
        coroutineScope.launch {
            mainLoop()
        }
    }

    suspend fun mainLoop() {
        val clock = AndroidUiDispatcher.Main[MonotonicFrameClock]
        assert(clock != null)
        if (clock != null) {
            var updatedSurfaces = LongArray(20)
            while (true) {
                // TODO: Are there any locking requirements?
                // TODO: If nothing to do, don't do anything
                clock.withFrameNanos { frameTimeNanos ->
                    val localCallbacks = callbacks
                    callbacks = scratchCallbacks
                    scratchCallbacks = localCallbacks
                    while (localCallbacks.size > updatedSurfaces.size) {
                        var newSize = updatedSurfaces.size * 2;
                        if (localCallbacks.size > newSize) {
                            newSize = localCallbacks.size + updatedSurfaces.size
                        }
                        updatedSurfaces = LongArray(newSize)
                    }
                    for (i in updatedSurfaces.indices) {
                        if (updatedSurfaces[i] == 0L) {
                            break
                        }
                        updatedSurfaces[i] = 0
                    }
                    for (i in 0 until localCallbacks.size) {
                        // TODO: Maybe callback can disable update for this Surface?
                        localCallbacks[i].callback(frameTimeNanos)
                        updatedSurfaces[i] = localCallbacks[i].surfaceId
                    }
                    doRender(state, updatedSurfaces, localCallbacks.size)
                    localCallbacks.clear()
                }
            }
        }
    }

    fun cleanup() {
        // TODO: Deallocate on the Rust side.
        // TODO: Defer? cleanup until all managed VelloSurfaces are cleaned up.
        val oldState = state
        state = 0
    }

    // This is defined in Rust code, so Kotlin doesn't know about it
    @Suppress("KotlinJniMissingFunction")
    private external fun initialise(): Long

    @Suppress("KotlinJniMissingFunction")
    private external fun newSurface(
        state: Long,
        surface: Surface,
        surfaceId: Long,
        width: Int,
        height: Int
    )

    @Suppress("KotlinJniMissingFunction")
    private external fun doRender(
        state: Long,
        updatedSurfaces: LongArray,
        updatedSurfacesCount: Int
    )

    @Suppress("KotlinJniMissingFunction")
    private external fun makeVariableFontSurface(
        state: Long,
        surfaceId: Long,
        text: String,
        fontSize: Float,
        fontWeight: Float
    )

    internal fun makeVariableFontSurface(
        surfaceId: Long,
        text: String,
        fontSize: Float,
        fontWeight: Float
    ) {
        makeVariableFontSurface(state, surfaceId, text, fontSize, fontWeight)
    }

    @Suppress("KotlinJniMissingFunction")
    private external fun updateVariableFontParameters(
        state: Long,
        surfaceId: Long,
        fontSize: Float,
        fontWeight: Float
    )

    internal fun updateVariableFontParameters(surfaceId: Long, fontSize: Float, fontWeight: Float) {
        updateVariableFontParameters(state, surfaceId, fontSize, fontWeight)
    }

    @Suppress("KotlinJniMissingFunction")
    private external fun updateVariableFontText(state: Long, surfaceId: Long, text: String)

    internal fun updateVariableFontText(surfaceId: Long, text: String) {
        updateVariableFontText(state, surfaceId, text)
    }

    companion object {
        // Used to load the 'vello' library on application startup.
        init {
            System.loadLibrary("vello_jni")
        }

        /**
         * A native method to initialise Rust context (primarily related to logging).
         *
         * Calling this multiple times is permitted, and only the first call with have an effect.
         *
         * You might wish to customise the stdout/stderr behaviour, so may not wish to
         * call this.
         */
        @Suppress("KotlinJniMissingFunction") // This is defined in Rust code
        @JvmStatic
        external fun initRust()
    }
}
