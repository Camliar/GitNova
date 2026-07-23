# Desktop Core Transport

GitNova Desktop is a presentation Host. It does not execute Git commands or interpret GitHub, pull request, diff, or Squash Trace data. The Tauri Rust layer owns only the lifecycle and byte transport for one independent `gitnova-core` child process.

## Process discovery and startup

The production Host resolves `gitnova-core` beside the Desktop executable. Debug and test builds may set `GITNOVA_CORE_BINARY` to an absolute executable path. Relative overrides are rejected, and Core is started directly without a shell, daemon, port, or Tauri shell plugin.

stdin, stdout, and stderr are piped. stdout is reserved for JSON-RPC frames. stderr is drained in the background and is never returned to the UI, preventing repository paths, credentials, and provider diagnostics from crossing the Host boundary.

## Handshake and requests

Immediately after spawn, the Host sends `gitnova/initialize`. It accepts only the same protocol major version and requires the repository discovery, GitHub PR commit diff, and Squash Trace capabilities needed by the Desktop MVP.

Requests are serialized through one supervisor. IDs are monotonically increasing integers. Both directions use the protocol `Content-Length` framing with a 16 MiB maximum. The Host validates JSON-RPC version, response ID, and the exclusive presence of `result` or `error`; malformed frames, unexpected EOF, timeouts, and mismatched responses fail closed and terminate the child.

The Tauri boundary exposes only status, start, generic request transport, and shutdown commands. Lifecycle methods cannot be sent through the generic command. Domain payloads remain opaque to the Host and are typed for UI consumers from the shared protocol package.

## Shutdown and errors

Normal shutdown sends `gitnova/shutdown`, then the `exit` notification, and waits briefly for Core to exit. Timeout, transport failure, application exit, and destructor paths kill and reap the process so no child is left behind.

Desktop lifecycle errors contain only a stable code, a fixed user-safe message, and retryability. Raw operating-system errors and child stderr are intentionally excluded.

