# TASK-0051: macOS Packaged Provider CLI PATH

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `codex/task-051-macos-provider-cli-path`
- **Dependencies:** completed TASK-0050 baseline (`ddb1795`)

## Goal

修复从 Finder 启动已安装 GitNova 后，本机已安装于 Homebrew/MacPorts 标准目录的 `gh`/`glab` 无法被 Core 找到并错误报告 `github.gh_unavailable` 的问题。

## Decisions

- Desktop 仍只负责 Core lifecycle。仅在启动 **local** Core child 时补充 macOS GUI 常缺失的 `/opt/homebrew/bin`、`/usr/local/bin`、`/opt/local/bin`；Core 仍自行直接启动 Provider CLI 并拥有全部 GitHub/GitLab 语义。
- 保留继承 PATH 的原始顺序并去重，补充目录只追加、不覆盖用户已有 executable resolution。
- WSL、SSH、Dev Container launchers 不修改 PATH；仓库在哪里，Provider CLI 就必须安装/认证在哪里。
- 不使用 shell，不扫描磁盘，不接受 Host/UI 传入 executable path，也不捆绑或读取 Provider 凭据。

## Scope

- local Core child PATH projection and deterministic unit tests.
- packaged-app regression smoke test with a restricted Finder-like PATH.
- synchronized Desktop transport, Provider and release documentation.

## Non-goals

- 自动安装/升级/认证 `gh` 或 `glab`，读取 token，修改 shell profile。
- 为远程环境复制本机 CLI/PATH，或将 Provider API 调用移入 Host。

## Done Definition

- [x] Packaged local Core can resolve standard macOS Provider CLI installs.
- [x] Remote launch environments remain untouched.
- [x] PATH order/dedup and no-shell boundaries are tested.
- [x] Autonomous review has no blocking findings.

## Verification

- `cargo test --workspace` (100 tests)
- `cargo clippy --workspace --all-targets -- -D warnings`
- Desktop TypeScript, Vite production build and 87 Vitest tests
- protocol generation, quality and delivery checks
- restricted macOS GUI-equivalent PATH smoke: `/usr/local/bin/gh` resolves as GitHub CLI 2.96.0
- local authentication audit: configured GitHub CLI accounts require explicit user re-authentication; no credentials were read or changed by GitNova
