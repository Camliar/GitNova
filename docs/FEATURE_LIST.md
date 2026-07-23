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
- 单文件 staged/working-tree 行级 diff（已实现）
- 提交与分支操作
- 固定 HEAD 快照的分页提交历史（已实现）
- 指定 commit-parent 的文件列表与行级 diff（已实现）
- HEAD、local/remote branches 与 tags 的只读枚举（已实现）
- commits、parents 与 ref decorations 的 Core graph projection（已实现）
- Tauri 2 + React 19 Desktop Host 基座（已实现）
- Desktop 提交图谱可视化
- GitHub Provider repository identity/metadata（Core-owned `gh api` adapter，已实现）
- GitHub PR detail 与 ordered original commits（已实现）
- PR 查看与原始 commit 列表
- 指定 PR 原始 commit 的文件列表与可用行级 diff（已实现；缺失 patch 显式标记）
- PR、原始 commits 与最终 merge commit 的保守关联模型（Squash Trace Core，已实现）
- Squash Trace Desktop 关联展示
- Desktop Host 的端到端关键工作流

## Post-MVP 候选

- VS Code、JetBrains、Visual Studio Host
- GitLab 等其他托管平台 Provider
- 超出 MVP 主路径的 PR 操作与协作工作流
- 基础主路径之外的搜索、筛选与历史洞察
- 高级可视化、扩展协议与企业策略适配

## 最终阶段：AI Assist

- 根据 staged diff 生成可编辑 commit message 草稿
- 根据仓库状态提供拆分 commit、测试与冲突处理建议
- 本地模型或用户配置的直连 AI Provider
- 输入预览、敏感路径排除、最小披露与用户确认
- AI 不自动执行 commit 或 reset/rebase/push 等高风险操作

AI Assist 必须排在 Squash Trace MVP、交付质量及 Post-MVP Host/Provider 之后，不得反向阻塞这些阶段。

## Foundation Task 明确禁止

本 Task 不得包含以下任何业务实现：**Repository、Git Status、Commit、Diff、Branch、Graph、GitHub API、PR、Squash Trace**。这一禁止只限定 TASK-0001 的交付边界，不改变 GitHub Provider、PR original commits、per-commit diff 和 Squash Trace 属于产品 MVP 的事实。

功能边界见[产品需求](PRODUCT_REQUIREMENTS.md)，实现顺序见[路线图](ROADMAP.md)，架构约束见[架构说明](ARCHITECTURE.md)。
