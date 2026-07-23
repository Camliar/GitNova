# Architecture Decision Records

ADR 记录难以逆转或影响多个模块的技术决策。状态使用 Proposed、Accepted、Superseded、Deprecated；已接受 ADR 不直接改写结论，变更应新增 ADR 并链接旧记录。

| ADR | 状态 | 决策 |
| --- | --- | --- |
| [0001](ADR-0001-Architecture.md) | Accepted | Host/Core 分层架构 |
| [0002](ADR-0002-Tauri.md) | Accepted | Desktop 采用 Tauri 2 |
| [0003](ADR-0003-Local-First.md) | Accepted | Local First 且无中心服务器 |
| [0004](ADR-0004-Core-Process.md) | Accepted | Core 独立进程与 JSON-RPC/stdio |

提交 ADR 应遵循[贡献指南](../CONTRIBUTING.md)和 [`tasks/README.md`](../tasks/README.md)。总体架构见 [`docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md)。

