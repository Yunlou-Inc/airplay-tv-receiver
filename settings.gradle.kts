pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

// Spotless creates detached configurations for formatter artifacts such as ktlint.
// With FAIL_ON_PROJECT_REPOS, declare the repository centrally for those configurations.
gradle.beforeProject {
    buildscript.repositories {
        mavenCentral()
    }
}

rootProject.name = "AirPlayTvReceiver"
include(":app")
