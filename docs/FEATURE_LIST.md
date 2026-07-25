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
- Core staged commit 与 local branch create/switch，以及 Desktop 显式确认 workflow（已实现）
- 固定 HEAD 快照的分页提交历史（已实现）
- 指定 commit-parent 的文件列表与行级 diff（已实现）
- HEAD、local/remote branches 与 tags 的只读枚举（已实现）
- commits、parents 与 ref decorations 的 Core graph projection（已实现）
- Tauri 2 + React 19 Desktop Host 基座（已实现）
- Desktop 提交图谱可视化（已实现）
- GitHub Provider repository identity/metadata（Core-owned `gh api` adapter，已实现）
- GitHub PR detail 与 ordered original commits（已实现）
- PR 查看与原始 commit 列表（Desktop 已实现）
- 指定 PR 原始 commit 的文件列表与可用行级 diff（已实现；缺失 patch 显式标记）
- PR、原始 commits 与最终 merge commit 的保守关联模型（Squash Trace Core，已实现）
- Squash Trace Desktop 关联展示（已实现）
- Desktop Host 的 Squash Trace 端到端关键工作流（已实现）
- MVP 本地质量门、冷启动性能基线与网络透明性（已实现）
- Desktop Core sidecar 打包、跨平台 CI 与受控 draft release（已实现；正式签名凭据由发布者配置）

## Post-MVP 候选

- WSL、Remote SSH 与 Dev Container 结构化 Core launcher（已实现）
- VS Code Host 的 workspace 与 Squash Trace vertical slice（已实现）
- JetBrains Host 的 project 与 Squash Trace vertical slice（已实现）
- Visual Studio Host 的进程外 Squash Trace vertical slice（已实现）
- GitLab.com/Self-Managed Provider 的 MR original commits、per-commit diff 与 Squash Trace（已实现）
- 超出 MVP 主路径的 PR 操作与协作工作流
- 基础主路径之外的搜索、筛选与历史洞察
- 高级可视化、扩展协议与企业策略适配

## 最终阶段：AI Assist

- 根据 staged diff 生成可编辑 commit message 草稿
- 根据仓库状态提供拆分 commit、测试与冲突处理建议
- 本地模型或用户配置的直连 AI Provider
- 输入预览、敏感路径排除、最小披露与用户确认
- AI 不自动执行 commit 或 reset/rebase/push 等高风险操作
- AI Assist 协议、Ollama/OpenAI Provider 与隐私契约（已实现）
- Core staged input preview、commit draft 与结构化操作建议（已实现）
- Desktop 输入披露、生成、草稿编辑与 commit 二次确认（已实现）

AI Assist 必须排在 Squash Trace MVP、交付质量及 Post-MVP Host/Provider 之后，不得反向阻塞这些阶段。

## Foundation Task 明确禁止

本 Task 不得包含以下任何业务实现：**Repository、Git Status、Commit、Diff、Branch、Graph、GitHub API、PR、Squash Trace**。这一禁止只限定 TASK-0001 的交付边界，不改变 GitHub Provider、PR original commits、per-commit diff 和 Squash Trace 属于产品 MVP 的事实。

功能边界见[产品需求](PRODUCT_REQUIREMENTS.md)，实现顺序见[路线图](ROADMAP.md)，架构约束见[架构说明](ARCHITECTURE.md)。
