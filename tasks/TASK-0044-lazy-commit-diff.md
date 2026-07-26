# TASK-0044: Lazy Commit Diff and Compact History

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `codex/task-044-lazy-commit-diff`
- **Dependencies:** completed TASK-0043 baseline (`3f5000a`)

## Goal

让超大 commit 也能稳定打开：timeline 始终保持 28px 紧凑行，选中普通/root/merge commit 后先展示 metadata 与有序文件列表，点击文件时才从 Core 加载该文件的行级 diff。

## Context

真实仓库中的 `67819c6b` 修改 3,281 个文件，完整 patch 约 37 MB。旧实现为每个文件单独运行一次 Git diff，并在一个 15 秒、16 MB transport response 中返回所有 hunks，因此出现 `desktop.core_transport_failed`。少量 timeline 行还会被 CSS Grid 平均拉伸到填满上半区。

## Scope

- 协议 1.16 增加 optional `lazyCommitDiff` capability、`repository/commitFiles` 和 `repository/commitFileDiff`。
- `repository/commitFiles` 只返回 commit、选定 parent edge 和有序 changed-file metadata；不读取 patch。
- `repository/commitFileDiff` 验证所选 path 确属该 commit/parent edge 后，只返回单文件结构化 diff。
- Desktop 先加载文件列表；Changes 以纵向文件列表展示，点击文件再加载/重试该文件 diff。
- merge commit 在 metadata 与 Changes 中都显示 parent edge 选择；root 与单父 commit 自动选择正确基线。
- timeline 固定 28px 行高且内容顶部排列，不随可用高度拉伸。
- 移除顶部重复的分支选择器；当前分支仅在左侧 Branches 中加粗、着色并带当前标记。
- 保留旧 `repository/commitDiff`，避免破坏现有 Host；Desktop 在 capability 不可用时使用兼容路径。

## Non-goals

- Squash Trace/original commits 自动解析与详情编排（TASK-0045）。
- Fetch/Pull/Push（TASK-0046）。
- 分支右键菜单和新增分支 mutation（TASK-0047）。
- 对 37 MB 完整 patch 做单次 JSON-RPC 返回或在 Host 复制 Git parsing。

## Deliverables

- [x] protocol 1.16 lazy commit diff contract
- [x] Core changed-file listing and single-file diff methods
- [x] Desktop lazy Changes browser and stable error recovery
- [x] 28px compact history and sidebar-only current branch context
- [x] benchmark regression coverage for thousands of changed files
- [x] synchronized docs and tests

## Review Checklist

- [x] File list request has bounded metadata and never embeds patch content.
- [x] File diff path is repository-relative, member-validated and literal-pathspec safe.
- [x] A slow/failed file diff does not terminate history or discard its file list.
- [x] Merge parent selection is explicit and bound to each request.
- [x] No Git/diff semantics move into Desktop.
- [x] Protocol, Rust, Desktop, Host and delivery checks pass.

## Done Definition

- [x] Deliverables and review checklist complete.
- [x] Autonomous review has no blocking findings.
- [x] Status Done; branch committed/pushed and fast-forwarded into verified remote `main`.

## Verification

- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- Desktop TypeScript, Vite build and 72 Vitest tests
- protocol generation, quality, delivery, VS Code, IDEA and Visual Studio checks
- 3,000-file Core contract regression; one selected patch materialized
- real 3,281-file commit discovery: 0.068 s and 239,713 bytes, versus the previous ~37 MB eager patch
