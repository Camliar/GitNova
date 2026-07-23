# TASK-0017: Desktop Repository Selection and Open

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/017-desktop-repository-open`
- **Estimate:** 2–3 days
- **Dependencies:** TASK-0016 Desktop Core transport (`d5f5ea1`), repository discovery protocol `1.11`

## Goal

让 Desktop 用户通过原生目录选择器明确选择一个仓库路径，并仅经由 Core 的 `repository/open` 方法打开和展示仓库身份，建立 Desktop 数据主路径的第一个端到端切片。

## Scope

- 使用 Tauri 原生目录选择能力；Host 只获得用户明确选择的路径，不扫描文件系统。
- Core 未连接时先显式启动并完成 TASK-0016 握手，再发送 `repository/open`。
- 使用共享 TypeScript 协议类型发送 `RepositoryPathParams` 并呈现 `RepositoryDescriptor`。
- 展示 worktree、linked worktree 与 bare 类型，以及 Core 返回的根路径、Git directory 和 System Git 版本。
- 将 Core 稳定领域错误映射为清晰 UI 状态；不展示 raw stderr，不声称失败时仓库被修改。
- 支持用户取消选择、打开中禁用重复操作，以及同一仓库的幂等重新打开。
- 为选择、成功、取消、Core 启动失败和 repository/open 错误补充前端测试。
- 更新 Desktop/仓库文档和当前能力说明。

## Non-goals

- 最近仓库、自动扫描、clone、init、拖放或文件选择。
- 在同一 Core 会话切换仓库；选择不同仓库需后续 session workflow Task。
- 工作区 status、branch、remote、history、diff、GitHub PR 或 Squash Trace UI。
- Host 检查 `.git`、执行 Git 或解释 repository 类型。
- WSL、Remote SSH、Dev Container 路径转换或远程 picker。

## Deliverables

- [x] native directory selection with minimum Tauri permission
- [x] typed repository/open client boundary
- [x] repository loading, success, cancellation, and error UI
- [x] frontend tests and documentation

## Review Checklist

- [x] 路径来自明确用户选择，Host 不扫描、不运行 Git、不解析 `.git`。
- [x] Repository facts 全部来自独立 Core，Host 仅呈现协议字段。
- [x] 无 shell、filesystem 或 network plugin 权限扩张。
- [x] 取消不产生错误，请求期间不可重复触发。
- [x] 未提前实现 status/history/GitHub/Squash Trace UI。
- [x] Rust fmt/Clippy/tests、frontend checks 与 Tauri build 通过。

## Done Definition

- [x] Deliverables 和 Review Checklist 完成。
- [x] 自主 Review 无阻塞项。
- [x] 状态更新 Done，提交、推送并快进合并至 `main`。

## References

[Repository Discovery](../docs/REPOSITORIES.md) · [Desktop Core Transport](../docs/DESKTOP_CORE_TRANSPORT.md) · [Core Protocol](../docs/PROTOCOL.md) · [Desktop Host](../apps/desktop/README.md)
