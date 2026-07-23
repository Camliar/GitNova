# ADR-0003: Local First 且无中心服务器

- **Status:** Accepted
- **Date:** 2026-07-23
- **Decision owners:** GitNova maintainers

## Context

Git 仓库包含高敏感度源码、历史、身份与工作状态。核心 Git 工作天然可在本地完成。依赖中心服务会降低离线可靠性，引入数据治理、延迟、成本、账户和运维负担。

## Decision

GitNova 采用 **Local First**：

- 核心能力在用户设备上运行，离线可用。
- 不建设 GitNova 中心 Server，不提供 Cloud Core，不要求 GitNova 账户。
- 仓库内容和本地派生数据默认不离开设备。
- SQLite 仅作为本地存储；System Git 是仓库操作事实边界。
- 未来 GitHub 网络集成必须由用户明确触发，直接访问相应平台，并可独立禁用。

## Consequences

正面：隐私边界清晰、离线可靠、低延迟、无需中心运维。代价：跨设备同步、集中策略和远程计算不作为默认能力；各平台需处理本地安装、迁移和资源限制。

## Alternatives considered

- **中心服务优先：** 便于同步和统一升级，但违背核心定位，拒绝。
- **强制混合云：** 增加账户与网络依赖，核心功能不需要，拒绝。
- **纯内存本地应用：** 隐私良好但无法可靠保存设置和索引，拒绝；选用本地 SQLite。

## Links

[Vision](../docs/VISION.md) · [Non-functional Requirements](../docs/NON_FUNCTIONAL.md) · [ADR-0001](ADR-0001-Architecture.md) · [ADR-0004](ADR-0004-Core-Process.md)

