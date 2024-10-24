package org.linebender.vello.compose

import android.util.Log
import androidx.compose.foundation.AndroidExternalSurface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.remember
import androidx.compose.runtime.staticCompositionLocalOf
import org.linebender.vello.Vello

private val LocalVello = staticCompositionLocalOf<Vello?> { null }

@Composable
fun VelloContext(allowNested: Boolean = false, content: @Composable() () -> Unit) {
    if (!allowNested && LocalVello.current != null) {
        throw IllegalStateException(
            "Tried to nest `VelloContext`s." +
                    "Enable `allowNested` on the inner `VelloContext` if this is intentional"
        )
    }
    val vello = remember { Vello() }
    CompositionLocalProvider(LocalVello provides vello) { }
}

@Composable
fun VelloSurface() {
    val vello = LocalVello.current
        ?: throw IllegalStateException("Tried to use Vello outside of a `VelloContext`");
    AndroidExternalSurface {
        onSurface { surface, width, height ->
            val vello_surface = vello.createSurface(surface, width, height)
            surface.onChanged { width, height ->
                vello_surface.resize
            }
            surface.onDestroyed {

            }
        }
    }
}