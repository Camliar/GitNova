# ADR-0007: GitLab Provider 首选 `glab api` 适配器

- **Status:** Accepted
- **Date:** 2026-07-26
- **Decision owners:** GitNova maintainers

## Context

Post-MVP 需要把 Squash Trace 的 original commits、per-commit diff 与最终 commit 关系扩展到 GitLab.com 和 Self-Managed，同时保持 Local-first、无 GitNova 账户、无中心代理和唯一 Core 业务层。

## Decision

- GitLab Provider 位于 Core，首个 adapter 通过仓库环境中的 `glab api` 访问 GitLab REST API。
- Core 从已验证 remote 同时解析 hostname 与多级 project namespace；Self-Managed 请求必须显式传递该 hostname，path 使用百分号编码，不允许参数或 endpoint 注入。
- `glab` 使用非交互模式并继承用户现有认证；Core 不读取、传递或持久化 token，不把 stderr 或原始响应交给 Host。
- Merge Request detail、完整 original commit 列表和指定 commit diff 分开请求。Core 先验证 OID 属于 MR，再请求 diff；受服务端限制的 patch 显式标为 unavailable。
- `squash_commit_sha` 是 Provider 明确证据，优先于 `merge_commit_sha`；缺少明确策略时才结合本地 commit parents 保守分类。
- Host 仅调用版本化协议并展示领域结果；不得直接调用 `glab` 或实现 GitLab 规则。

## Consequences

GitLab.com 与 Self-Managed 可复用同一 Core 能力和用户已有凭据，不引入 GitNova 服务。代价是仓库环境必须安装并配置 `glab`，且不同 GitLab 版本的 diff 限制必须保守呈现。

## Alternatives considered

- **Host 直接调用 GitLab：** 会复制业务与凭据边界，拒绝。
- **Core 自管 token/直连 REST：** 增加凭据生命周期责任，当前阶段拒绝。
- **GitNova Cloud proxy：** 违反 Local-first，拒绝。

## Links

[Architecture](../docs/ARCHITECTURE.md) · [GitHub Provider](ADR-0005-GitHub-Provider.md) · [Protocol](../docs/PROTOCOL.md)
