# Project

## 定位

GitNova 是本地优先的 Git 客户端平台，目标是在不把仓库数据交给中心服务的前提下，为开发者提供更清晰、可解释、跨宿主一致的版本控制体验。核心产品价值是在 Squash Merge 之后仍能从 PR 追溯原始 commits、per-commit 文件与行级 diff，并解释它们与最终 squash commit 的关系。

产品口号是 **Smarter Git. Deeper Insight.**；完整叙事见[愿景](VISION.md)，品牌表达见[品牌指南](BRANDING.md)。

## 项目边界

一个独立的本地进程 `gitnova-core` 是唯一业务能力层。Desktop、VS Code、JetBrains 和 Visual Studio 均为 Host，只负责 UI、生命周期与平台集成。Core 使用 stdio 上的 JSON-RPC 接口，调用 System Git，并在需要时使用本地 SQLite。项目不依赖中心服务器或云端运行时。详见[架构](ARCHITECTURE.md)与 [ADR-0001](../adr/ADR-0001-Architecture.md)。

## 当前阶段

`chore/001-project-foundation` 仅建立 Monorepo、文档、ADR、规范和占位品牌资产。禁止实现任何 Git/GitHub 业务功能，详见[功能清单](FEATURE_LIST.md#foundation-task-明确禁止)。

## 成功标准

- 新贡献者可从 [README](../README.md) 独立理解产品、架构和下一步。
- 所有 Host 共享 Core 能力，不复制业务规则。
- 默认离线可用，仓库内容不离开本机。
- Desktop MVP 能完成 PR original commits、per-commit diff 与 Squash Trace 主路径。
- 技术决策、Task 和变更都有可追溯记录。
- 产品质量满足[非功能需求](NON_FUNCTIONAL.md)。

目录职责见[项目结构](PROJECT_STRUCTURE.md)，演进节奏见[路线图](ROADMAP.md)。
