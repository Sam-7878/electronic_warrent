import androidx.compose.runtime.Composable
import androidx.compose.ui.window.ComposeUIViewController

actual fun getPlatformName(): String = "iOS"

@Composable fun MainView() = UserMobileApp()

fun MainViewController() = ComposeUIViewController { MainView() }
