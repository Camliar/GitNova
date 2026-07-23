# ADR-0001: Host/Core 分层架构

- **Status:** Accepted
- **Date:** 2026-07-23
- **Decision owners:** GitNova maintainers

## Context

GitNova 需要同时服务 Desktop、VS Code、JetBrains 与 Visual Studio。若各 Host 自行实现 Git 与产品规则，行为、错误处理和演进速度会迅速分叉，也难以独立测试。

## Decision

采用 **Host → `gitnova-core` → Git/GitHub** 的分层架构。

- Host 只负责 UI、输入、呈现、生命周期和宿主 API 适配。
- `gitnova-core` 是唯一业务能力层，为无 UI、Host 无关的本地独立进程。
- Core 通过 System Git 执行 Git 操作；MVP 的 GitHub Provider 可通过 `gh`、REST 或 GraphQL 接入。
- Host 不得直接实现或复制 Git/GitHub 业务规则。

明确拒绝 **Desktop → Business Logic** 的架构。Desktop 只是多个 Host 之一。

## Consequences

正面：跨 Host 语义一致；Core 可独立测试和发布；平台适配与业务演进解耦。代价：必须维护版本化 RPC 协议、进程生命周期和兼容性矩阵；简单功能也需要跨进程调用。

## Alternatives considered

- **每个 Host 独立实现：** 初期直接，但导致重复、漂移和测试成本，拒绝。
- **把业务逻辑放进 Tauri backend：** 将 Desktop 提升为特殊架构中心，其他 Host 无法复用，拒绝。
- **中心服务承载业务：** 违背隐私、离线和本地优先目标，拒绝。

## Links

[Architecture](../docs/ARCHITECTURE.md) · [Tech Stack](../docs/TECH_STACK.md) · [ADR-0003](ADR-0003-Local-First.md) · [ADR-0004](ADR-0004-Core-Process.md)
