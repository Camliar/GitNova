# Desktop Core Transport

GitNova Desktop is a presentation Host. It does not execute Git commands or interpret GitHub, pull request, diff, or Squash Trace data. The Tauri Rust layer owns only the lifecycle and byte transport for one independent `gitnova-core` child process.

## Process discovery and startup

The production local target resolves `gitnova-core` beside the Desktop executable. Debug and test builds may set `GITNOVA_CORE_BINARY` to an absolute executable path. Relative overrides are rejected. macOS GUI applications do not reliably inherit the interactive shell PATH, so the local Core child preserves the inherited order and appends the fixed conventional `/opt/homebrew/bin`, `/usr/local/bin`, `/opt/local/bin` and system binary directories when absent. This process-environment projection lets Core find an already-installed System Git/Provider CLI; Desktop still never invokes those tools, scans for executables, accepts a path from UI, or handles credentials. WSL, SSH and Dev Container targets use their own untouched PATH through the closed structured projections in [Remote Core Environments](REMOTE_ENVIRONMENTS.md) and [ADR-0006](../adr/ADR-0006-Remote-Core-Launching.md). No target accepts an arbitrary command or uses a local shell, daemon, port, or Tauri shell plugin.

stdin, stdout, and stderr are piped. stdout is reserved for JSON-RPC frames. stderr is drained in the background and is never returned to the UI, preventing repository paths, credentials, and provider diagnostics from crossing the Host boundary.

## Handshake and requests

Immediately after spawn, the Host sends `gitnova/initialize`. It accepts only the same protocol major version and requires the repository discovery, GitHub PR commit diff, and Squash Trace capabilities needed by the Desktop MVP.

Requests are serialized through one supervisor. IDs are monotonically increasing integers. Both directions use the protocol `Content-Length` framing with a 16 MiB maximum. The Host validates JSON-RPC version, response ID, and the exclusive presence of `result` or `error`; malformed frames, unexpected EOF, timeouts, and mismatched responses fail closed and terminate the child.

The Tauri boundary exposes only structured environment configuration, status, start, allowlisted request transport, and shutdown commands. Environment changes are rejected while Core runs. Lifecycle and arbitrary methods cannot be sent through the generic command; the closed Desktop allowlist contains only methods used by its current Repository/GitHub/mutation/AI views. Domain payloads remain opaque to the Host and are typed for UI consumers from the shared protocol package.

Local read and non-network mutation requests retain the 15-second transport timeout. Explicit GitHub/GitLab Provider requests and `repository/fetch`/`pull`/`push` receive a 45-second ceiling, so `gh`/`glab`/remote System Git can complete without being mistaken for a dead local Core; explicit `ai/generateCommitDraft` receives a separate 75-second ceiling for Core's bounded 60-second Provider request. Every timeout still fails closed and terminates the child. `ai/inputPreview` remains a normal local request and never receives the extended network budget.

## Shutdown and errors

Normal shutdown sends `gitnova/shutdown`, then the `exit` notification, and waits briefly for Core to exit. Timeout, transport failure, application exit, and destructor paths kill and reap the process so no child is left behind.

Desktop lifecycle errors contain only a stable code, a fixed user-safe message, and retryability. Raw operating-system errors and child stderr are intentionally excluded.
