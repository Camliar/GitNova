# TASK-0006: Paginated Commit History

- **Status:** In Progress
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/006-paginated-commit-history`
- **Estimate:** 2–3 days
- **Dependencies:** TASK-0005 structured file diff baseline (`49e1513`)

## Goal

为已打开的 worktree 或 bare repository 返回基于固定 HEAD 快照的分页 commit 历史，包含完整父关系、author/committer 和 message，为后续 commit detail/diff、图谱和 PR original commit 关联建立稳定领域类型。

## Scope

- 新增 `repository/history`，params 为可选 `limit` 和 `cursor`。
- `limit` 默认 50，范围 1–200。
- 首页将当前 HEAD OID 锁定为快照；后续 opaque cursor 同时编码 snapshot OID 和 offset，仓库新提交不得改变已开始的分页序列。
- 使用 System Git `rev-list --topo-order --date-order --max-count --skip <snapshot>` 生成 OID 页，使用 `git cat-file commit` 读取原始 commit。
- 不使用人类化 `git log` 分隔符解析，避免 commit message 与自定义 delimiter 冲突。
- commit 返回 `oid`、ordered `parents`、author、committer、summary 和完整 message。
- identity 包含 name、email 和 Git 记录的 ISO-8601 timestamp；不应用本机时区重写。
- 支持 merge commit、detached HEAD 和 bare repository。
- unborn/empty repository 返回空 page 与 `nextCursor: null`。
- cursor 仅由 Core 生成和解析；无效 cursor 返回 `history.invalid_cursor`。
- 非 UTF-8 identity/message 返回 `history.unsupported_encoding`，不静默损坏内容。
- 协议升级至 `1.4`，增加 `paginatedCommitHistory` capability，同步 Rust/Schema/TypeScript。
- 测试分页无重复/遗漏、快照稳定、merge parents、author/committer、multiline message、bare、detached、empty 和 cursor 错误。

## Non-goals

- commit 修改文件列表、patch 或行级 diff。
- graph lane/layout、branch/tag decorations、搜索或过滤。
- arbitrary revision range、reflog 或 unreachable objects。
- commit、amend、rebase、reset、checkout 或任何写操作。
- GitHub、PR、Squash Trace、SQLite 或 Host UI。

## Deliverables

- [ ] `repository/history` 与快照 cursor
- [ ] raw commit parser 与 commit identity/message 契约
- [ ] Rust、Schema、TypeScript 与 capability 同步
- [ ] pagination/merge/bare/detached/empty/error 契约测试
- [ ] commit history 语义、cursor 和刻意限制文档

## Review Checklist

- [ ] 只调用 System Git，不经 shell，无写操作。
- [ ] cursor 稳定、opaque、可验证，分页无重复/遗漏。
- [ ] raw commit parser 不受 multiline message 或 gpgsig header 影响。
- [ ] merge parent 顺序、timestamp 和 author/committer 保真。
- [ ] 未实现 commit diff、graph layout、GitHub 或写操作。
- [ ] fmt、Clippy、Rust tests、`npm run check` 与文档检查通过。

## Done Definition

- [ ] Deliverables 和 Review Checklist 完成。
- [ ] 自主 Review 无阻塞项。
- [ ] 状态更新 Done，提交并推送。

## References

[Roadmap](../docs/ROADMAP.md#phase-2--local-git-foundation当前) · [Repositories](../docs/REPOSITORIES.md) · [Protocol](../docs/PROTOCOL.md)
