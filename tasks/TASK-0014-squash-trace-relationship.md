# TASK-0014: Squash Trace Relationship

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/014-squash-trace-relationship`
- **Estimate:** 2–3 days
- **Dependencies:** TASK-0011 GitHub PR original commits (`f7ecd13`), TASK-0013 GitHub PR commit diff (`7787039`)

## Goal

在 Core 中把 GitHub PR、ordered original commits 与 Provider 返回的最终 merge commit OID 关联为可解释的 Squash Trace，并通过本地 System Git 拓扑证据避免把不确定推断包装成事实。

## Scope

- 新增显式网络方法 `github/squashTrace`，复用 `github/pullRequest` 的 `number`、`remote`、`nameWithOwner` 参数与完整性保护。
- 返回 normalized PR 与 relationship：classification、confidence、merge commit OID、本地可用性、本地 parent OIDs 和 machine-readable evidence。
- Provider 未合并时分类为 `notMerged`；merge OID 命中 original commit 时分类为 `originalCommit`。
- merge OID 在本地存在且具有两个或更多 parents 时分类为 `mergeCommit`。
- merge OID 与 originals 不同且本地只有 0/1 个 parent 时分类为 `squashCandidate`；明确说明 GitHub Provider 未提供 merge strategy，单靠该证据仍可能与 rebase 结果混淆。
- merge OID 缺失或本地对象不可用时分类为 `unresolved`，保留 Provider 事实和缺失证据，不把缺失对象作为请求错误。
- 本地拓扑只通过 System Git 读取，不 fetch、不写仓库、不调用 shell；真正的 Git 执行失败继续使用稳定 repository/Git 错误。
- 协议升级至 `1.11`，增加 `githubSquashTrace` capability，同步 Rust、Schema、TypeScript 和文档。
- 测试覆盖未合并、original match、merge topology、squash candidate、本地缺失、参数/session gating 与错误边界。

## Non-goals

- 声称精确获知 GitHub merge strategy，或把 `squashCandidate` 展示为已证实 squash。
- 聚合/比较全部 patch、tree 或 patch-id 来区分 squash 与 rebase。
- 自动 fetch merge commit、SQLite 缓存、background refresh 或历史重写。
- Desktop/Host UI、PR 列表/写操作或其他 Provider。

## Deliverables

- [x] `github/squashTrace` Core method
- [x] conservative classification、confidence 与 evidence model
- [x] local merge commit topology inspection
- [x] Rust、Schema、TypeScript、capability 与文档同步
- [x] unit、fake runner 与 contract 测试

## Review Checklist

- [x] Provider 事实与 Core inference 明确分离。
- [x] 缺少本地对象不误报、不自动 fetch、不泄漏内容。
- [x] merge、rebase/original 与 squash candidate 不被错误合并为一个确定状态。
- [x] Host 不承担 GitHub、Git 或 relationship 业务逻辑。
- [x] fmt、Clippy、Rust tests、`npm run check` 与文档检查通过。

## Done Definition

- [x] Deliverables 和 Review Checklist 完成。
- [x] 自主 Review 无阻塞项。
- [x] 状态更新 Done，提交、推送并快进合并至 `main`。

## References

[Product Requirements](../docs/PRODUCT_REQUIREMENTS.md) · [GitHub Pull Requests](../docs/GITHUB_PULL_REQUESTS.md) · [Architecture](../docs/ARCHITECTURE.md) · [Roadmap](../docs/ROADMAP.md#phase-3--desktop-squash-trace-mvp)
