# GitNova for VS Code

This extension is a thin Host for the independent `gitnova-core` process. It owns VS Code commands, status bar state, workspace path input, process lifecycle and safe webview presentation. It never runs Git/`gh`, calls Provider HTTP APIs or derives Squash Trace relationships.

The Extension Host starts Core in its own environment. Consequently a workspace opened through VS Code Remote SSH, WSL or Dev Containers launches Core there and sends the environment-native workspace path unchanged. A compatible `gitnova-core` must be on PATH in remote Extension Hosts. Local packaged extensions may include it under `bin/`; development can set the absolute `gitnova.core.path` setting.

Commands:

- `GitNova: Connect Core`
- `GitNova: Open Workspace Repository`
- `GitNova: Inspect Pull Request / Squash Trace`

The PR command is explicitly user-triggered, explains that Core will use the configured GitHub Provider, renders original commits and conservative relationship evidence, then lets the user select one original commit for its Core-owned remote diff. Webviews have no scripts or remote origins.

Run `pnpm --filter @gitnova/vscode check` from the repository root. Marketplace packaging and automatic remote Core installation are intentionally out of scope.
