# Desktop Host

Tauri 2 + React 19 Desktop Host. This package owns the native window, presentation, accessibility, and the lifecycle and byte transport for a separate `gitnova-core` process.

The Tauri layer starts Core directly, completes the protocol handshake, transports framed JSON-RPC requests, and guarantees child cleanup. The React shell displays the real connection state and an explicit retry action. It intentionally contains no Git, GitHub, PR, diff, or Squash Trace business logic. See [Desktop Core Transport](../../docs/DESKTOP_CORE_TRANSPORT.md).

Run `pnpm --filter @gitnova/desktop check`, `test`, or `build` from the repository root. Debug and test builds may select an absolute Core executable with `GITNOVA_CORE_BINARY`; release builds resolve Core beside the Desktop executable.

Native configuration lives in `src-tauri`. Its default capability grants only Tauri core window/event functionality; no shell, network, filesystem, or process plugin is enabled.
