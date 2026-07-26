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
    pluginConfiguration {
        ideaVersion {
            sinceBuild = "262"
        }
    }
}

tasks.test { useJUnitPlatform() }
