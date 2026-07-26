# TASK-0041: Multi-repository Workbench

- **Status:** Done
- **Priority:** P1
- **Owner:** Codex
- **Branch:** `codex/task-041-multi-repository-workbench`
- **Dependencies:** completed TASK-0040 baseline (`525622c`)

## Goal

让 Desktop 的日常仓库操作更接近 Fork：可以在多个已打开仓库之间切换，在顶部直接切换本地分支，并收紧工作区标题、变更文件与 commit timeline 的交互密度。

## Scope

- 将单一仓库 bookmark 升级为带当前项的最近仓库列表，并兼容 v1 bookmark 迁移。
- 应用启动时恢复上次活动仓库；用户可从顶部仓库选择器切换或添加仓库。
- 在顶部工作台提供分支选择器，分组展示全部本地分支和远程跟踪分支，复用 Core 现有的引用查询与安全切换 RPC。
- 分支切换仍需显式确认，不自动 stash、force 或丢弃工作区改动。
- 移除左侧重复的 Worktree 标题块，仓库身份仅保留在顶部选择器。
- 变更文件通过点击文件行查看 diff，移除 View 按钮；同时保留 staged/working 范围的可预期选择。
- commit timeline 进一步压缩行高、图标和 metadata 占用。
- 更新测试、Desktop 文档与交互指南。

## Non-goals

- 同时运行多个 Core 进程或并行查询多个仓库。
- 跨 Core 环境同时保持多个活动仓库会话。
- 新增 fetch、pull、push、stash 或远程分支 checkout 协议。
- AI Provider 扩展（延后至 TASK-0042）。

## Deliverables

- [x] multi-repository recent list and startup restoration
- [x] repository switcher and add-repository action
- [x] grouped local/remote branch browser and local switcher with confirmation
- [x] duplicate Worktree title removal
- [x] clickable change rows without View buttons
- [x] denser commit timeline
- [x] interaction tests, docs and native package verification

## Review Checklist

- [x] Existing v1 repository bookmark migrates without losing the last repository.
- [x] Switching repository resets stale diff, commit detail and AI draft state.
- [x] Switching branch is explicit, safe, and refreshes status/history/references.
- [x] All local and remote-tracking branches returned by Core are visible and clearly distinguished.
- [x] Staged and working-tree diffs remain distinguishable for mixed-status files.
- [x] Compact timeline remains keyboard accessible and preserves refs/author/SHA/time.
- [x] Desktop tests, production build and macOS bundle pass.

## Done Definition

- [x] 自主 Review 无阻塞项，状态 Done，提交推送并快进合并 main。

## Verification

- `./node_modules/.bin/tsc --noEmit`
- `./node_modules/.bin/vitest run` (12 files, 65 tests)
- `./node_modules/.bin/vite build`
- `node scripts/check-quality.mjs`
- `node scripts/check-delivery.mjs`
- `node scripts/generate-protocol.mjs --check`
- Browser layout QA at 1280×720 with no horizontal overflow or console warnings/errors
- Tauri release DMG build and `hdiutil verify`
