# GitNova for JetBrains IDEs

This IntelliJ Platform plugin is a thin Host for `gitnova-core`. A project-scoped service owns process lifecycle and JSON-RPC bytes; the Tools menu action sends the current project path to Core, explicitly requests one GitHub PR/Squash Trace, lets the user select an original commit, and displays the returned remote diff. No Git, `gh`, Provider HTTP or relationship inference exists in the plugin.

The plugin backend and Core run in the same project environment. Set the JVM system property `gitnova.core.path` only to an absolute development binary path; otherwise a compatible `gitnova-core` is resolved from backend PATH. Remote Development therefore requires Core inside the remote backend environment.

The project targets IntelliJ IDEA 2026.2 with Java 17 and IntelliJ Platform Gradle Plugin 2.18.1. Run `node scripts/check-idea.mjs` for dependency-free framing/security checks. Run `./gradlew test buildPlugin` when the Gradle wrapper and IntelliJ Platform artifacts are available. Marketplace publication and automatic Core installation are out of scope.
