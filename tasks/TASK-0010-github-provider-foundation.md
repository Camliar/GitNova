# TASK-0010: GitHub Provider Foundation

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/010-github-provider-foundation`
- **Estimate:** 2–3 days
- **Dependencies:** TASK-0009 commit graph projection (`238ba6c`), ADR-0005

## Goal

在 `gitnova-core` 内建立首个显式、只读、非交互的 GitHub Provider 请求，从已打开仓库解析 GitHub identity 并通过用户配置的 `gh` 凭据返回规范化 repository metadata。

## Scope

- 接受 ADR-0005：首个 Provider adapter 使用 `gh api`，不由 Host 或 GitNova 管理 token。
- 新增 `github/repository`，参数为可选 `remote`（默认 `origin`）和可选 `nameWithOwner` 显式覆盖。
- 未覆盖时通过 System Git `remote get-url` 读取指定 remote，并解析 `https://github.com/owner/repo(.git)`、`ssh://git@github.com/owner/repo(.git)` 和 `git@github.com:owner/repo(.git)`。
- 首个 Task 仅允许 `github.com` 与严格 `owner/name`；不接受额外 path segment、query、fragment、空 segment 或 revision-like 输入。
- 显式调用 `gh api --hostname github.com repos/{owner}/{repo}`，设置 `GH_PROMPT_DISABLED=1`、`GH_PAGER=cat`、`NO_COLOR=1`，不经 shell。
- 返回规范化 `GitHubRepository`：host、owner、name、nameWithOwner、url、defaultBranch、isPrivate。
- Core 只选择所需 JSON 字段，不向 Host 透传 `gh` 原始响应或 stderr。
- 稳定区分 remote 缺失/unsupported、`gh` unavailable、authentication required、request failed 和 response parse failed。
- `github/repository` 是用户/Host 显式网络动作；Core 不自动调用、不重试、不缓存、不写 SQLite。
- 协议升级至 `1.8`，增加 `githubRepository` capability，同步 Rust、Schema、TypeScript、ADR 和文档。
- 以 fake Git/gh runner 测试命令参数、环境、URL 解析、JSON normalization、错误/敏感信息不泄漏；Core contract 测试参数与 repository session gating。

## Non-goals

- PR 列表/detail、original commits、per-commit remote diff 或 Squash Trace。
- GitHub Enterprise、GitLab、direct REST client、GraphQL 或 adapter fallback。
- 登录 UI、`gh auth login`、读取/显示 token、scope 管理。
- background refresh、cache、SQLite、webhook 或中心服务器。
- Host UI、Desktop 网络层或任何 Git 写操作。

## Deliverables

- [x] ADR-0005 与 Core GitHub Provider/`gh` adapter 边界
- [x] GitHub remote/nameWithOwner 严格解析
- [x] `github/repository` 与规范化 metadata
- [x] Rust、Schema、TypeScript、capability 与稳定错误同步
- [x] fake runner、contract、安全与错误测试
- [x] 显式联网、凭据和限制文档

## Review Checklist

- [x] GitHub/Git 业务逻辑只在 Core，Host 不接触凭据。
- [x] 仅显式请求联网；不自动重试、缓存或后台访问。
- [x] 不调用 shell 或 `gh auth token`，命令/错误不泄漏 token、stderr 或响应正文。
- [x] remote/repository identity 严格验证，不能注入 endpoint/flag。
- [x] normalized protocol 不暴露 Provider 原始 JSON。
- [x] 未实现 PR、Squash Trace、Enterprise、SQLite 或 Host UI。
- [x] fmt、Clippy、Rust tests、`npm run check` 与文档检查通过。

## Done Definition

- [x] Deliverables 和 Review Checklist 完成。
- [x] 自主 Review 无阻塞项。
- [x] 状态更新 Done，提交、推送并快进合并至 `main`。

## References

[ADR-0005](../adr/ADR-0005-GitHub-Provider.md) · [Local First](../adr/ADR-0003-Local-First.md) · [Architecture](../docs/ARCHITECTURE.md) · [Roadmap](../docs/ROADMAP.md#phase-3--desktop-squash-trace-mvp)
