# Desktop Diagnostics

GitNova Desktop writes a best-effort local diagnostic log for troubleshooting Core lifecycle and transport failures. It is not telemetry: no event is uploaded, no GitNova account exists, and GitNova does not automatically export the files.

## Location and retention

Settings → Diagnostic log displays the exact active file and provides **Copy path**. Tauri resolves the parent directory per platform:

- macOS: `~/Library/Logs/dev.gitnova.desktop/diagnostics.jsonl`
- Windows: the local app-data `dev.gitnova.desktop/logs/diagnostics.jsonl`
- Linux: the local data `dev.gitnova.desktop/logs/diagnostics.jsonl`

The active JSON Lines file rotates before it would exceed 1 MiB. GitNova retains only `diagnostics.jsonl` and `diagnostics.previous.jsonl`. Rotation and write failures are ignored so diagnostics can never block startup, Core requests, shutdown, or repository work.

## Event contract

The typed Desktop writer accepts only a fixed metadata allowlist:

- millisecond timestamp, severity and event name;
- Desktop version and negotiated protocol version where applicable;
- Core environment (`local`, `wsl`, `ssh`, or `devContainer`);
- allowlisted JSON-RPC method name, elapsed milliseconds and outcome;
- stable Desktop/Core error code.

It has no parameter capable of accepting repository paths, RPC params/results, commit messages, author identity, diffs, Git output, child stderr, Provider response bodies, remote URLs, prompts, API keys, tokens, or other credentials. A rejected arbitrary method is logged without its caller-provided method string. Core stdout remains framed transport only, and child stderr continues to be drained without capture.

When reporting a problem, reproduce it once, open Settings, copy the diagnostic path and inspect the JSONL file before sharing it. The log is designed to avoid repository content, but it still reveals app timing, selected environment, invoked feature names, versions and stable failure categories; share it only with parties you trust.
