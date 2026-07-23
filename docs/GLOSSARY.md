# Glossary

| 术语 | 定义 |
| --- | --- |
| GitNova | 本项目及其多 Host 本地 Git 产品平台（暂定名） |
| Host | 承载 UI、用户输入和平台集成的客户端：Desktop、VS Code、JetBrains 或 Visual Studio；不是业务层 |
| Core | `gitnova-core`，本地独立 Rust 进程和唯一业务能力层 |
| Local First | 核心体验在本机、可离线运行，用户数据默认不离开设备 |
| JSON-RPC | Host 与 Core 之间的版本化 RPC 消息协议 |
| stdio | Core 使用标准输入/输出与其 Host 交换消息的传输方式 |
| System Git | 用户环境中安装并配置的 Git 可执行程序 |
| `gh` | GitHub 官方 CLI；MVP GitHub Provider 的一种可选适配路径 |
| Original commit | PR 在 Squash Merge 之前包含的单个 commit |
| Squash commit | Squash Merge 在目标分支生成的最终单一 commit |
| Squash Trace | 关联 PR、原始 commits、per-commit diff 与最终 squash commit 的可解释视图 |
| 中心服务器 | 由 GitNova 运营、承载业务或仓库数据的远程服务；本架构明确不存在 |
| 派生数据 | 可从仓库或外部事实源重新计算的本地缓存、索引或视图数据 |
| Task | 一个有范围、禁区、交付物和完成定义的最小工作单元 |
| ADR | Architecture Decision Record，记录重要决策的背景、选择和后果 |
| MVP | 用于验证核心价值的最小可行产品阶段 |

术语在[项目说明](PROJECT.md)、[架构](ARCHITECTURE.md)和 [ADR](../adr/README.md) 中保持相同含义。
