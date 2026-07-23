# TASK-0024: Desktop Squash Trace

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/024-desktop-squash-trace`
- **Dependencies:** TASK-0023 (`cc344cf`), Squash Trace protocol `1.12`

## Goal

在当前 PR 中显式请求 Core 的 Squash Trace，并以保守、可解释的方式展示 original commits 与最终 merge commit 的关系。

## Scope

- 从已加载 PR 显式触发 `github/squashTrace`，请求绑定 PR number 与 normalized nameWithOwner。
- 展示 classification、confidence、merge commit OID、本地可用性、parents 与 Core evidence。
- `squashCandidate` 只呈现为候选关系，不宣称已验证 squash；未知与缺失状态不得猜测。
- loading 禁用、错误保留 PR 并可 Retry；切换 PR 清除旧 trace，stale response 不回写。
- 测试显式触发、参数、关系语义、错误/retry 与 PR 切换；更新文档。

## Non-goals

- Host 推断 merge strategy、比较 patch、任意 commit 查询、缓存、Git mutation、提交图谱视觉改造。

## Deliverables

- [x] typed client and explicit trace request
- [x] conservative relationship and evidence UI
- [x] loading/error/stale tests and docs

## Review Checklist

- [x] 所有 Git/GitHub/relationship 语义来自 Core，Host 只展示协议值。
- [x] 无自动网络、credentials、raw JSON/stderr 或 mutation。
- [x] `medium` confidence 与 candidate 文案不被升级为确定事实。
- [x] frontend tests、typecheck、protocol check 与 production build 通过；本轮环境无 `cargo`，Rust/Tauri 原生验证未重跑（无 Rust/Tauri Rust 改动）。

## Done Definition

- [x] 自主 Review 无阻塞项，状态 Done，提交推送并快进合并 main。
