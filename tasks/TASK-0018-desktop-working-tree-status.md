# TASK-0018: Desktop Working Tree Status

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/018-desktop-working-tree-status`
- **Estimate:** 2–3 days
- **Dependencies:** TASK-0017 Desktop repository open (`02544a2`), working tree status protocol `1.11`

## Goal

在 Desktop 打开非 bare 仓库后，通过 Core `repository/status` 呈现分支、上游差异和结构化文件状态，交付第一个只读本地 Git 工作区视图。

## Scope

- `repository/open` 成功后自动请求一次 `repository/status`，并提供用户显式 Refresh。
- 使用共享 `WorkingTreeStatus` 类型；Host 不运行或解析 Git porcelain。
- 展示 branch/detached/unborn、upstream、ahead/behind 和干净工作区状态。
- 按协议顺序展示所有 entries，分别标识 staged 与 working-tree 状态、untracked、conflict 和 rename/copy 原路径。
- bare repository 明确显示不支持 working tree status，不发送 status 请求。
- 加载期间禁用重复 Refresh；刷新失败保留 repository identity，并展示稳定、非敏感错误与重试。
- 测试自动加载、clean、mixed staged/unstaged、rename、conflict、bare、刷新与错误状态。
- 更新 Desktop 与 status 文档。

## Non-goals

- 文件 watcher、轮询或后台自动刷新。
- stage/unstage/discard、commit、diff、文件内容预览或冲突解决。
- branch checkout、fetch/pull/push、history、GitHub PR 或 Squash Trace UI。
- Host 执行 Git、解析 status code 或读取 repository filesystem。

## Deliverables

- [x] typed repository/status client boundary
- [x] branch/divergence and structured change list UI
- [x] clean, bare, loading, refresh, and error states
- [x] frontend tests and documentation

## Review Checklist

- [x] 所有 Git facts 来自 Core；Host 仅映射协议枚举到展示文案。
- [x] staged 与 working-tree 状态保持独立，rename/conflict/untracked 不丢失。
- [x] bare 不调用 status；失败不丢失已打开仓库 identity。
- [x] 无 watcher、shell、filesystem 或新增网络权限。
- [x] 未提前实现 diff/mutation/history/GitHub/Squash Trace UI。
- [x] Rust fmt/Clippy/tests、frontend checks 与 Tauri build 通过。

## Done Definition

- [x] Deliverables 和 Review Checklist 完成。
- [x] 自主 Review 无阻塞项。
- [x] 状态更新 Done，提交、推送并快进合并至 `main`。

## References

[Working Tree Status](../docs/STATUS.md) · [Repository Discovery](../docs/REPOSITORIES.md) · [Desktop Core Transport](../docs/DESKTOP_CORE_TRANSPORT.md) · [Desktop Host](../apps/desktop/README.md)
