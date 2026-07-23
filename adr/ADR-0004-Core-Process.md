# ADR-0004: Core 独立进程与 JSON-RPC/stdio

- **Status:** Accepted
- **Date:** 2026-07-23
- **Decision owners:** GitNova maintainers

## Context

多种 Host 使用不同语言和运行时。Core 必须能独立测试、复用和故障隔离，同时不能为了本地调用引入监听端口、后台 Server 或 Cloud 服务。

## Decision

`gitnova-core` 以 Host 启动的**本地独立子进程**运行：

- 使用 JSON-RPC 2.0 请求、响应与通知模型。
- 使用 stdin/stdout 传输逐帧消息，stderr 专用于结构化诊断。
- 不开放 TCP/HTTP 监听端口，不注册系统常驻服务。
- 一次 Host 会话拥有明确的 Core 生命周期；异常退出可检测、可重启。
- 协议包含版本握手、能力协商、稳定错误码、取消和超时语义。
- stdout 禁止输出协议外文本；敏感数据不得进入日志。

## Consequences

正面：跨语言、跨 Host；部署简单；无端口安全面；故障隔离；CLI 级可测试。代价：需要 framing、背压、并发、取消和子进程监管；协议兼容需要严格治理。

## Alternatives considered

- **动态库/FFI：** 性能高，但语言绑定、ABI 和崩溃隔离成本高，拒绝。
- **localhost HTTP/gRPC：** 工具成熟，但引入端口、发现、鉴权和 Server 生命周期，拒绝。
- **Host 内嵌 Core：** 破坏独立性并使不同 Host 重复集成，拒绝。

## Links

[Architecture](../docs/ARCHITECTURE.md) · [Non-functional Requirements](../docs/NON_FUNCTIONAL.md) · [ADR-0001](ADR-0001-Architecture.md) · [ADR-0003](ADR-0003-Local-First.md)

