# TASK-0012: AI Assist Roadmap

- **Status:** Done
- **Priority:** P3
- **Owner:** Codex
- **Branch:** `docs/012-ai-assist-roadmap`
- **Estimate:** < 1 day
- **Dependencies:** Product decision after TASK-0011

## Goal

把已批准的 AI Assist 方向和“最后实现”顺序写入产品事实来源，确保它不抢占 Squash Trace MVP、交付质量或 Post-MVP Host/Provider 工作。

## Scope

- Roadmap 在现有所有阶段之后新增最终 AI Assist 阶段。
- Feature List 记录 commit message 生成与操作建议候选能力。
- PRD 与 Architecture 锁定 Core-owned、Local-first、显式触发、最小披露、预览确认和危险操作禁区。
- AI 默认只生成建议或草稿；commit 及其他 mutation 必须由用户确认并走 Core 的确定性 Git 能力。

## Non-goals

- 模型、Provider、SDK、prompt、存储、UI 或 Git mutation 实现。
- 调整 Squash Trace MVP、Desktop、质量、IDE Host 或 Provider 的既有优先级。
- 选择云模型或本地模型。

## Deliverables

- [x] Roadmap 最终 AI 阶段
- [x] Feature List/PRD AI Assist 范围
- [x] Architecture 安全与 Host/Core 边界

## Review Checklist

- [x] AI 实现明确排在现有所有阶段之后。
- [x] 不引入中心服务器或 GitNova 账户。
- [x] 用户可见输入范围并在 mutation 前确认。
- [x] 本 Task 无业务实现或依赖变更。

## Done Definition

- [x] 文档一致且链接有效。
- [x] 自主 Review 无阻塞项。
- [x] 提交、推送并快进合并至 `main`。

## References

[Roadmap](../docs/ROADMAP.md) · [Feature List](../docs/FEATURE_LIST.md) · [Product Requirements](../docs/PRODUCT_REQUIREMENTS.md) · [Architecture](../docs/ARCHITECTURE.md)
