# Product Requirements

## 产品目标

GitNova 为开发者提供本地优先、跨 Host 一致且更具解释力的 Git 使用体验。它的核心差异化能力是将本地 Git 历史与托管平台 PR 事实关联，让 Squash Merge 后仍可追溯 PR 的原始 commits 与逐行变更。愿景与受众见[愿景文档](VISION.md)。

## MVP 核心用户需求

当 PR 以 Squash Merge 方式合并后，用户能够从该 PR：

1. 查看合并前的原始 commit 列表，而不只看到最终的 squash commit。
2. 选择任一原始 commit，查看它修改的文件和具体行级 diff。
3. 查看 PR、原始 commits 与最终 squash commit 之间的可解释关系（Squash Trace）。

该工作流是 MVP 用于验证产品核心价值的必备能力，不是 Post-MVP 增强项。GitHub 是 MVP 的首个托管平台 Provider；其他平台可在后续阶段扩展。

## 核心需求

- 用户无需 GitNova 中心账户或中心服务器即可使用核心功能。
- Desktop、VS Code、JetBrains、Visual Studio 均通过 `gitnova-core` 获得一致业务语义。
- Core 作为本地独立进程，通过 JSON-RPC/stdio 服务单个 Host 会话。
- Git 操作使用 System Git；MVP 的 GitHub Provider 可通过 `gh`、REST 或 GraphQL 接入。
- 本地数据使用 SQLite，并区分事实数据与可重建派生数据。
- 用户应明确知道何时执行外部命令、访问网络或修改仓库。

## MVP 原则

MVP 必须交付 Desktop 端到端主路径：可靠的 Core 协议、必要的本地 Git 能力、GitHub Provider、PR 原始 commits、per-commit 文件与行级 diff，以及 Squash Trace。IDE Host 与其他托管平台不阻塞首个 MVP。每项功能仍必须按[路线图](ROADMAP.md)和[功能清单](FEATURE_LIST.md)进入单独 Task；TASK-0001 只锁定范围，不实现这些业务能力。

## 非目标

- 中心化仓库托管、云端 Core 或 Web SaaS。
- 替代 GitHub/GitLab 等代码托管平台。
- 自研 Git 实现或绕过用户的 System Git。
- 在 Host 内实现或复制业务逻辑。
- 实时多人协作、项目管理或 CI/CD 平台。
- TASK-0001 中实现 Repository、Git Status、Commit、Diff、Branch、Graph、GitHub API、PR 或 Squash Trace 业务功能（这些中的 MVP 项目将由后续独立 Task 实现）。

质量约束见[非功能需求](NON_FUNCTIONAL.md)，系统边界见[架构](ARCHITECTURE.md)。
