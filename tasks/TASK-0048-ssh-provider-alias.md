# TASK-0048: SSH Provider Alias Resolution

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `codex/task-048-ssh-provider-alias`
- **Dependencies:** completed TASK-0047 baseline (`932439c`)

## Goal

让 Core 正确识别 `git@github-work:owner/repo.git` 一类由用户 SSH config 映射到 GitHub/GitLab 的 remote，并把无文件变化的 commit 明确显示为空提交，而不是让用户误以为 diff 再次失败。

## Decisions

- 仅对已严格解析的 SSH remote hostname 调用仓库环境中的 System OpenSSH `ssh -G -- <alias>`；不连接网络、不执行 Host-side 解析。
- Core 只读取有界输出中的单个 `hostname` 值，重新执行 hostname 安全验证；不返回 SSH config、用户名、端口、proxy 或凭据。
- GitHub alias 必须解析为 `github.com`；GitLab alias 解析后的安全 hostname 继续作为 `glab --hostname` 参数。
- HTTPS remote 不做 alias 解析；显式 Provider identity override 保持原契约。
- changed-file 列表为空时，Desktop 明确说明 commit tree 与比较基线相同。

## Scope

- Core-owned SSH alias resolution for GitHub and GitLab remote identities.
- Bounded, non-interactive command invocation and injection rejection.
- Provider command-boundary tests and a real configured-alias smoke check.
- Desktop empty-commit copy and regression test.
- synchronized Provider/Desktop/Roadmap docs.

## Non-goals

- 修改用户 SSH config、连接 SSH server、支持动态 wildcard rewrite 或读取私钥。
- 把 SSH/Provider 解析放进 Host。
- 多仓库 Tab/工具栏重构（TASK-0049）或 timeline lane 样式（TASK-0050）。

## Done Definition

- [x] Alias remote resolves only through bounded Core validation.
- [x] GitHub/GitLab and empty-commit regressions pass.
- [x] Autonomous review passes; branch is committed, pushed and fast-forwarded to remote `main`.

## Verification

- `cargo test --workspace` (99 tests)
- `cargo clippy --workspace --all-targets -- -D warnings`
- Desktop TypeScript, Vite build and 83 Vitest tests
- protocol generation, quality and delivery checks
- local smoke check: `ssh -G -- github-lp` resolves only to `hostname github.com`; observed commit `a6bcb207` and parent share tree `70289750…`
