package io.github.jqssun.airplay.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

// Fixed brand palette. Dynamic color (Material You) is deliberately not used:
// on TV launchers it makes the app follow the system accent and lose identity.
private val BrandBlue = Color(0xFF2F6BFF)

private val LightScheme = lightColorScheme(
    primary = BrandBlue,
    onPrimary = Color.White,
    primaryContainer = Color(0xFFDCE5FF),
    onPrimaryContainer = Color(0xFF001A44),
    secondary = Color(0xFF00897B),
    onSecondary = Color.White,
    background = Color(0xFFF7F8FC),
    onBackground = Color(0xFF14171F),
    surface = Color(0xFFFFFFFF),
    onSurface = Color(0xFF14171F),
    surfaceVariant = Color(0xFFE9EDF7),
    onSurfaceVariant = Color(0xFF454B5A)
)

private val DarkScheme = darkColorScheme(
    primary = Color(0xFFAFC3FF),
    onPrimary = Color(0xFF00246B),
    primaryContainer = Color(0xFF1B3C8F),
    onPrimaryContainer = Color(0xFFDCE5FF),
    secondary = Color(0xFF4DD0C4),
    onSecondary = Color(0xFF00332E),
    background = Color(0xFF0B0F17),
    onBackground = Color(0xFFE6E9F2),
    surface = Color(0xFF121826),
    onSurface = Color(0xFFE6E9F2),
    surfaceVariant = Color(0xFF1C2436),
    onSurfaceVariant = Color(0xFFB8C0D4)
)

@Composable
fun AirPlayTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit
) {
    MaterialTheme(
        colorScheme = if (darkTheme) DarkScheme else LightScheme,
        content = content
    )
}
