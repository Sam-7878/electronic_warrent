// Root package

import androidx.compose.animation.core.*
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.composed
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.BlendMode
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.rotate
import androidx.compose.ui.unit.dp

val MetalDarkBg = Color(0xFF090A0F)
val MetalSurface = Color(0xFF13151A)
val NeonCyan = Color(0xFF00F0FF)
val NeonPurple = Color(0xFFB026FF)
val NeonGreen = Color(0xFF00FF66)

val AppColorScheme = darkColorScheme(
    background = MetalDarkBg,
    surface = MetalSurface,
    primary = NeonCyan,
    secondary = NeonPurple,
    tertiary = NeonGreen,
    onBackground = Color.White,
    onSurface = Color(0xFFB0B3B8)
)

@Composable
fun AppTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = AppColorScheme,
        content = content
    )
}

fun Modifier.glassmorphism(cornerRadius: Int = 16) = composed {
    this.background(
        color = Color(0xFF1A1C23).copy(alpha = 0.6f),
        shape = RoundedCornerShape(cornerRadius.dp)
    ).border(
        width = 1.dp,
        color = Color.White.copy(alpha = 0.08f),
        shape = RoundedCornerShape(cornerRadius.dp)
    )
}

fun Modifier.metalGradientBackground(cornerRadius: Int = 16) = composed {
    this.background(
        brush = Brush.linearGradient(
            colors = listOf(Color(0xFF2A2D34), Color(0xFF141518))
        ),
        shape = RoundedCornerShape(cornerRadius.dp)
    ).border(
        width = 1.dp,
        color = Color.White.copy(alpha = 0.15f),
        shape = RoundedCornerShape(cornerRadius.dp)
    )
}

fun Modifier.neonGlow(color: Color, cornerRadius: Int = 16) = composed {
    this.border(
        width = 2.dp,
        color = color.copy(alpha = 0.8f),
        shape = RoundedCornerShape(cornerRadius.dp)
    )
}

// 1. Holographic Sheen Animation
fun Modifier.holographicSheen() = composed {
    val infiniteTransition = rememberInfiniteTransition()
    val translateAnim by infiniteTransition.animateFloat(
        initialValue = -500f,
        targetValue = 2000f,
        animationSpec = infiniteRepeatable(
            animation = tween(durationMillis = 3000, easing = LinearEasing),
            repeatMode = RepeatMode.Restart
        )
    )

    this.drawWithContent {
        drawContent()
        val brush = Brush.linearGradient(
            colors = listOf(Color.Transparent, Color.White.copy(alpha = 0.15f), Color.Transparent),
            start = Offset(translateAnim, translateAnim),
            end = Offset(translateAnim + 300f, translateAnim + 300f)
        )
        drawRect(brush = brush, blendMode = BlendMode.Screen)
    }
}

// 2. Animated Neon Border (Laser Beam)
fun Modifier.animatedNeonBorder(color: Color, cornerRadius: Int = 16) = composed {
    val infiniteTransition = rememberInfiniteTransition()
    val angle by infiniteTransition.animateFloat(
        initialValue = 0f,
        targetValue = 360f,
        animationSpec = infiniteRepeatable(
            animation = tween(durationMillis = 2000, easing = LinearEasing),
            repeatMode = RepeatMode.Restart
        )
    )

    this.drawWithContent {
        drawContent()
        val sweepGradient = Brush.sweepGradient(
            colors = listOf(Color.Transparent, color.copy(alpha = 0.1f), color, Color.Transparent),
            center = Offset(size.width / 2, size.height / 2)
        )
        rotate(angle) {
            drawRoundRect(
                brush = sweepGradient,
                size = size,
                cornerRadius = androidx.compose.ui.geometry.CornerRadius(cornerRadius.dp.toPx()),
                style = androidx.compose.ui.graphics.drawscope.Stroke(width = 4.dp.toPx())
            )
        }
    }
}
