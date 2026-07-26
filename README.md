# GitNova

> **Smarter Git. Deeper Insight.**<br>
> 更聪明的 Git 客户端，洞察每一次提交背后的故事。

GitNova 是一个本地优先、面向多种开发工具宿主的现代 Git 客户端。它将本地 Git 历史与托管平台 PR 数据关联：即使 PR 已经 Squash Merge，用户仍能查看原始 commit 列表、每个 commit 修改的文件和行级 diff，以及它们与最终 squash commit 的关系。

桌面端与 IDE 扩展只负责交互和宿主集成；独立的 `gitnova-core` 进程承载全部 Git、GitHub/GitLab、PR/MR 与 Squash Trace 领域能力，并通过 stdio 上的 JSON-RPC 与 Host 通信。

> 当前仓库已包含可执行的 Core 协议、本地 Git 读模型与安全 mutation、Core-owned GitHub/GitLab Squash Trace 与 AI Assist，以及可构建的 Tauri 2 + React 19 Desktop Host。Desktop 可查看 status、commit graph 与 diff，经确认执行 staged commit/local branch 及 Fetch/fast-forward Pull/non-force Push，显式查看 PR original commits、远程 patch 与最终 commit 关系证据，并可预览 AI 输入、生成/编辑 commit 草稿后交给独立 commit 确认；Core 也可运行在 WSL、Remote SSH 或 Dev Container 的仓库环境。

## 架构

```text
Desktop · VS Code · JetBrains · Visual Studio
                    │
             JSON-RPC / stdio
                    ▼
      gitnova-core（本地独立进程）
                    │
 System Git · gh · glab · AI Provider · SQLite
```

Host 是展示和适配层，不承载业务逻辑。Core 在用户设备上独立运行；产品没有中心服务器，也不要求云端运行时。详见[架构说明](docs/ARCHITECTURE.md)和[架构决策](adr/ADR-0001-Architecture.md)。

## 快速开始

需要 Rust stable 与 Node.js 22+：

```bash
git clone <repository-url> GitNova
cd GitNova
cargo test --workspace
npm run check
```

Desktop 前端可单独使用 `pnpm --filter @gitnova/desktop check` 验证；原生无 bundle 构建使用 `pnpm --filter @gitnova/desktop tauri build --no-bundle`，当前平台安装包使用 `npm run bundle:desktop`。

`npm run check` 同时执行协议、隐私/网络/交付静态质量门、前端测试与 production build；完整 Rust、性能和平台矩阵见[质量基线](docs/QUALITY_BASELINE.md)，安装包与签名流程见[Desktop 发布指南](docs/DESKTOP_RELEASE.md)。

阅读[项目总览](docs/PROJECT.md)、[产品需求](docs/PRODUCT_REQUIREMENTS.md)、[Core 协议](docs/PROTOCOL.md)、[Repository Mutations](docs/MUTATIONS.md)、[Desktop Core Transport](docs/DESKTOP_CORE_TRANSPORT.md)、[本地诊断日志](docs/DIAGNOSTICS.md)、[仓库发现](docs/REPOSITORIES.md)、[工作区状态](docs/STATUS.md)、[结构化 Diff](docs/DIFF.md)和[贡献指南](CONTRIBUTING.md)，再从 `tasks/` 选择已批准的 Task。

## Monorepo

- `apps/`：Desktop、VS Code、JetBrains、Visual Studio Host
- `crates/`：`gitnova-core` 及 Rust 协议类型
- `packages/`：生成的 TypeScript 协议类型与后续共享包
- `sdk/`：Host/Core 协议 JSON Schema 事实源
- `docs/`、`adr/`：活文档与不可变架构决策
- `tasks/`：Task 规范、模板和交付记录
- `assets/`：Logo、图标和品牌资产

完整说明见[目录结构](docs/PROJECT_STRUCTURE.md)。

VS Code Host 已提供 workspace 与 Squash Trace vertical slice；使用与边界见 [`apps/vscode/README.md`](apps/vscode/README.md)。

JetBrains Host 已提供 IntelliJ Platform plugin project 与 Squash Trace action；构建和远端 backend 约束见 [`apps/idea/README.md`](apps/idea/README.md)。

Visual Studio Host 已提供进程外扩展工程与 Squash Trace command；GitLab.com/Self-Managed MR Provider 位于 Core，使用与边界分别见 [`apps/visualstudio/README.md`](apps/visualstudio/README.md)和 [ADR-0007](adr/ADR-0007-GitLab-Provider.md)。

最终阶段 AI Assist 已完成 Core 与 Desktop 主路径：可预览 staged 输入范围，并通过本地 Ollama 或用户配置的 OpenAI、Claude、DeepSeek、Qwen、Kimi 直连生成可编辑草稿与结构化建议；AI 不会自动执行 commit。详见 [AI Assist Contract](docs/AI_ASSIST.md)。

## MVP Roadmap

MVP 将按“基础设施 → Core 协议 → 本地 Git 基础 → Desktop Squash Trace 端到端体验 → 发布质量”的顺序推进。GitHub Provider、PR original commits、per-commit diff 和 squash relationship 都是 MVP 必备能力。范围和阶段见[路线图](docs/ROADMAP.md)与[功能清单](docs/FEATURE_LIST.md)。

## Non-goals

- 不建设中心服务器、云端 Git 执行环境或强制账户体系。
- 不在 Host 中复制 Core 业务逻辑。
- 不替代 System Git，也不内嵌自研 Git 实现。
- Core Contract 不预先实现任何 Git 或 GitHub 业务能力。

更多边界见[产品需求](docs/PRODUCT_REQUIREMENTS.md#非目标)和[非功能需求](docs/NON_FUNCTIONAL.md)。

## Brand & License

品牌资产和使用方式见[品牌指南](docs/BRANDING.md)。本项目暂以 [MIT License](LICENSE) 发布；正式发布前复核名称、商标和第三方许可。
