# TASK-0020: Desktop Commit History

- **Status:** In Progress
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/020-desktop-commit-history`
- **Estimate:** 2–3 days
- **Dependencies:** TASK-0019 Desktop structured file diff (`31bcb17`), commit graph projection protocol `1.11`

## Goal

在 Desktop 打开任意 repository 后，通过 Core `repository/graph` 呈现可分页的 commit history、HEAD 和 refs，建立后续 commit diff 与 PR/Squash Trace 导航入口。

## Scope

- repository 打开后自动请求 `repository/graph` 第一页，固定 page size 30。
- 使用共享 `CommitGraphPage`/`CommitGraphNode` 类型；Host 不调用 Git，也不关联 refs 与 commits。
- 按 Core 顺序展示 summary、短 OID、author、author timestamp、parent 数量、HEAD 与 local/remote/tag decorations。
- 使用 Core opaque `nextCursor` 显式 Load more，追加页面且不解析、持久化或合成 cursor。
- 支持 worktree、linked worktree、detached、bare 和 empty/unborn repository。
- 第一页 loading/error 与增量 loading/error 分离；Load more 失败保留已加载 commits 并可重试。
- 防止重复加载和 stale response；同仓库 reopen 及 repository identity 变化时重置 history snapshot。
- status Refresh 不重置 history，避免将 working-tree snapshot 与 commit snapshot 耦合。
- 测试第一/后续页参数、顺序、decorations、merge parents、empty、bare、错误与 retry。
- 更新 Desktop、History 与 Commit Graph 文档。

## Non-goals

- graph lane routing、虚拟滚动、搜索、任意 ref/range 或 reflog。
- commit diff、文件列表、checkout、branch/tag mutation、fetch/pull/push。
- GitHub PR、original commits、Squash Trace 或 provider 请求。
- Host 解析 cursor、commit objects、refs 或 Git 输出。

## Deliverables

- [ ] typed repository/graph client boundary
- [ ] commit metadata and decoration list
- [ ] opaque cursor pagination with robust loading/error states
- [ ] frontend tests and documentation

## Review Checklist

- [ ] history ordering、HEAD、parents 和 decorations 全部来自 Core projection。
- [ ] cursor 保持 opaque，分页 append 无重复触发或 stale overwrite。
- [ ] bare、empty、detached 状态不依赖 working-tree status。
- [ ] 增量失败保留既有 commits；无 Git、filesystem 或权限扩张。
- [ ] 未提前实现 commit diff/GitHub/Squash Trace UI。
- [ ] Rust fmt/Clippy/tests、frontend checks 与 Tauri build 通过。

## Done Definition

- [ ] Deliverables 和 Review Checklist 完成。
- [ ] 自主 Review 无阻塞项。
- [ ] 状态更新 Done，提交、推送并快进合并至 `main`。

## References

[Commit Graph Projection](../docs/COMMIT_GRAPH.md) · [Paginated Commit History](../docs/HISTORY.md) · [Desktop Core Transport](../docs/DESKTOP_CORE_TRANSPORT.md) · [Desktop Host](../apps/desktop/README.md)
