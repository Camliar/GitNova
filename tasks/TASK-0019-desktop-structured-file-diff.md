# TASK-0019: Desktop Structured File Diff

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/019-desktop-structured-file-diff`
- **Estimate:** 2–3 days
- **Dependencies:** TASK-0018 Desktop working tree status (`57764fe`), structured file diff protocol `1.11`

## Goal

让 Desktop 用户从工作区变更列表选择 staged 或 working-tree scope，并通过 Core `repository/diff` 查看单文件结构化 hunks、行号和内容。

## Scope

- 为 status entry 分别提供可用的 Staged diff 与 Working diff 操作，scope 由对应协议状态决定。
- untracked 文件不合成 diff；明确说明当前结构化 diff 只覆盖 tracked changes。
- 使用共享 `DiffParams`、`DiffScope`、`FileDiff` 类型；默认请求 3 行 context。
- 展示 old/new path、binary 状态、无变化状态、hunk ranges/function header，以及 context/addition/deletion 的新旧行号。
- 保留 Core 返回的行内容，仅通过文本节点渲染；不解释 ANSI、HTML 或仓库控制内容。
- 请求期间禁用重复 diff 操作；选择新 scope 替换当前结果。
- diff 失败保留工作区 status 与文件选择上下文，展示稳定错误并允许 Retry。
- status Refresh 后关闭旧 diff，避免展示过期 snapshot 对应结果。
- 测试 scope 参数、mixed changes、untracked、text/binary/empty diff、行号、错误与 retry。
- 更新 Desktop 与 Diff 文档。

## Non-goals

- repository-wide diff、untracked 内容合成、word diff、image diff 或 side-by-side 布局。
- stage/unstage/discard、编辑、commit、冲突解决或外部 diff tool。
- commit diff、history、GitHub PR commit diff 或 Squash Trace UI。
- Host 运行 Git、解析 unified patch 或读取文件。

## Deliverables

- [x] typed repository/diff client boundary
- [x] per-scope change selection and loading/error state
- [x] structured text, binary, and empty diff presentation
- [x] frontend tests and documentation

## Review Checklist

- [x] path 与 scope 仅来自 Core status entry 和用户明确操作。
- [x] Host 不执行 Git、不解析 patch、不读取文件；内容安全作为 text 渲染。
- [x] staged/workingTree scope 不混淆；untracked 不请求 synthetic diff。
- [x] status Refresh 清除旧 diff；错误不丢失 status snapshot。
- [x] 未提前实现 mutation/commit diff/history/GitHub/Squash Trace UI。
- [x] Rust fmt/Clippy/tests、frontend checks 与 Tauri build 通过。

## Done Definition

- [x] Deliverables 和 Review Checklist 完成。
- [x] 自主 Review 无阻塞项。
- [x] 状态更新 Done，提交、推送并快进合并至 `main`。

## References

[Structured File Diff](../docs/DIFF.md) · [Working Tree Status](../docs/STATUS.md) · [Desktop Core Transport](../docs/DESKTOP_CORE_TRANSPORT.md) · [Desktop Host](../apps/desktop/README.md)
