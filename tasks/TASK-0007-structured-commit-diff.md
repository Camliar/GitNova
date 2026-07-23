# TASK-0007: Structured Commit Diff

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/007-structured-commit-diff`
- **Estimate:** 2–3 days
- **Dependencies:** TASK-0006 paginated commit history (`e8f46f6`)

## Goal

为已打开仓库中的指定 commit 返回结构化文件列表与行级 diff，并明确 root、普通和 merge commit 的 parent 语义，为后续 PR original commit 与 Squash Trace 复用同一领域模型建立本地 Git 基线。

## Scope

- 新增 `repository/commitDiff`，参数包含完整 `oid`、可选 `parentOid` 和可选 `contextLines`。
- `oid` 与 `parentOid` 仅接受 40 或 64 位十六进制完整 object ID；Core 验证目标为 commit，且 parent 必须是其直接 parent。
- root commit 使用空树语义并返回 `parentOid: null`；单 parent commit 默认使用唯一 parent。
- merge commit 未指定 parent 时返回稳定错误 `commit.parent_required`；指定任一直接 parent 后返回相对于该 parent 的 diff。
- 使用 System Git 的 machine-oriented `--name-status -z` 获取 ordered file changes，再逐文件生成 patch 并复用结构化 hunk/line 类型。
- 返回 commit metadata、实际 parent OID 和 ordered `FileDiff[]`；支持 add/delete/modify/rename、binary、空文件和无变更 commit。
- 路径保持仓库相对 UTF-8 字符串；不经 shell、不启用 external diff/textconv。
- 支持 worktree、detached HEAD 和 bare repository；目标 commit 不要求等于 HEAD，但必须存在于当前仓库。
- 协议升级至 `1.5`，增加 `structuredCommitDiff` capability，同步 Rust、Schema、TypeScript 和文档。
- 测试 root/single-parent/merge parent 选择、multifile、rename、binary、empty commit、bare/detached、非法 OID/parent 和 context。

## Non-goals

- staged 或 working-tree diff（由 TASK-0005 提供）。
- combined merge diff、自动选择“最佳”merge parent 或跨多个 parent 聚合。
- arbitrary tree/blob diff、revision range、patch application 或任何写操作。
- graph lane/layout、branch/tag decorations。
- GitHub、PR、remote-only commit、Squash Trace、SQLite 或 Host UI。

## Deliverables

- [x] `repository/commitDiff` 与明确 parent 规则
- [x] NUL-delimited commit file-change parser 与逐文件结构化 patch
- [x] Rust、Schema、TypeScript、capability 与稳定错误同步
- [x] root/merge/multifile/rename/binary/bare/error 契约测试
- [x] commit diff 语义和限制文档

## Review Checklist

- [x] 只调用 System Git，不经 shell、不读取 worktree 内容、无写操作。
- [x] object ID 和 parent relationship 均验证，参数不能注入 revision/pathspec。
- [x] root、single-parent 和 merge parent 语义无歧义。
- [x] 路径、rename、binary、empty file 与 hunk 行号保真。
- [x] 未实现 GitHub、PR、Squash Trace、graph 或写操作。
- [x] fmt、Clippy、Rust tests、`npm run check` 与文档检查通过。

## Done Definition

- [x] Deliverables 和 Review Checklist 完成。
- [x] 自主 Review 无阻塞项。
- [x] 状态更新 Done，提交并推送。

## References

[Paginated History](../docs/HISTORY.md) · [Structured File Diff](../docs/DIFF.md) · [Protocol](../docs/PROTOCOL.md) · [Roadmap](../docs/ROADMAP.md#phase-2--local-git-foundation当前)
