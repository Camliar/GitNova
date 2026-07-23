# TASK-0002: Core Contract

- **Status:** Review
- **Priority:** P0
- **Owner:** Unassigned
- **Branch:** `feature/002-core-contract`
- **Estimate:** 2–3 days
- **Dependencies:** TASK-0001 approved foundation baseline (`8145fe7`)

## Goal

交付一个可独立启动、可由自动化测试驱动的最小 `gitnova-core` 进程，锁定 Host/Core 的 JSON-RPC 2.0 over stdio 基础契约，使后续 Git、GitHub、PR 与 Squash Trace 能力可在不改变进程边界的前提下增量接入。

## Context

GitNova 的所有 Host 都必须通过独立 Core 获得业务能力。[ADR-0004](../adr/ADR-0004-Core-Process.md) 已锁定 JSON-RPC 2.0 over stdio、版本握手、能力协商、稳定错误码、取消和超时语义；本 Task 将这些约束落成可执行契约，但不接入任何 Git 或托管平台业务。

## Scope

- 创建 Rust `gitnova-core` 二进制与独立的协议类型 crate，并加入 Cargo workspace。
- 使用 `Content-Length` header framing 在 stdin/stdout 上传输 UTF-8 JSON-RPC 2.0 消息；stdout 不得输出协议外文本。
- 定义并实现 `gitnova/initialize`、`gitnova/shutdown` 和 `exit` 生命周期；未成功初始化前拒绝业务请求。
- 握手交换 Host/Core 实现信息、协议版本与能力集；不兼容的主版本必须返回稳定错误。
- 定义协议错误 envelope、GitNova 稳定错误码命名规则，以及不泄露敏感数据的 `data` 字段。
- 支持 `$/cancelRequest` 通知；对可取消请求返回明确的 cancellation 错误。
- 定义 Host 超时、取消与 Core 异常退出的责任边界。
- 编写协议文档、JSON Schema 事实源和首个 TypeScript SDK 类型生成/一致性检查；生成产物不包含业务规则。
- 为 framing、请求/通知、握手、版本拒绝、结构化错误、取消、shutdown 和 stdout 纯度提供单元与子进程契约测试。
- 记录 Windows、macOS 与 Linux 的进程启动/终止预期，以及 WSL、Remote SSH、Dev Container 的 Core 定位原则；本 Task 不实现远程 Host 适配。

## Protocol Baseline

- 协议初始版本为 `1.0`；主版本表示兼容边界，次版本通过能力协商增量演进。
- JSON-RPC request id 允许 string 或 integer，Core 不得将 id 强制转换为特定类型。
- 单个 stdio 会话只允许一次成功 initialize；shutdown 后只接受 `exit`。
- Host 负责设定超时并发送取消；Core 负责停止可取消工作并终结对应请求。
- stderr 可输出诊断，但不得包含 token、凭据、仓库内容或 diff。

## Non-goals

- Repository 发现、Git Status、Commit、Diff、Branch 或 Graph。
- GitHub Provider、身份验证、PR 数据、original commits 或 Squash Trace。
- Desktop、VS Code、JetBrains 或 Visual Studio Host 集成。
- SQLite schema、缓存、索引或持久化迁移。
- TCP/HTTP Server、daemon、中心服务、遥测或账户系统。
- 完整的 SDK 运行时、UI 或任何后续 Task 的占位业务 method。

## Deliverables

- [x] `gitnova-core` 及协议 crate 骨架
- [x] 版本化 JSON-RPC/stdio 协议文档与 JSON Schema
- [x] initialize/shutdown/exit、能力协商、错误与取消实现
- [x] TypeScript 协议类型生成或一致性验证
- [x] Rust 单元测试与跨进程契约测试
- [x] Core 生命周期、Host 责任和跨环境定位文档
- [x] 相关 README、项目结构、技术栈和路线图状态同步

## Review Checklist

- [x] Core 是独立 Rust 进程，Host 不承载或复制协议/业务规则。
- [x] 仅使用 JSON-RPC 2.0 over stdio，未引入端口、daemon 或中心服务。
- [x] 协议版本、能力、错误、取消和生命周期具有可执行契约测试。
- [x] stdout 纯度与敏感数据日志约束已测试或静态验证。
- [x] Rust 和 TypeScript 公共类型与 JSON Schema 一致。
- [x] Windows、macOS、Linux 的差异已记录，无硬编码单平台路径假设。
- [x] 未实现 Git、GitHub、PR、Squash Trace、SQLite 或 Host UI 能力。
- [x] `cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings` 与 `cargo test --workspace` 通过。
- [x] Markdown 链接、JSON Schema、生成结果和目录说明已验证。

## Done Definition

- [x] Deliverables 全部完成。
- [x] Review Checklist 全部通过且无未声明范围扩张。
- [ ] 非作者 Reviewer 批准并合并对应 PR。
- [ ] Task 状态更新为 Done。

## Notes

- TASK-0002 的协议骨架不应预测后续业务 method；新能力由对应 Task 扩展协议。
- 若实现中证明 `Content-Length` framing、版本模型或 Schema 事实源需改变，先更新 ADR-0004 并完成 Review，不得静默偏离。

## References

[Roadmap](../docs/ROADMAP.md#phase-1--core-contract) · [Architecture](../docs/ARCHITECTURE.md) · [ADR-0004](../adr/ADR-0004-Core-Process.md) · [Non-functional Requirements](../docs/NON_FUNCTIONAL.md) · [Coding Standard](../docs/CODING_STANDARD.md)
