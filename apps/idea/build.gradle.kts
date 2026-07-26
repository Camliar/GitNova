plugins {
    id("java")
    id("org.jetbrains.intellij.platform") version "2.18.1"
}

group = "dev.gitnova"
version = "0.1.0"

repositories {
    mavenCentral()
    intellijPlatform { defaultRepositories() }
}

dependencies {
    intellijPlatform { intellijIdea("2026.2") }
    testImplementation("org.junit.jupiter:junit-jupiter:5.13.4")
}

java { toolchain { languageVersion = JavaLanguageVersion.of(25) } }

intellijPlatform {
    // IDEA-390693: 2026.2 nullability instrumentation fails on otherwise valid Java classes.
    // GitNova uses no GUI Designer forms, so packaging the compiled classes directly is safe.
    instrumentCode = false
    pluginConfiguration {
        ideaVersion {
            sinceBuild = "262"
        }
    }
}

tasks.test { useJUnitPlatform() }
