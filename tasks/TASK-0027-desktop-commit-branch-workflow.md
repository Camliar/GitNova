# TASK-0027: Desktop Commit and Branch Workflow

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/027-desktop-commit-branch-workflow`
- **Dependencies:** TASK-0026 (`cb360a5`), Core protocol `1.12`

## Goal

在 Desktop 提供显式确认的 staged commit、创建 local branch 与切换 local branch 工作流，并完全消费 Core mutation 结果。

## Scope

- typed client 调用 `repository/commit`、`repository/createBranch`、`repository/switchBranch` 和只读 `repository/references`。
- commit 展示 staged path 数量、可编辑 message、Review/Confirm 两步操作；不自动 stage。
- branch create 与 switch 均两步确认；switch 选项只来自 Core local branch refs，不猜 remote。
- loading 禁用并防止重复提交；错误保留输入与确认上下文，可 Retry 或 Cancel。
- mutation 成功直接应用 Core snapshot，并刷新 history/graph；清除可能失效的 file/commit diff。
- bare repository 不展示 mutation UI；测试显式确认、参数、snapshot、刷新、错误恢复与无自动 mutation。

## Non-goals

- staging/unstaging、amend、AI message、branch delete/rename、remote tracking、detached checkout。
- reset/restore/stash/merge/rebase/fetch/pull/push 或 Host 内 Git 逻辑。

## Deliverables

- [x] typed mutation client and confirmation state machine
- [x] commit/create/switch responsive UI
- [x] snapshot refresh/error/stale tests and docs

## Review Checklist

- [x] 所有 mutation 都要求第二次明确确认，打开仓库/刷新不会触发写操作。
- [x] Host 不自动 stage/stash/force/discard，不推断 refs 或 Git mutation 结果。
- [x] 成功状态来自 Core snapshot，失败保留上下文且不宣称成功。
- [x] frontend/Rust tests、typecheck、Clippy、protocol check 与 production/Tauri build 通过。

## Done Definition

- [x] 自主 Review 无阻塞项，状态 Done，提交推送并快进合并 main。
