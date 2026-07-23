# TASK-0023: Desktop PR Original Commit Diff

- **Status:** In Progress
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/023-desktop-pr-commit-diff`
- **Dependencies:** TASK-0022 (`9fd94e0`), GitHub PR commit diff protocol `1.11`

## Goal

从当前 PR original commit 列表选择 commit，通过 Core 展示其远程文件、统计与行级 patch。

## Scope

- OID 仅来自当前 PR commits；请求绑定 PR number 与 normalized nameWithOwner。
- 显示 ordered files、status、additions/deletions/changes、rename path。
- `patchState=available` 呈现 structured hunks；`unavailable` 明确说明 Provider 未提供 patch，不猜测 binary。
- 显式 View diff、loading 禁用、错误保留 PR/selection 并 Retry；新 PR 清除旧 diff，stale response 不回写。
- 测试参数、顺序、patch/unavailable、错误/retry；更新文档。

## Non-goals

- arbitrary remote commit、binary 内容、Squash Trace relationship、本地/远程 patch 比较、仓库写操作。

## Deliverables

- [ ] typed client and PR-member selection
- [ ] remote file statistics and structured patch UI
- [ ] loading/error/stale tests and docs

## Review Checklist

- [ ] membership、completeness 与 patch semantics 由 Core 保证，Host 不推断。
- [ ] 无自动网络、credentials、raw JSON/stderr 或 mutation。
- [ ] 未提前实现 Squash Trace UI。
- [ ] Rust/frontend tests、Clippy 与 Tauri build 通过。

## Done Definition

- [ ] 自主 Review 无阻塞项，状态 Done，提交推送并快进合并 main。
