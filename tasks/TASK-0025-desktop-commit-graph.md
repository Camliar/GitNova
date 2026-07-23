# TASK-0025: Desktop Commit Graph

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/025-desktop-commit-graph`
- **Dependencies:** TASK-0024 (`ce5a7fb`), Core graph projection `1.7`

## Goal

把 Core ordered nodes、ordered parents、HEAD 与 ref decorations 呈现为可读的 Desktop commit graph，同时保持 Git 语义只属于 Core。

## Scope

- 根据当前已加载 graph page 在 Host 内计算纯视觉 lane 与 connector，不调用 Git、不重新关联 refs。
- first parent 延续当前 lane，额外 parents 分叉；已出现/待出现 parent 尽量复用 lane。
- 分页追加后对完整已加载 nodes 重新投影，支持 off-page parent 的 continuation marker。
- graph 颜色不是唯一信息：保留 commit summary、merge parent count、HEAD/ref 文本与可访问标签。
- 空仓库、detached HEAD、root、linear、branch/merge、分页追加与窄屏布局测试。

## Non-goals

- 改变 Core graph contract/order、all-refs history、搜索/过滤、虚拟滚动、交互式折叠。
- Git mutation、GitHub/PR/Squash Trace、commit detail 语义改动。

## Deliverables

- [x] deterministic visual lane projection
- [x] responsive graph renderer integrated with timeline
- [x] linear/branch/merge/pagination/accessibility tests and docs

## Review Checklist

- [x] Host 只计算像素/lane，不推断 branch/tag/HEAD Git 语义。
- [x] ordered node/parent/ref 数据不被修改，commit detail 入口保持可用。
- [x] graph 不只依赖颜色传达信息，窄屏不截断核心文本。
- [x] frontend tests、typecheck、protocol check 与 production build 通过。

## Done Definition

- [x] 自主 Review 无阻塞项，状态 Done，提交推送并快进合并 main。
