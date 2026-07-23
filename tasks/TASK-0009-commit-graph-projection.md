# TASK-0009: Commit Graph Projection

- **Status:** In Progress
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/009-commit-graph-projection`
- **Estimate:** 1–2 days
- **Dependencies:** TASK-0008 repository references (`eda3cc2`)

## Goal

在 Core 中把固定快照分页 commit 与 branch/tag decorations 关联成统一 graph projection，避免 Desktop 或未来 IDE Host 复制 Git reference-to-commit 关联逻辑。

## Scope

- 新增 `repository/graph`，复用 `repository/history` 的可选 `limit`、`cursor` 参数与 1–200 限制。
- 返回 ordered graph nodes；每个 node 包含完整 `CommitSummary`、`isHead` 和指向该 commit 的 ordered `RepositoryReference[]`。
- local/remote branch 以直接 target OID 关联；annotated tag 优先使用 peeled target OID，lightweight tag 使用直接 target OID。
- tag 指向 tree/blob 或无法与当前页 commit 匹配时不产生 decoration。
- commit 序列和 `nextCursor` 保持 TASK-0006 固定 HEAD 快照语义，分页无重复/遗漏。
- decorations 与 `isHead` 反映每次请求时的当前 refs/HEAD；cursor 不冻结 mutable refs，并在文档中明确。
- merge topology 继续由 ordered `parents` 表达；Core 不输出像素、颜色或 lane 坐标。
- 支持 worktree、linked worktree、detached HEAD、bare 和 empty repository。
- 协议升级至 `1.7`，增加 `commitGraphProjection` capability，同步 Rust、Schema、TypeScript 和文档。
- 测试分页、branch/remote/lightweight/annotated tag decorations、HEAD、merge parents、detached、bare、empty 和 cursor 错误。

## Non-goals

- UI lane routing、坐标、颜色、折叠或虚拟列表策略。
- graph 搜索/过滤、arbitrary revision ranges、all-refs reachability 或 unreachable commits。
- branch/tag 写操作、checkout、fetch/push。
- GitHub、PR、Squash Trace、SQLite 或 Host UI。

## Deliverables

- [ ] `repository/graph` 分页 projection
- [ ] commit/ref decoration 关联与 HEAD 标记
- [ ] Rust、Schema、TypeScript、capability 同步
- [ ] pagination/decorations/merge/detached/bare/empty/error 测试
- [ ] graph projection、mutable decorations 与 UI 边界文档

## Review Checklist

- [ ] commit 分页复用固定快照契约，无重复实现 Host 业务逻辑。
- [ ] annotated/lightweight tag 与 branch decoration OID 选择正确。
- [ ] merge parents 顺序不变，HEAD 状态明确。
- [ ] Core 不输出 UI lane/像素/颜色策略。
- [ ] 未实现 branch/tag 写操作、fetch/push 或 GitHub。
- [ ] fmt、Clippy、Rust tests、`npm run check` 与文档检查通过。

## Done Definition

- [ ] Deliverables 和 Review Checklist 完成。
- [ ] 自主 Review 无阻塞项。
- [ ] 状态更新 Done，提交、推送并合并回 `main`。

## References

[Paginated History](../docs/HISTORY.md) · [Repository References](../docs/REFERENCES.md) · [Protocol](../docs/PROTOCOL.md) · [Roadmap](../docs/ROADMAP.md#phase-2--local-git-foundation当前)
