# TASK-0050: Colored Curved Commit Timeline

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `codex/task-050-colored-curved-timeline`
- **Dependencies:** completed TASK-0049 baseline (`83460ff`)

## Goal

按 Fork 参考布局优化 commit timeline：不同拓扑 lane 使用稳定的区分色，branch/merge 连接改为平滑曲线，同时严格保持 28 px 单行密度和现有整行选择交互。

## Decisions

- Core 继续是 commit topology 的唯一事实来源；Desktop 只把 Core 投影的 lane/edge 坐标映射为 SVG 路径和设计令牌颜色。
- lane 颜色由非语义的稳定调色板按 lane index 取模，不声称颜色等同于 branch identity；同一 lane 在已加载页面中保持一致。
- vertical edge 保持直线；额外 parent 从节点横向圆润转出、垂直接入目标 lane，第一父回流使用两端垂直切线的 cubic Bézier 曲线，并以 round cap/join 保持 Fork 风格。
- 当前 commit node 使用所属 lane 色；选中行保留全宽蓝色背景，并提高图线/节点对比度。
- 继续使用 Core 提供的有限 graph width，不在 Host 重算 Git DAG、分支归属或 merge 语义。

## Scope

- timeline SVG lane palette and curved edge rendering.
- compact clipping/alignment and selected-row contrast.
- graph renderer unit tests and synchronized UI/Desktop docs.

## Non-goals

- Core graph protocol/algorithm changes、branch identity recoloring、interactive graph editing。
- repository tabs/toolbars（TASK-0049 已完成）或新增 Git mutation。

## Done Definition

- [x] Different lanes render with stable distinguishable colors.
- [x] Cross-lane edges render as Fork-style curves without increasing row height.
- [x] Merge topology and accessibility regressions pass.
- [x] Autonomous review has no blocking findings.

## Verification

- `cargo test --workspace` (99 tests)
- `cargo clippy --workspace --all-targets -- -D warnings`
- Desktop TypeScript, Vite production build and 87 Vitest tests
- focused graph projection tests for linear continuity, merge branching, first-parent rejoin, palette stability and accessible non-color description
- protocol generation, quality and delivery checks
