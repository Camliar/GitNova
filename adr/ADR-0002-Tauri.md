# ADR-0002: Desktop 采用 Tauri 2

- **Status:** Accepted
- **Date:** 2026-07-23
- **Decision owners:** GitNova maintainers

## Context

Desktop Host 需要跨 Windows、macOS 和 Linux 提供现代 Web UI，同时控制安装体积、系统集成和本地进程安全边界。团队已锁定 React 19、TypeScript 与 Vite 作为前端栈。

## Decision

Desktop Host 采用 **Tauri 2**，承载窗口、WebView、打包、权限和 `gitnova-core` 子进程生命周期。React 19 + TypeScript + Vite 用于 UI。

Tauri Rust 层是 Host 适配层，不是业务层。Git/GitHub 业务逻辑仍只能位于独立 `gitnova-core`。

## Consequences

正面：可复用 Web 技术栈、较小分发体积、明确的能力权限与跨平台路径。代价：需处理各平台 WebView 差异、Tauri capability 配置、签名和打包矩阵；团队需同时维护 Rust 与 TypeScript。

## Alternatives considered

- **Electron：** 生态成熟但运行时和分发体积更大，本阶段不选。
- **纯原生多平台 UI：** 原生体验强，但团队成本和跨平台一致性代价过高。
- **浏览器 Web App：** 无法满足本地 System Git 与进程集成，并引入 Server 依赖。

## Links

[Tech Stack](../docs/TECH_STACK.md) · [Architecture](../docs/ARCHITECTURE.md) · [ADR-0001](ADR-0001-Architecture.md) · [ADR-0004](ADR-0004-Core-Process.md)

