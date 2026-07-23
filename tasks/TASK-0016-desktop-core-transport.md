# TASK-0016: Desktop Core Process Transport

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/016-desktop-core-transport`
- **Estimate:** 2–3 days
- **Dependencies:** TASK-0015 Desktop Host foundation (`2025f6c`), ADR-0004, Core protocol `1.11`

## Goal

让 Desktop Host 安全启动、握手、调用并终止独立 `gitnova-core` 子进程，通过统一 JSON-RPC 2.0 `Content-Length` stdio transport 建立后续所有 Desktop 功能的唯一 Core 通道。

## Scope

- 在 Tauri Rust Host 层实现 `CoreSupervisor`，直接启动独立 `gitnova-core`，stdin/stdout/stderr 全部 pipe，不调用 shell。
- 默认从 Desktop 可执行文件同目录解析 Core；开发/测试允许通过 `GITNOVA_CORE_BINARY` 显式覆盖绝对路径。
- 启动后立即发送 `gitnova/initialize`，要求协议主版本兼容并校验所需 Core capabilities。
- 串行化 JSON-RPC requests，生成单调递增整数 id，执行 16 MiB framing 上限并验证 response id/envelope。
- stderr 在后台排空但不转发仓库内容或凭据；stdout 只作为协议读取。
- 公开最小 Tauri commands：查询连接状态、启动 Core、发送通用 Core request、优雅 shutdown。
- shutdown 发送 `gitnova/shutdown` 和 `exit`；窗口退出/异常路径保证 kill + wait，不遗留子进程。
- 将稳定、非敏感 transport/lifecycle 错误映射给 React；不透传 child stderr 或 raw OS error。
- React foundation 状态接入真实 supervisor status，并提供显式 retry；不请求 repository/GitHub 数据。
- 测试 framing、id 匹配、初始化能力、异常 EOF、oversize、shutdown/drop 与 frontend state。

## Non-goals

- repository picker/open、Git status、GitHub、PR、commit diff 或 Squash Trace 请求。
- 在 Host 中解释 Core 领域结果或复制稳定业务错误。
- remote/WSL/SSH/Dev Container Core launcher、sidecar 打包、签名或 installer。
- request multiplexing、通知订阅、取消 UI 或自动重启策略。

## Deliverables

- [x] Core supervisor and child lifecycle
- [x] framed JSON-RPC request/response transport
- [x] initialize/version/capability validation
- [x] minimal Tauri command boundary and sanitized errors
- [x] React connection status/retry state
- [x] Rust/frontend tests and transport documentation

## Review Checklist

- [x] Core 仍是独立进程，Host 未承载 Git/GitHub/Squash Trace 业务逻辑。
- [x] 无 shell、端口、daemon 或 Tauri shell plugin。
- [x] stdout framing、id、size、EOF 和 lifecycle failures fail closed。
- [x] shutdown/drop 不遗留 child，stderr 不泄漏到 UI。
- [x] 未提前实现 repository 或 Squash Trace screen。
- [x] fmt、Clippy、Rust tests、frontend checks 与 Tauri build 通过。

## Done Definition

- [x] Deliverables 和 Review Checklist 完成。
- [x] 自主 Review 无阻塞项。
- [x] 状态更新 Done，提交、推送并快进合并至 `main`。

## References

[ADR-0004](../adr/ADR-0004-Core-Process.md) · [Core Protocol](../docs/PROTOCOL.md) · [Architecture](../docs/ARCHITECTURE.md) · [Desktop Host](../apps/desktop/README.md)
