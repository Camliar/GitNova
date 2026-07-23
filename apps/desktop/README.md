# Desktop Host

Tauri 2 + React 19 Desktop Host. This package owns the native window, presentation, accessibility, and—in later Tasks—the lifecycle of a separate `gitnova-core` process.

The current foundation intentionally contains no Git, GitHub, PR, or Squash Trace business logic and does not start Core yet. Run `pnpm --filter @gitnova/desktop check`, `test`, or `build` from the repository root.

Native configuration lives in `src-tauri`. Its default capability grants only Tauri core window/event functionality; no shell, network, filesystem, or process plugin is enabled.
