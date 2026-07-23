# Feature List

本清单是产品分期参考，不代表本 Task 已实现任何功能。功能只有在独立 Task 获批、实现并 Review 后才视为存在。

## Foundation

- Monorepo 结构与基础配置
- 产品、架构、技术、规范和品牌文档
- ADR 与 Task 管理流程
- 占位 Logo、图标与 CI 文件

## MVP 必备（后续 Task）

- Core 生命周期、握手、能力协商和结构化错误（已实现）
- 本地仓库发现与打开（已实现）
- 工作区状态（已实现）
- 差异查看、提交与分支操作
- 提交历史与图谱浏览
- GitHub Provider（`gh`、REST 或 GraphQL 适配器，由 Core 统一封装）
- PR 查看与原始 commit 列表
- 指定 PR 原始 commit 的文件列表与行级 diff
- PR、原始 commits 与最终 squash commit 的关联展示（Squash Trace）
- Desktop Host 的端到端关键工作流

## Post-MVP 候选

- VS Code、JetBrains、Visual Studio Host
- GitLab 等其他托管平台 Provider
- 超出 MVP 主路径的 PR 操作与协作工作流
- 基础主路径之外的搜索、筛选与历史洞察
- 高级可视化、扩展协议与企业策略适配

## Foundation Task 明确禁止

本 Task 不得包含以下任何业务实现：**Repository、Git Status、Commit、Diff、Branch、Graph、GitHub API、PR、Squash Trace**。这一禁止只限定 TASK-0001 的交付边界，不改变 GitHub Provider、PR original commits、per-commit diff 和 Squash Trace 属于产品 MVP 的事实。

功能边界见[产品需求](PRODUCT_REQUIREMENTS.md)，实现顺序见[路线图](ROADMAP.md)，架构约束见[架构说明](ARCHITECTURE.md)。
