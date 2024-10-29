package org.linebender.vellocomposeapp

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import org.linebender.vello.Vello
import org.linebender.vello.compose.VariableFontsVelloSurface
import org.linebender.vello.compose.VelloContext
import org.linebender.vellocomposeapp.ui.theme.VelloComposeTheme

class MainActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        Vello.initRust()
        setContent {
            VelloComposeTheme {
                VelloContext {
                    Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                        Column(Modifier.padding(innerPadding)) {
                            Text("Hello from Compose")
                            CustomView()
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun Greeting(name: String, modifier: Modifier = Modifier) {
    Text(
        text = "Hello $name!",
        modifier = modifier
    )
}

@Preview(showBackground = true)
@Composable
fun GreetingPreview() {
    Vello.initRust()
    VelloComposeTheme {
        Greeting("Android Testing again")
    }
}

@Composable
fun CustomView() {
    val infiniteTransition = rememberInfiniteTransition(label = "infinite")
    val weight by infiniteTransition.animateFloat(
        initialValue = 100f,
        targetValue = 1000f,
        animationSpec = infiniteRepeatable(
            animation = tween(1000, easing = LinearEasing),
            repeatMode = RepeatMode.Reverse
        ),
        label = "fontWeight"
    )
    VariableFontsVelloSurface(
        "00:00:10", 70f, modifier = Modifier
            .fillMaxWidth()
            .height(60.dp)
    )
    Text("In Between")
    VariableFontsVelloSurface(
        "00:00:20", 70f, weight, modifier = Modifier
            .fillMaxWidth()
            .height(60.dp)
    )
    VariableFontsVelloSurface(
        "00:00:20", 70f, weight, modifier = Modifier
            .fillMaxWidth()
            .height(60.dp)
    )
    VariableFontsVelloSurface(
        "00:00:20", 70f, weight, modifier = Modifier
            .fillMaxWidth()
            .height(60.dp)
    )
    VariableFontsVelloSurface(
        "00:00:20", 70f, weight, modifier = Modifier
            .fillMaxWidth()
            .height(60.dp)
    )
    VariableFontsVelloSurface(
        "00:00:20", 70f, weight, modifier = Modifier
            .fillMaxWidth()
            .height(60.dp)
    )
    VariableFontsVelloSurface(
        "00:00:20", 70f, weight, modifier = Modifier
            .fillMaxWidth()
            .height(60.dp)
    )
    VariableFontsVelloSurface(
        "00:00:20", 70f, weight, modifier = Modifier
            .fillMaxWidth()
            .height(60.dp)
    )
    VariableFontsVelloSurface(
        "00:00:20", 70f, weight, modifier = Modifier
            .fillMaxWidth()
            .height(60.dp)
    )
    VariableFontsVelloSurface(
        "00:00:20", 70f, weight, modifier = Modifier
            .fillMaxWidth()
            .height(60.dp)
    )
    VariableFontsVelloSurface(
        "00:00:20", 70f, weight, modifier = Modifier
            .fillMaxWidth()
            .height(60.dp)
    )
    VariableFontsVelloSurface(
        "00:00:20", 70f, weight, modifier = Modifier
            .fillMaxWidth()
            .height(60.dp)
    )
    VariableFontsVelloSurface(
        "00:00:20", 70f, weight, modifier = Modifier
            .fillMaxWidth()
            .height(60.dp)
    )
    VariableFontsVelloSurface(
        "00:00:20", 70f, weight, modifier = Modifier
            .fillMaxWidth()
            .height(60.dp)
    )
    VariableFontsVelloSurface(
        "00:00:20", 70f, weight, modifier = Modifier
            .fillMaxWidth()
            .height(60.dp)
    )
}
