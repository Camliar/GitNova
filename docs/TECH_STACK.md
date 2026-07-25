# Tech Stack

## 锁定技术栈

| 层 | 技术 | 用途 |
| --- | --- | --- |
| Frontend | React 19、TypeScript、Vite | Desktop WebView UI 与可复用 UI 基础 |
| Desktop | Tauri 2 | 原生窗口、打包和本地进程管理 |
| Backend/Core | Rust stable | `gitnova-core` 独立本地进程 |
| Storage | SQLite | 本地设置、索引与可重建派生数据 |
| Git | System Git | 唯一 Git 执行实现 |
| GitHub | `gh`、REST、GraphQL | MVP Provider 的可选适配路径 |
| GitLab | `glab api` | GitLab.com/Self-Managed Provider adapter |
| AI（最终阶段） | System `curl`、Ollama loopback、OpenAI Responses API 直连（可选） | Core-owned commit draft 与结构化建议；无 GitNova proxy |
| Protocol | JSON-RPC 2.0 over stdio、`Content-Length` framing | Host/Core 通信 |
| Workspace | Cargo workspace、pnpm workspace | Rust 与 TypeScript Monorepo |

## 选择原则

- Tauri 的选择和限制见 [ADR-0002](../adr/ADR-0002-Tauri.md)。
- Core 必须是独立 Rust 进程，不能变成 Tauri command 集合，见 [ADR-0004](../adr/ADR-0004-Core-Process.md)。
- GitNova 不内嵌 Git 实现，不绕过 System Git。
- `gh`、REST 与 GraphQL 是 MVP GitHub Provider 的互补适配路径，不属于 Foundation Task 实现范围。
- `glab api` 复用用户在仓库环境中的 GitLab authentication，不向 Host 暴露 token。
- AI 模型由用户配置；OpenAI key 仅由 Core 从仓库环境读取，Ollama endpoint 限制在 loopback。具体边界见 [ADR-0008](../adr/ADR-0008-AI-Assist-Providers.md)。
- 依赖需锁定、审计许可证和安全公告；新增跨层框架需要 ADR。

架构映射见[架构说明](ARCHITECTURE.md)，协议细节见[Core 协议](PROTOCOL.md)，编码约束见[编码规范](CODING_STANDARD.md)。Rust 依赖版本由 `Cargo.lock` 锁定。
