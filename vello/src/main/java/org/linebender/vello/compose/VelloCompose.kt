package org.linebender.vello.compose

import androidx.compose.foundation.AndroidExternalSurface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Modifier
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.selects.select
import org.linebender.vello.VariableFontSurface
import org.linebender.vello.Vello
import org.linebender.vello.VelloSurface
import java.util.concurrent.ArrayBlockingQueue
import java.util.concurrent.BlockingQueue
import kotlin.concurrent.thread

private val LocalVello = staticCompositionLocalOf<Vello?> { null }

/**
 * The context
 */
@Composable
fun VelloContext(allowNested: Boolean = false, content: @Composable() () -> Unit) {
    if (!allowNested && LocalVello.current != null) {
        // https://stackoverflow.com/a/71871643
        // This ensures that the exception isn't swallowed by Compose's machinery
        thread(name = "VelloComposeAssertion") {
            throw IllegalStateException(
                "Tried to nest `VelloContext`s." +
                        "Enable `allowNested` on the inner `VelloContext` if this is intentional"
            )
        }
    }
    val scope = rememberCoroutineScope();
    val vello = remember { Vello(scope) }
    CompositionLocalProvider(LocalVello provides vello) {
        content()
    }
    DisposableEffect(Unit) {
        onDispose {
            // If we're being disposed, all of our child components have been disposed, so cleaning
            // up now is fine.
            vello.cleanup()
        }
    }
}

@Composable
fun VariableFontsVelloSurface(
    text: String,
    fontSize: Float,
    fontWeight: Float = 400f,
    modifier: Modifier = Modifier
) {
    val textChannel = remember { Channel<String>(Channel.CONFLATED) }
    VariableFontsSendChannel(textChannel, text)

    val fontSizeChannel = remember { Channel<Float>(Channel.CONFLATED) }
    VariableFontsSendChannel(fontSizeChannel, fontSize)

    val weightChannel = remember { Channel<Float>(Channel.CONFLATED) }
    VariableFontsSendChannel(weightChannel, fontWeight)

    /// In theory, we shouldn't actually need to remember these, because the channels will be sent
    // them. However, that doesn't work in the case of a recreated surface (e.g. for scrolling
    // on/off screen?)
    val currentText = rememberUpdatedState(text);
    val currentFontSize = rememberUpdatedState(fontSize);
    val currentFontWeight = rememberUpdatedState(fontWeight);

    BaseVelloSurface(modifier) {
        val vfSurface = VariableFontSurface(
            this,
            currentText.value,
            currentFontSize.value,
            currentFontWeight.value
        )
        while (true) {
            select {
                weightChannel.onReceive { weight ->
                    vfSurface.setWeight(weight)
                }
                fontSizeChannel.onReceive { fontSize ->
                    vfSurface.setFontSize(fontSize)
                }
                textChannel.onReceive { text ->
                    vfSurface.setText(text)
                }
            }

        }
    }
}

@Composable
private fun <V> VariableFontsSendChannel(channel: Channel<V>, value: V) {
    SideEffect { channel.trySend(value) }
}

/**
 * A View which creates and manages the lifetime of a [VelloSurface].
 *
 * Must be called inside a [VelloContext].
 */
@Composable
fun BaseVelloSurface(modifier: Modifier = Modifier, withSurface: suspend VelloSurface.() -> Unit) {
    val vello = LocalVello.current
        ?: throw IllegalStateException("Tried to use Vello outside of a `VelloContext`");
    AndroidExternalSurface(modifier) {
        onSurface { surface, initialWidth, initialHeight ->
            val velloSurface = vello.createSurface(surface, initialWidth, initialHeight)
            surface.onChanged { newWidth, newHeight ->
                velloSurface.resize(newWidth, newHeight)
            }
            surface.onDestroyed {
                velloSurface.cleanUp()
            }
            withSurface(velloSurface)
        }
    }
}