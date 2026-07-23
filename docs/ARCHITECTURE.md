# Architecture

## 目标架构

```text
┌─────────────────────────────────────────────────────────┐
│ Hosts                                                   │
│ Desktop │ VS Code │ JetBrains │ Visual Studio           │
│ UI、输入、呈现、进程生命周期、宿主 API 适配             │
└──────────────────────────┬──────────────────────────────┘
                           │ JSON-RPC 2.0 over stdio
                           ▼
┌─────────────────────────────────────────────────────────┐
│ gitnova-core                                            │
│ 本地独立进程、唯一业务能力层、无 UI、Host 无关           │
└───────────────┬───────────────────────┬─────────────────┘
                │                       │
                ▼                       ▼
        System Git / gh           Local SQLite
        REST / GraphQL（可选）     （本地派生状态）
```

必须理解为 **Host → Core → Git/GitHub**，绝不是 **Desktop → Business Logic**。Desktop 与 IDE 集成都只是 Host；任何可跨 Host 复用的业务规则都属于 Core。

## Core

Core 的正式进程名为 `gitnova-core`，具备以下不可变约束：

- 作为本机子进程独立运行，不链接进 Host 进程。
- 通过 stdin/stdout 交换 JSON-RPC 2.0 消息；stderr 只输出诊断。
- 不监听 TCP/HTTP 端口，无常驻 Server，无 Cloud 依赖。
- 调用用户已经安装的 System Git，而不是重新实现 Git。
- 对 Host 保持无关；同一能力只实现一次。
- 仅在本地 SQLite 中保存必要的设置、索引或可重建派生数据。

进程细节见 [ADR-0004](../adr/ADR-0004-Core-Process.md)。

## Hosts

| Host | 职责 | 不负责 |
| --- | --- | --- |
| Desktop | Tauri 窗口、React UI、Core 生命周期、系统集成 | Git/GitHub 业务规则 |
| VS Code | 编辑器 UI、命令与 Core 生命周期 | Git/GitHub 业务规则 |
| JetBrains | IDE UI、动作与 Core 生命周期 | Git/GitHub 业务规则 |
| Visual Studio | IDE UI、命令与 Core 生命周期 | Git/GitHub 业务规则 |

Host 可以做平台特有的输入映射和呈现，但不能解释领域结果或直接绕过 Core 执行业务操作。

## 数据与集成

- 仓库是事实源，System Git 是 Git 操作边界。
- SQLite 只保存本地数据，并应支持迁移、备份和安全重建。
- GitHub Provider 是 MVP Squash Trace 主路径的一部分，由 Core 通过 `gh`、REST 或 GraphQL 接入；网络集成必须可配置、透明且可禁用，Foundation Task 不实现它。
- Core 负责获取并关联 PR、原始 commits、per-commit diff 与最终 squash commit；Host 只呈现 Core 返回的领域结果。
- 仓库在哪个环境，Core 就运行在哪个环境；该边界后续同样适用于 WSL、Remote SSH 和 Dev Container。
- Core 不把仓库、历史、凭据或遥测发送到 GitNova 中心服务，因为不存在这样的服务。

## 约束与演进

模块间依赖只能向内：Host → 协议 SDK → Core → 外部适配器。协议必须版本化，破坏性变化需 ADR。安全、性能和可观测性要求见[非功能需求](NON_FUNCTIONAL.md)，依赖选型见[技术栈](TECH_STACK.md)。架构选择由 [ADR-0001](../adr/ADR-0001-Architecture.md)、[ADR-0002](../adr/ADR-0002-Tauri.md) 和 [ADR-0003](../adr/ADR-0003-Local-First.md) 固化。
