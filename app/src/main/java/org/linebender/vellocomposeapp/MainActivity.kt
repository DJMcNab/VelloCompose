package org.linebender.vellocomposeapp

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
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
    VariableFontsVelloSurface(
        "00:00:10", 40f, modifier = Modifier
            .fillMaxWidth()
            .height(Dp(300f))
    )
    Text("In Between")
    VariableFontsVelloSurface(
        "00:00:20", 40f, 800f, modifier = Modifier
            .fillMaxWidth()
            .height(Dp(300f))
    )
}
