# TASK-0011: GitHub PR Original Commits

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/011-github-pr-original-commits`
- **Estimate:** 2–3 days
- **Dependencies:** TASK-0010 GitHub Provider foundation (`292dfcd`), ADR-0005

## Goal

通过 Core-owned GitHub Provider 返回指定 PR 的规范化 detail 与按 GitHub 顺序排列的 original commits，形成 Squash Merge 后仍可浏览 PR 原始提交链的首个核心领域契约。

## Scope

- 新增显式网络方法 `github/pullRequest`，参数包含正整数 `number`，以及与 `github/repository` 相同的可选 `remote`、`nameWithOwner` identity 选择。
- 复用 TASK-0010 严格 repository identity、`gh` command runner、非交互环境与稳定 Provider 错误。
- 调用 `GET repos/{owner}/{repo}/pulls/{number}` 获取 PR detail。
- 调用 `GET repos/{owner}/{repo}/pulls/{number}/commits?per_page=100`，使用 `gh api --paginate --slurp` 收集并扁平化所有返回页，保持 API 顺序。
- 返回 `GitHubPullRequest`：repository identity、number、title/body、state、draft、author、URL、created/updated/closed/merged timestamps、base/head refs 与 OIDs、head repository、merge commit OID、original commits。
- 每个 original commit 返回 oid、ordered parents、author/committer（name/email/timestamp/可选 GitHub login）、summary、完整 message 和 URL。
- `state` 规范化为 `open`、`closed`、`merged`；`merged=true` 优先于 REST `state=closed`。
- 校验 detail 的 `commits` 数量与实际列表一致；GitHub PR commits endpoint 的 250 commit 上限不得静默截断，返回稳定 `github.pr_commit_limit_exceeded`。
- 限制 detail 1 MiB、commit pages 16 MiB；不透传 body 之外的原始 JSON、headers 或 stderr。
- 协议升级至 `1.9`，增加 `githubPullRequest` capability，同步 Rust、Schema、TypeScript 和文档。
- fake runner 测试 command、pagination/slurp flatten、merge/squash OID、multiline message、nullable users/fork、数量不一致、oversize、auth/request/parse；contract 测试 params/session gating。

## Non-goals

- original commit 的 files、patch 或行级 diff（下一独立 Task）。
- squash relationship 自动判定、confidence/reason 或本地 commit 关联。
- PR 列表、搜索、评论、review、checks 或任何 PR 写操作。
- 超过 GitHub REST PR commits 250 上限的 GraphQL/generic commits fallback。
- GitHub Enterprise、SQLite、background refresh、Host UI 或 Desktop。

## Deliverables

- [x] `github/pullRequest` detail + original commits
- [x] normalized PR、ref、identity 与 commit 类型
- [x] paginated/slurped commit response parser 与 250 上限保护
- [x] Rust、Schema、TypeScript、capability 与错误同步
- [x] fake runner、contract、安全/边界测试
- [x] PR original commits 语义、限制与显式联网文档

## Review Checklist

- [x] GitHub Provider 逻辑只在 Core，复用 ADR-0005 凭据边界。
- [x] original commits 顺序、parents、identity、message 和 merge commit OID 保真。
- [x] pagination 不遗漏，250 上限与 count mismatch 不静默降级。
- [x] 不泄漏 raw JSON、stderr、token 或非必要 Provider 字段。
- [x] 未实现 per-commit diff、Squash Trace 判定、PR 写操作或 Host UI。
- [x] fmt、Clippy、Rust tests、`npm run check` 与文档检查通过。

## Done Definition

- [x] Deliverables 和 Review Checklist 完成。
- [x] 自主 Review 无阻塞项。
- [x] 状态更新 Done，提交、推送并快进合并至 `main`。

## References

[GitHub Provider](../docs/GITHUB_PROVIDER.md) · [ADR-0005](../adr/ADR-0005-GitHub-Provider.md) · [GitHub REST PR API](https://docs.github.com/en/rest/pulls/pulls) · [Roadmap](../docs/ROADMAP.md#phase-3--desktop-squash-trace-mvp)
