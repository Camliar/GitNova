# TASK-0015: Desktop Host Foundation

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/015-desktop-host-foundation`
- **Estimate:** 1–2 days
- **Dependencies:** TASK-0014 Squash Trace relationship (`459ed51`), ADR-0002, ADR-0004

## Goal

建立可构建、可测试的 Tauri 2 + React 19 + TypeScript + Vite Desktop Host 基座，为后续 Core transport 和 Squash Trace UI 提供明确的进程/呈现边界。

## Scope

- 将 `apps/desktop` 从占位目录升级为 pnpm workspace package。
- 建立 React 19 + strict TypeScript + Vite 应用入口、语义化应用壳和基础设计令牌。
- 建立最小 Tauri 2 Rust Host crate、配置、build script 与 capability 文件。
- Tauri Rust 层只负责 Host 启动与窗口，不包含 Git/GitHub/PR/Squash Trace 业务逻辑。
- UI 仅呈现 Desktop 基座状态和后续连接入口，不伪造 repository、PR 或 Squash Trace 数据。
- 增加组件 smoke/accessibility-oriented 测试、TypeScript 检查与 Vite production build。
- 将根 workspace check 接入 Desktop 检查，并同步 Desktop/架构文档和依赖锁文件。

## Non-goals

- 启动或监管 `gitnova-core`、JSON-RPC framing、request client 或 capability handshake。
- repository picker、GitHub 网络请求、PR 列表、commit diff 或 Squash Trace 数据展示。
- Git mutation、SQLite、自动更新、签名、installer 或发布流水线。
- VS Code、JetBrains 或 Visual Studio Host。

## Deliverables

- [x] React 19 + Vite Desktop package
- [x] Tauri 2 minimal Host shell and capability configuration
- [x] shared visual tokens and accessible empty state
- [x] frontend tests, typecheck and production build
- [x] workspace scripts, lockfile and documentation

## Review Checklist

- [x] Desktop/Tauri 不包含 Core 业务规则或直接 Git/GitHub 调用。
- [x] CSP 与 Tauri capability 保持最小权限，无 shell/network capability。
- [x] UI 可键盘访问、焦点可见且不以颜色作为唯一状态信息。
- [x] 依赖版本锁定，Rust 与 frontend checks 通过。
- [x] 未提前实现 TASK-0016 Core process transport 或 Squash Trace screen。

## Done Definition

- [x] Deliverables 和 Review Checklist 完成。
- [x] 自主 Review 无阻塞项。
- [x] 状态更新 Done，提交、推送并快进合并至 `main`。

## References

[ADR-0002](../adr/ADR-0002-Tauri.md) · [ADR-0004](../adr/ADR-0004-Core-Process.md) · [Tech Stack](../docs/TECH_STACK.md) · [UI Guideline](../docs/UI_GUIDELINE.md) · [Roadmap](../docs/ROADMAP.md#phase-3--desktop-squash-trace-mvp)
