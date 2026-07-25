# Automation

`ci.yml` runs the complete quality gate on Windows, macOS and Linux with read-only repository permission. It never references repository signing secrets.

`release.yml` accepts only `v*` tags, validates that the tag exactly matches `apps/desktop/src-tauri/tauri.conf.json`, builds the matching Core sidecar and creates a draft GitHub Release. It is the only workflow with `contents: write` and signing-secret access. See [Desktop Release](../../docs/DESKTOP_RELEASE.md).
