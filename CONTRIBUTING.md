# Contributing to GitNova

开始前请阅读[项目说明](docs/PROJECT.md)、[架构](docs/ARCHITECTURE.md)、[编码规范](docs/CODING_STANDARD.md)和 [Task 规范](tasks/README.md)。未关联已批准 Task 的范围性实现不应进入 PR。

## Branch

分支名格式为 `<type>/<task-id>-<short-description>`：

- `feature/`：新增用户能力
- `fix/`：缺陷修复
- `refactor/`：不改变行为的重构
- `chore/`：工程、文档、依赖和维护

使用小写 kebab-case，例如 `chore/001-project-foundation`。不要直接向默认分支提交。

## Commit

使用 [Conventional Commits](https://www.conventionalcommits.org/)：

```text
<type>(<optional-scope>): <imperative summary>
```

常用 type：`feat`、`fix`、`refactor`、`docs`、`test`、`build`、`ci`、`chore`。一个提交只表达一个可审查意图；破坏性变更使用 `!` 并在正文说明迁移方式。

## Pull Request

- 使用 [PR 模板](.github/pull_request_template.md)，关联 Task 并声明范围外内容。
- PR 保持小而完整；同步文档、测试、协议或 ADR。
- 作者完成自查并回应 Review；作者不能批准自己的 PR。
- 至少一名非作者 Reviewer 批准，且所有阻塞意见解决后才能合并。
- 涉及架构、协议、安全、持久化或跨 Host 边界时，需要对应领域 Owner Review。
- 合并策略由仓库设置决定；提交仍需符合 Conventional Commits。

## Review Rule

Reviewer 核查正确性、边界、可测试性、安全、可访问性和文档，不仅检查格式。任何把业务逻辑放入 Host、引入中心 Server、绕过 System Git 或破坏 Local First 的变更都必须阻塞，除非新 ADR 明确取代现有决策。

## Task Rule

每个 Task 必须包含目标、Scope、Non-goals、Deliverables、Review Checklist 与 Done Definition。执行中发现新增范围时，拆分后续 Task；不要静默扩大 PR。Task 完成后保留记录并标为等待 Review。模板见 [`tasks/templates/TASK_TEMPLATE.md`](tasks/templates/TASK_TEMPLATE.md)。

## Development quality

提交前运行适用的格式化、静态检查和测试，并检查文档链接。当前 Foundation 只有占位 CI；完整流水线将在路线图最后阶段交付。新增依赖必须说明必要性、许可证和安全影响。

