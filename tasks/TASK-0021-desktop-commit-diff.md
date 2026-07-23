# TASK-0021: Desktop Commit Detail and Diff

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/021-desktop-commit-diff`
- **Estimate:** 3 days
- **Dependencies:** TASK-0020 Desktop commit history (`dd258c9`), structured commit diff protocol `1.11`

## Goal

让 Desktop 用户从 commit timeline 选择一个 commit，查看完整 metadata/message，并通过 Core `repository/commitDiff` 浏览该 commit 相对明确 parent edge 的文件与行级结构化 diff。

## Scope

- timeline commit 提供明确 View commit 操作，只使用 Core projection 给出的完整 OID 和 parents。
- root commit 自动使用 empty-tree comparison；single-parent commit 省略 parentOid 由 Core 选择唯一 parent。
- merge commit 请求前必须由用户明确选择一个 direct parent；Host 不猜测 first parent 或 preferred parent。
- 使用共享 `CommitDiffParams`/`CommitDiff`/`FileDiff` 类型，固定 3 行 context。
- 展示完整 OID、ordered parents、author/committer identity 与 timestamp、完整 message 和实际比较 parent。
- 展示 ordered changed files；用户选择文件后呈现复用的 structured hunks/line numbers、rename path、binary 和 empty states。
- loading 期间禁用重复 commit 操作；选择另一个 commit 使旧 response 失效。
- 请求失败保留 timeline 和 commit selection，支持对相同 oid/parent Retry。
- repository reopen/history snapshot reload 清除旧 commit detail；status Refresh 不清除 commit detail。
- 测试 root/single/merge parent 参数、metadata/message、files/hunks、binary/empty、错误/retry 和 stale response。
- 更新 Desktop 与 Commit Diff 文档。

## Non-goals

- combined merge diff、自动 preferred parent、arbitrary tree/range compare。
- stage/unstage/discard、checkout、cherry-pick、revert 或 patch apply。
- GitHub PR commit diff、remote-only object fetch 或 Squash Trace UI。
- Host 解析 commit objects、patch 或读取 repository filesystem。

## Deliverables

- [x] typed repository/commitDiff client boundary
- [x] commit selection, explicit merge-parent choice, loading/error state
- [x] commit metadata, changed files, and reusable structured diff rendering
- [x] frontend tests and documentation

## Review Checklist

- [x] oid/parents 只来自 Core graph；merge parent 必须由用户明确选择。
- [x] root/single/merge 请求参数符合 Core contract，Host 不推断 Git parent 语义。
- [x] message 和 diff content 只作为 text 渲染；Host 不运行 Git 或解析 patch。
- [x] stale request 不覆盖新 selection；错误保留 timeline 并可 Retry。
- [x] 未提前实现 mutation/GitHub/Squash Trace UI。
- [x] Rust fmt/Clippy/tests、frontend checks 与 Tauri build 通过。

## Done Definition

- [x] Deliverables 和 Review Checklist 完成。
- [x] 自主 Review 无阻塞项。
- [x] 状态更新 Done，提交、推送并快进合并至 `main`。

## References

[Structured Commit Diff](../docs/COMMIT_DIFF.md) · [Commit Graph Projection](../docs/COMMIT_GRAPH.md) · [Structured File Diff](../docs/DIFF.md) · [Desktop Host](../apps/desktop/README.md)
