# TASK-0052: Privacy-safe Diagnostic Log

- **Status:** Done
- **Priority:** P1
- **Owner:** Codex
- **Branch:** `codex/task-052-diagnostic-log`
- **Dependencies:** completed TASK-0051 baseline (`fc95a75`)

## Goal

为 Desktop Host 增加本地、可轮转、可定位的诊断日志，帮助排查 Core 启动、transport 超时、Provider/AI 稳定错误等问题，同时保持 GitNova 的 Local-first 与敏感数据边界。

## Decisions

- 日志由 Desktop Host 写入 Tauri 平台 app log directory；不会上传、遥测或自动导出。
- 使用 JSON Lines，每条只包含时间戳、级别、事件、Core environment、RPC method、耗时、protocol version 与稳定错误码等白名单字段。
- 禁止写入 repository path、commit message、diff、RPC params/result、Core/Provider stderr、Provider response、用户名、邮箱、token、API key 或其他凭据。
- 单个 active log 最大 1 MiB，并保留一个上一代文件；轮转或写入失败不得阻塞 Core 与 UI。
- Settings 展示 active log 的本机路径、轮转上限和隐私说明，便于用户自行定位后提供日志。

## Scope

- Desktop diagnostic writer、轮转和单元测试。
- Core lifecycle 与 allowlisted JSON-RPC request outcome/duration instrumentation。
- Settings diagnostic location presentation。
- 同步运维与隐私文档。

## Non-goals

- 记录业务 payload、完整错误 message、stdout/stderr 或 shell command。
- 网络 telemetry、远程日志服务、自动上传或 GitNova 账户。
- Core 业务逻辑、Provider 行为或协议能力变更。
- UI 内查看、搜索、删除或打包日志。

## Done Definition

- [x] app start、Core configure/start/shutdown 和 RPC outcome 产生稳定 JSONL 事件。
- [x] RPC 日志不含 params/result，只记录 allowlisted method、duration 和稳定错误码。
- [x] active log 超过 1 MiB 时轮转，并仅保留一个 previous log。
- [x] 任何日志 I/O 错误都不会使 Core lifecycle/request 失败。
- [x] Settings 可展示当前日志路径与隐私边界。
- [x] 自动测试验证事件字段、轮转、redaction-by-construction 和失败降级。
- [x] 自主 Review 无阻塞问题。

## Verification

- `cargo test --workspace` (104 tests)
- `cargo clippy --workspace --all-targets -- -D warnings`
- Desktop TypeScript typecheck, Vite production build and 89 Vitest tests
- protocol generation, quality/privacy and delivery configuration checks
- macOS x86_64 ad-hoc signed DMG: `GitNova_0.1.0_x64.dmg`
- DMG SHA-256: `fa370e1bb9e411e0d33c3832698e1dcc98699ea8320982e6b7b36e6cfbb2472f`
