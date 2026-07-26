# TASK-0049: Repository Tabs and Fork-style Toolbar

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `codex/task-049-repository-tabs-toolbar`
- **Dependencies:** completed TASK-0048 baseline (`be6345d`)

## Goal

按用户提供的 Fork 参考图，把已添加仓库改为稳定顺序的顶部 Tab，并将 Add/Fetch/Pull/Push/Refresh/Reopen 操作放到主内容左上工具栏；左侧导航顶部显示当前仓库名称。

## Decisions

- 第一行是操作工具栏，第二行是 repository Tab strip；不恢复 branch selector。
- Tab 表示 Desktop 本地保存的 opaque path/Core launch target。点击 Tab 仍通过现有单 active Core 安全切换，不并行访问多个仓库。
- 已有 Tab 切换不改变 Tab 顺序；新添加仓库追加到末尾并成为 active，最多保留 12 个。
- `+` 与 Add repository 复用现有显式 picker/path workflow；不自动扫描仓库。
- Fetch/Pull/Push 继续使用 Core-owned mutation 与原确认契约，只改变位置和视觉层级。
- Stash 尚无 Core 契约，本 Task 不制作虚假按钮；Reopen 归入 `More` 风格次要操作。

## Scope

- stable persisted tab order and active-tab switching/rollback.
- two-row toolbar/tab shell and sidebar repository identity.
- accessible tablist semantics, loading/active states and responsive overflow.
- repository switch/add/restore UI regression tests and synchronized docs.

## Non-goals

- parallel Core sessions、tab close/reorder/pinning、stash mutation。
- timeline lane colors/curves（TASK-0050）。

## Done Definition

- [x] Multiple repositories render and switch as stable tabs.
- [x] Repository actions are left-aligned in the first toolbar row.
- [x] Restore/add/switch failure behavior remains safe and tested.
- [x] Autonomous review has no blocking findings.

## Verification

- `cargo test --workspace` (99 tests)
- `cargo clippy --workspace --all-targets -- -D warnings`
- Desktop TypeScript, Vite production build and 85 Vitest tests
- protocol generation, quality and delivery checks
- browser-rendered setup shell plus repository-tab DOM, arrow-key navigation, stable-order and rollback regressions
