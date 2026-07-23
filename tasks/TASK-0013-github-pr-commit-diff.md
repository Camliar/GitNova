# TASK-0013: GitHub PR Original Commit Diff

- **Status:** In Progress
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/013-github-pr-commit-diff`
- **Estimate:** 2–3 days
- **Dependencies:** TASK-0011 GitHub PR original commits (`f7ecd13`), ADR-0005

## Goal

通过 Core-owned GitHub Provider 返回指定 PR original commit 的完整文件列表与可用行级 patch，使 Squash Merge 后仍可逐个查看原始 commit 的具体改动。

## Scope

- 新增显式网络方法 `github/pullRequestCommitDiff`，参数包含正整数 PR `number`、完整 40/64 位十六进制 commit `oid`，以及可选 `remote`、`nameWithOwner`。
- 先通过 PR original commits 验证 `oid` 属于指定 PR，禁止把该方法退化为任意远端 commit 查询。
- 调用 `GET repos/{owner}/{repo}/commits/{oid}?per_page=100`，通过 `gh api --paginate --slurp` 收集分页文件列表。
- 返回 repository identity、PR number、匹配的 normalized original commit，以及保持 GitHub 顺序的文件 diff。
- 每个文件返回 old/new path、normalized status、additions/deletions/changes、`patchState` 与结构化 hunks/lines。
- 对 GitHub 未返回 `patch` 的文件显式返回 `unavailable` 和空 hunks，不臆测其一定为 binary。
- 校验分页 commit OID 一致、文件路径不重复、rename 的 previous path、patch 结构与行号；达到 GitHub 3000 文件上限时拒绝静默截断。
- 限制 commit pages 响应体积；不透传 raw JSON、headers、stderr 或凭据。
- 协议升级至 `1.10`，增加 `githubPullRequestCommitDiff` capability，同步 Rust、Schema、TypeScript 和文档。
- fake runner 与 contract 测试覆盖 membership、分页、rename、missing patch、结构化 hunks、上限和稳定错误。

## Non-goals

- Squash relationship 自动判定、confidence/reason 或本地 squash commit 关联。
- 任意 GitHub commit diff、compare API、merge-base 或跨 commit range diff。
- 对缺失 patch 的文件下载 blob 并在本地重建 diff。
- GitHub Enterprise、SQLite、background refresh、Host UI 或 PR 写操作。

## Deliverables

- [ ] `github/pullRequestCommitDiff` method 与 PR membership guard
- [ ] paginated commit-file response parser 与 3000 文件上限保护
- [ ] normalized file status、patch availability 与 structured hunks
- [ ] Rust、Schema、TypeScript、capability 与错误同步
- [ ] fake runner、contract、安全/边界测试
- [ ] PR commit diff 语义、限制与显式联网文档

## Review Checklist

- [ ] GitHub Provider 逻辑只在 Core，复用 ADR-0005 凭据边界。
- [ ] 只能查询指定 PR original commit，文件顺序与路径/统计保真。
- [ ] pagination、3000 文件上限与 missing patch 不静默降级。
- [ ] 不泄漏 raw JSON、stderr、token 或非必要 Provider 字段。
- [ ] 未实现 Squash Trace 判定、任意 commit 查询、PR 写操作或 Host UI。
- [ ] fmt、Clippy、Rust tests、`npm run check` 与文档检查通过。

## Done Definition

- [ ] Deliverables 和 Review Checklist 完成。
- [ ] 自主 Review 无阻塞项。
- [ ] 状态更新 Done，提交、推送并快进合并至 `main`。

## References

[GitHub Pull Requests](../docs/GITHUB_PULL_REQUESTS.md) · [GitHub Provider](../docs/GITHUB_PROVIDER.md) · [ADR-0005](../adr/ADR-0005-GitHub-Provider.md) · [GitHub REST Commits API](https://docs.github.com/en/rest/commits/commits) · [Roadmap](../docs/ROADMAP.md#phase-3--desktop-squash-trace-mvp)
