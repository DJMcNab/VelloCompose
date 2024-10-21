package org.linebender.vello

class NativeLib {

    /**
     * A native method that is implemented by the 'vello' native library,
     * which is packaged with this application.
     */
    external fun stringFromJNI(): String

    companion object {
        // Used to load the 'vello' library on application startup.
        init {
            System.loadLibrary("vello")
        }
    }
}