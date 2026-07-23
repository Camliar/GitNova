# TASK-0001: Project Foundation

- **Status:** Review
- **Priority:** P0
- **Branch:** `chore/001-project-foundation`
- **Estimate:** 1–2 days
- **Dependencies:** None

## Goal

建立 GitNova 后续 Task 共用的 Monorepo、产品与架构文档、工程规范、ADR、Task 流程和占位品牌资产，不开发业务功能。

## Scope

- 初始化项目结构、Cargo/pnpm workspace 与仓库基础配置。
- 编写 README、基础文档、ADR、贡献和 Task 规范。
- 创建占位 Logo、品牌令牌、PR/Issue 模板与非执行 CI 占位。

## Non-goals

- Repository、Git Status、Commit、Diff、Branch、Graph。
- GitHub API、PR、Squash Trace。
- 任何其他 Git/GitHub 业务实现或完整 CI/CD。

## Deliverables

- [x] Monorepo 与目录结构
- [x] 基础文档与互链
- [x] ADR-0001 至 ADR-0004
- [x] README、CONTRIBUTING、LICENSE 和仓库配置
- [x] Task 规范与协作模板
- [x] SVG/PNG 占位 Logo、图标与品牌令牌
- [x] 非执行 CI 占位文件

## Review Checklist

- [x] 文档链接可解析，目录描述与实际结构一致
- [x] 架构一致定义 Core 为本地独立进程
- [x] 明确 Local First、无中心 Server、Host/Core 分层
- [x] Host 不承载业务逻辑，Core 使用 JSON-RPC/stdio 与 System Git
- [x] MVP 明确包含 GitHub Provider、PR original commits、per-commit diff 与 Squash Trace
- [x] 不包含业务实现
- [x] README 可独立介绍项目
- [x] Logo 与品牌文档已建立

## Done Definition

- [x] 项目骨架、文档、ADR、品牌和 Monorepo 已完成
- [x] 无业务实现
- [x] 状态进入 Review
- [ ] Review 通过后更新为 Done

## References

[Project](../docs/PROJECT.md) · [Architecture](../docs/ARCHITECTURE.md) · [Branding](../docs/BRANDING.md) · [ADR Index](../adr/README.md) · [Contributing](../CONTRIBUTING.md)
