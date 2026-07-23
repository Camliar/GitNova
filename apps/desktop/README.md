# Desktop Host

Tauri 2 + React 19 Desktop Host. This package owns the native window, presentation, accessibility, and the lifecycle and byte transport for a separate `gitnova-core` process.

The Tauri layer starts Core directly, completes the protocol handshake, transports framed JSON-RPC requests, and guarantees child cleanup. The React shell displays the real connection state and an explicit retry action. It intentionally contains no Git, GitHub, PR, diff, or Squash Trace business logic. See [Desktop Core Transport](../../docs/DESKTOP_CORE_TRANSPORT.md).

The first end-to-end Desktop slice lets the user choose one directory with the native dialog and sends that opaque path to Core's `repository/open`. Repository kind, roots, Git directory, and Git version shown by the Host are protocol facts returned by Core. Desktop does not scan for repositories, inspect `.git`, or run Git. Dialog access is limited to the main window's open-dialog permission and does not grant general filesystem access.

After a non-bare repository opens, Desktop requests one `repository/status` snapshot and renders the Core-provided branch, upstream divergence, and ordered changes. Staged and working-tree states remain separate, including untracked, rename/copy, and conflict entries. Refresh is explicit: there is no watcher or polling, and this read-only slice cannot stage, discard, or modify files. Bare repositories are identified before the request and show that no working tree is available.

Tracked status entries expose separate staged and working-tree diff actions when that scope contains a change. Desktop sends the Core-provided repository-relative path and selected scope to `repository/diff`, then renders structured hunks, old/new line numbers, binary, and empty results. Repository-controlled line content is inserted only as text. Untracked content is not read or synthesized, and refreshing status closes any previously selected diff.

Desktop also requests the Core-owned `repository/graph` projection after any repository opens, including bare repositories. The timeline renders commit order, parents, HEAD, and branch/tag decorations exactly as projected by Core. Pages contain 30 commits and advance only through the opaque `nextCursor`; the Host never parses or persists it. Incremental failures retain already loaded commits and can be retried independently from working-tree status.

Each timeline row can open `repository/commitDiff`. Root and single-parent commits use Core's automatic comparison rules; merge commits require the user to choose one displayed direct parent before any request is sent. The detail presents the full message, author/committer metadata, actual comparison parent, ordered changed files, and the shared structured diff renderer. Closing or replacing a selection invalidates its pending response, while errors retain the timeline and exact comparison for retry.

Run `pnpm --filter @gitnova/desktop check`, `test`, or `build` from the repository root. Debug and test builds may select an absolute Core executable with `GITNOVA_CORE_BINARY`; release builds resolve Core beside the Desktop executable.

Native configuration lives in `src-tauri`. Its default capability grants only Tauri core window/event functionality; no shell, network, filesystem, or process plugin is enabled.
