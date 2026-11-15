plugins {
    id("com.android.application") version "8.7.3" apply false
    id("org.jetbrains.kotlin.android") version "2.0.21" apply false
    id("org.jetbrains.kotlin.plugin.compose") version "2.0.21" apply false
    id("com.diffplug.spotless") version "7.0.2"
}

spotless {
    kotlin {
        target("**/*.kt")
        targetExclude("**/build/**")
        ktlint("1.5.0")
    }
    kotlinGradle {
        target("**/*.gradle.kts")
        ktlint("1.5.0")
    }
}
