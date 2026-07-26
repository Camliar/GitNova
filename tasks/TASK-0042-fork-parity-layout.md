# TASK-0042: Fork-parity Desktop Layout

- **Status:** Done
- **Priority:** P1
- **Owner:** Codex
- **Branch:** `codex/task-042-fork-parity-layout`
- **Dependencies:** completed TASK-0041 baseline (`d249ab5`)

## Goal

按参考图收敛 Desktop 的仓库工作台：顶部仓库与分支上下文、左侧仓库导航与 refs 树、中央高密度列表/文件区、底部上下文详情或提交栏。

## Scope

- 顶部栏以仓库选择、分支选择、添加/重新打开和刷新为核心，不渲染未实现的 Git 命令。
- 左侧保留 Local Changes / All Commits / Pull Requests / Settings，并使用 Core `repository/references` 展示 Branches、Remotes 和 Tags 树。
- 本地分支在左侧与顶部均可发起同一安全切换确认；远程跟踪分支和 Tags 只读。
- All Commits 上半部是可滚动高密度 timeline，下半部使用 Commit / Changes 标签展示 metadata、文件和 line diff。
- Local Changes 左中部是 staged/unstaged 变更列表，右侧是大尺寸 diff，底部是紧凑 commit composer；AI 仍只在 commit 区按需展开。
- 使用平台中性图标、色彩与文案，不复制 Fork 商标、专有图标或未授权素材。
- 更新响应式规则、交互测试、Desktop 文档与 UI 指南。

## Non-goals

- 逐像素复制 Fork 或使用 Fork 的商标资源。
- 新增 fetch、pull、push、stash、submodule 或 worktree 业务协议。
- 为分支树新增历史过滤语义；本 Task 只展示 refs 和切换本地分支。
- AI Provider 扩展（延后至 TASK-0043）。

## Deliverables

- [x] compact command/context toolbar
- [x] repository navigation with Branches/Remotes/Tags tree
- [x] Fork-style history list and tabbed lower detail pane
- [x] Fork-style local changes list, large diff, and compact commit composer
- [x] responsive and keyboard-accessible interaction states
- [x] tests, docs, browser QA, and native DMG verification

## Review Checklist

- [x] Every visible action maps to an implemented Host/Core capability.
- [x] Ref tree is projected only from Core references and preserves local/remote/tag distinctions.
- [x] Local branch switching shares explicit confirmation and refresh semantics.
- [x] History selection and file selection never restore stale responses.
- [x] AI remains absent outside the commit composer disclosure.
- [x] Desktop tests, production build and macOS bundle pass.

## Verification

- `./node_modules/.bin/tsc --noEmit` (Desktop): pass
- `./node_modules/.bin/vitest run` (Desktop): 65 tests pass
- `./node_modules/.bin/vite build` (Desktop): pass
- `cargo fmt --all -- --check`: pass
- `cargo test --workspace`: 42 unit + 36 contract tests pass
- protocol, quality and delivery check scripts: pass
- in-app browser at 1440×900: setup shell renders without overflow or console warnings; repository workbench behavior is covered through the mocked Core integration tests because a browser page cannot start the Tauri sidecar
- native x86_64 `.app` contains Desktop and Core Mach-O executables plus valid `Info.plist`; final DMG passes `hdiutil verify`
- DMG SHA-256: `64acbce82877505dd2b00dfc252846384044f70f24a29285263d90a152a08594`

## Done Definition

- [x] 自主 Review 无阻塞项，状态 Done，提交推送并快进合并 main。
