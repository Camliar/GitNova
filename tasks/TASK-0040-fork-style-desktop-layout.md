# TASK-0040: Fork-style Desktop Layout

- **Status:** Done
- **Priority:** P1
- **Owner:** Codex
- **Branch:** `codex/task-040-fork-layout`
- **Dependencies:** completed TASK-0039 baseline (`2f7514c`)

## Goal

把 Desktop 从纵向功能卡片集合收敛为接近 Fork 使用习惯的仓库工作台：左侧导航、顶部仓库操作、中央主视图和上下文详情，并让 AI 仅在提交工作流中按需出现。

## Scope

- 已打开仓库时提供 Local Changes、All Commits、Pull Requests 三个主视图入口。
- 使用紧凑顶部工具栏和固定仓库侧栏替代首页式大标题与常驻系统状态卡。
- Local Changes 聚合文件、diff、commit composer，并把 AI Assist 放入 commit composer 的显式展开入口。
- AI Provider、模型、端点与排除路径配置迁移到独立 Settings 视图，commit 区不展示配置表单。
- All Commits 使用紧凑单行 timeline，保留 commit graph、refs、author、SHA、时间、commit metadata、file list 与行级 diff。
- Pull Requests 保留 Provider、original commits 与 Squash Trace 能力。
- 记住上次成功打开的 opaque repository path 与 Core launch target，应用重启后自动恢复该仓库。
- 保持 Core/Host、Local-first、安全确认和现有 RPC 边界不变。

## Non-goals

- 复制 Fork 商标、图标或逐像素视觉设计。
- 为尚未实现的 fetch、pull、push、stash 操作添加无效按钮。
- 修改 Git、GitHub、Squash Trace 或 AI Core 协议。

## Deliverables

- [x] repository workbench information architecture
- [x] compact Local Changes and commit workspace
- [x] AI Assist visible only inside the commit flow
- [x] AI configuration isolated in Settings
- [x] compact single-row commit timeline
- [x] last repository startup restoration
- [x] responsive layout and updated interaction tests
- [x] verified macOS replacement DMG

## Review Checklist

- [x] Open repository workflows and remote Core configuration remain usable.
- [x] Local Changes, history and PR data are no longer rendered as one long page.
- [x] AI configuration is absent outside the explicit commit assistant disclosure.
- [x] Existing Git action confirmations and provider disclosure controls remain intact.
- [x] Previously opened repositories restore without another directory picker and fail safely when unavailable.
- [x] Desktop tests, production build and native bundle pass.

## Done Definition

- [x] 自主 Review 无阻塞项，状态 Done，提交推送并快进合并 main。

## Verification

- `./node_modules/.bin/tsc --noEmit`
- `./node_modules/.bin/vitest run` (12 files, 64 tests)
- `./node_modules/.bin/vite build`
- `node scripts/check-quality.mjs`
- `node scripts/check-delivery.mjs`
- `node scripts/generate-protocol.mjs --check`
- Tauri release DMG build and `hdiutil verify`
- Browser visual QA at 1180×760 with no console warnings or errors
