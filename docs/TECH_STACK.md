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
| Protocol | JSON-RPC 2.0 over stdio | Host/Core 通信 |
| Workspace | Cargo workspace、pnpm workspace | Rust 与 TypeScript Monorepo |

## 选择原则

- Tauri 的选择和限制见 [ADR-0002](../adr/ADR-0002-Tauri.md)。
- Core 必须是独立 Rust 进程，不能变成 Tauri command 集合，见 [ADR-0004](../adr/ADR-0004-Core-Process.md)。
- GitNova 不内嵌 Git 实现，不绕过 System Git。
- `gh`、REST 与 GraphQL 是 MVP GitHub Provider 的互补适配路径，不属于 Foundation Task 实现范围。
- 依赖需锁定、审计许可证和安全公告；新增跨层框架需要 ADR。

架构映射见[架构说明](ARCHITECTURE.md)，编码约束见[编码规范](CODING_STANDARD.md)，版本锁定将在首个实现 Task 中以 lockfile 完成。
