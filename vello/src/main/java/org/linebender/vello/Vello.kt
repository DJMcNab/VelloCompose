package org.linebender.vello

import android.util.Log
import android.view.Surface
import android.view.SurfaceHolder
import dalvik.annotation.optimization.CriticalNative
import dalvik.annotation.optimization.FastNative

class Vello {
    private var holder: SurfaceHolder? = null
    private var state: Long = 0

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
                TODO("Not yet implemented")
            }


        })
    }

    init {
        state = initialise()
    }

    fun cleanup() {
        state = 0
    }

    //    @Suppress("KotlinJniMissingFunction") // This is defined in Rust code
    private external fun initialise(): Long

    @Suppress("KotlinJniMissingFunction") // This is defined in Rust code
    private external fun setColor(state: Long, color: Int)

    private external fun setSurface(state: Long, surface: Surface)

    companion object {
        // Used to load the 'vello' library on application startup.
        init {
            System.loadLibrary("vello")
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
//        @Suppress("KotlinJniMissingFunction") // This is defined in Rust code
        @JvmStatic
        external fun initRust()
    }
}
