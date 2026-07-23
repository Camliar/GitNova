# ADR-0005: GitHub Provider 首选 `gh` 适配器

- **Status:** Accepted
- **Date:** 2026-07-23
- **Decision owners:** GitNova maintainers

## Context

Squash Trace MVP 需要 Core 访问 GitHub PR、commit 和 diff 数据，同时保持 Local-first、无 GitNova 账户、无中心服务器，并避免 Host 接触或复制凭据与 Provider 业务逻辑。直接 REST/GraphQL 需要 GitNova 自行管理 token 生命周期；用户机器上常已有 GitHub CLI 与安全凭据配置。

## Decision

- Core 内定义 GitHub Provider 边界，首个适配器通过 `gh api` 调用 GitHub REST/GraphQL。
- Host 只能显式触发 Provider 请求；Core 不自动联网、不后台刷新。
- Core 不调用 `gh auth token`、不要求 Host 传 token，也不记录命令 stderr、响应正文或凭据。
- `gh` 继承用户在仓库环境中的标准凭据配置；Core 设置非交互与无 pager 环境，直接启动进程而不经 shell。
- 首个 MVP 仅支持 `github.com`；GitHub Enterprise hostname 与直接 HTTP adapter 进入后续独立 Task。
- Provider 的领域响应与稳定错误属于 Core 协议，不把 `gh` 原始 JSON 或文本泄漏给 Host。
- `gh` 不可用、需要认证、请求失败和响应无效必须是不同的稳定错误；不得根据可能含敏感信息的 stderr 文本构造协议错误。

## Consequences

正面：复用用户已有认证与安全存储；无需 GitNova 账户或中心服务；Core/Host 边界清晰；后续可在不改 Host 的情况下增加 REST/GraphQL adapter。代价：首个 MVP 依赖仓库环境安装并配置 `gh`，且错误细节需要保守归一化。

## Alternatives considered

- **Host 直接调用 `gh`：** 会在每个 Host 复制 Provider 与凭据处理逻辑，拒绝。
- **Core 自管 personal access token：** 增加凭据采集、存储与轮换责任，首个 MVP 拒绝。
- **立即实现 REST 与 GraphQL 双 adapter：** 扩大当前 Task 和测试面，延后。
- **GitNova Cloud proxy：** 违反 Local-first 与无中心服务器约束，拒绝。

## Links

[Architecture](../docs/ARCHITECTURE.md) · [Local First](ADR-0003-Local-First.md) · [Core Process](ADR-0004-Core-Process.md) · [Product Requirements](../docs/PRODUCT_REQUIREMENTS.md)
