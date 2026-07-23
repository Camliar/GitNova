# Desktop Host

Tauri 2 + React 19 Desktop Host. This package owns the native window, presentation, accessibility, and the lifecycle and byte transport for a separate `gitnova-core` process.

The Tauri layer starts Core directly, completes the protocol handshake, transports framed JSON-RPC requests, and guarantees child cleanup. The React shell displays the real connection state and an explicit retry action. It intentionally contains no Git, GitHub, PR, diff, or Squash Trace business logic. See [Desktop Core Transport](../../docs/DESKTOP_CORE_TRANSPORT.md).

The first end-to-end Desktop slice lets the user choose one directory with the native dialog and sends that opaque path to Core's `repository/open`. Repository kind, roots, Git directory, and Git version shown by the Host are protocol facts returned by Core. Desktop does not scan for repositories, inspect `.git`, or run Git. Dialog access is limited to the main window's open-dialog permission and does not grant general filesystem access.

After a non-bare repository opens, Desktop requests one `repository/status` snapshot and renders the Core-provided branch, upstream divergence, and ordered changes. Staged and working-tree states remain separate, including untracked, rename/copy, and conflict entries. Refresh is explicit: there is no watcher or polling, and this read-only slice cannot stage, discard, or modify files. Bare repositories are identified before the request and show that no working tree is available.

Run `pnpm --filter @gitnova/desktop check`, `test`, or `build` from the repository root. Debug and test builds may select an absolute Core executable with `GITNOVA_CORE_BINARY`; release builds resolve Core beside the Desktop executable.

Native configuration lives in `src-tauri`. Its default capability grants only Tauri core window/event functionality; no shell, network, filesystem, or process plugin is enabled.
