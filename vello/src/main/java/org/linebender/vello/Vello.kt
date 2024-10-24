package org.linebender.vello

import android.graphics.Color
import android.util.Log
import android.view.Surface
import android.view.SurfaceHolder

class VelloSurface internal constructor(
    val vello: Vello,
    val surface: Surface,
    width: Int,
    height: Int
) {

    var width: Int = width
        private set
    var height: Int = height
        private set

    fun resize(width: Int, height: Int) {
        this.width = width;
        this.height = height;
    }
}

/**
 * A global instance of a Vello renderer.
 */
class Vello {
    /** A pointer to the state of this renderer in Rust */
    private var state: Long = 0

    fun createSurface(surface: Surface, width: Int, height: Int): VelloSurface {
        return VelloSurface(this, surface, width, height)
    }

    /**
     * Set the surface this
     */
    fun setHolder(
        holder: SurfaceHolder
    ) {
        if (this.holder == holder) {
            return;
        }
        assert(this.holder == null, { "Cannot call setHolder more than once." })
        val self = this;
        holder.addCallback(object : SurfaceHolder.Callback {
            override fun surfaceCreated(holder: SurfaceHolder) {
                Log.i(null, "Setting surface");
                self.setSurface(self.state, holder.surface);
            }

            override fun surfaceChanged(
                holder: SurfaceHolder,
                format: Int,
                width: Int,
                height: Int
            ) {
                // TODO
            }

            override fun surfaceDestroyed(holder: SurfaceHolder) {
                Log.i(null, "unsetting surface");
                self.setSurface(self.state, null);
            }
        })
    }

    init {
        state = initialise()
    }

    fun cleanup() {
        state = 0
    }

    fun setColor(color: Color) {
        val color = color.toArgb();
        setColor(state, color);
    }

    @Suppress("KotlinJniMissingFunction") // This is defined in Rust code
    private external fun initialise(): Long

    @Suppress("KotlinJniMissingFunction") // This is defined in Rust code
    private external fun setColor(state: Long, color: Int)

    @Suppress("KotlinJniMissingFunction") // This is defined in Rust code
    private external fun setSurface(state: Long, surface: Surface?)

    companion object {
        // Used to load the 'vello' library on application startup.
        init {
            System.loadLibrary("vello_jni")
            Log.e(null, "Test loaded library")
            initRust();
        }

        /**
         * A native method to initialise Rust context (primarily related to logging).
         *
         * Calling this multiple times is permitted, and only the first call with have an effect.
         *
         * You might wish to customise the logging and stdout/stderr behaviour, so may not wish to call this.
         */
        @Suppress("KotlinJniMissingFunction") // This is defined in Rust code
        @JvmStatic
        external fun initRust()
    }
}
