# Task Management

Task 是 GitNova 的最小交付与 Review 单元。它将产品目标转成有明确边界、可验证且可追踪的变更。

## 生命周期

```text
Draft → Ready → In Progress → Review → Done
                    └──────────────→ Blocked
```

- **Draft：** 补全背景、范围、禁区和验收条件。
- **Ready：** 范围获批、依赖明确，可开始工作。
- **In Progress：** 使用对应分支实现；新增范围拆分 Task。
- **Review：** 交付物和 checklist 完成，等待非作者审查。
- **Done：** PR 合并且验收通过。
- **Blocked：** 记录阻塞原因、Owner 和解除条件。

## 编号与文件

文件名采用 `TASK-<四位编号>-<kebab-title>.md`，例如 `TASK-0001-project-foundation.md`。分支使用[贡献指南](../CONTRIBUTING.md#branch)规定的 type 与任务编号。新 Task 从[模板](templates/TASK_TEMPLATE.md)复制。

## 规则

- 一个 Task 只有一个可判断的目标和 Owner。
- Scope 与 Non-goals 同等重要；禁止在实现中静默扩张。
- Deliverables 必须对应 Review Checklist 和 Done Definition。
- 架构、协议、安全或持久化决策必须新增或引用 ADR。
- 文档、测试和品牌资产是交付物，不是后续清理项。
- Review 发现的新能力进入新 Task；缺陷可在原范围内修复。

工程规范见[贡献指南](../CONTRIBUTING.md)，产品分期见[路线图](../docs/ROADMAP.md)，架构决策见 [ADR 索引](../adr/README.md)。

