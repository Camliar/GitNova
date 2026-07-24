# Roadmap

路线图表达顺序，不承诺具体日期。每一阶段都必须通过独立 Task、ADR（如需要）和 Review 才能进入实现。

## Phase 0 — Foundation

建立 Monorepo、文档、ADR、协作规范、品牌和占位 CI；不实现业务功能。

## Phase 1 — Core Contract

定义 `gitnova-core` 进程生命周期、JSON-RPC 基础协议、能力协商、错误模型与 SDK 生成方式。协议决策需遵循 [ADR-0004](../adr/ADR-0004-Core-Process.md)。

## Phase 2 — Local Git Foundation

按经批准的 Task 增量实现本地 Git 能力；使用 System Git，保持离线和可撤销。具体候选范围见[功能清单](FEATURE_LIST.md)。

## Phase 3 — Desktop Squash Trace MVP（当前）

交付 Tauri 2 Desktop Host 的端到端核心工作流：在 Core 中接入 GitHub Provider（`gh`、REST 或 GraphQL），获取 PR 原始 commits，展示指定 commit 的文件与行级 diff，并关联 PR、原始 commits 与最终 squash commit。该 Squash Trace 主路径是 MVP 验证门槛。Host 不得承载或复制任何 Git/GitHub 业务逻辑。

Desktop Host 基座、独立 Core transport、仓库/PR 导航、commit graph、original commit diff、Squash Trace 关系呈现，以及 staged commit/local branch mutation workflow 已经建立。后续 MVP Task 进入端到端质量与交付。

## Phase 4 — MVP Quality & Delivery

对 Desktop Squash Trace 主路径补齐跨平台测试、性能预算、凭据与网络访问透明性、签名、打包、发布和 CI/CD。GitHub 访问必须由用户明确配置或触发，结果与派生数据默认仅保存在仓库所在环境。

## Phase 5 — Post-MVP Hosts & Providers

扩展 VS Code、JetBrains 与 Visual Studio Host，并按独立 Task 接入其他托管平台 Provider。无论 Core 运行在本机、WSL、Remote SSH 还是 Dev Container，都必须保持“仓库在哪里，Core 就运行在哪里”。

## Phase 6 — AI Assist（最终阶段）

在 Squash Trace MVP、Desktop 交付质量和 Post-MVP Host/Provider 全部完成后，再以独立 Task 引入 AI Assist。候选能力包括根据 staged diff 生成 commit message 草稿，以及根据仓库状态给出拆分 commit、测试、冲突处理等操作建议。

AI 编排和 Git 语义属于 Core；Host 只展示输入范围、建议、可编辑草稿和确认步骤。功能必须显式触发，默认只生成建议，不自动 commit，也不自动执行 reset、rebase、push 等高风险操作。模型可为本地模型或用户自行配置的直连 Provider；不得引入 GitNova 账户或中心代理，发送前必须展示并最小化将离开仓库环境的数据。

产品目标见[愿景](VISION.md)，技术选择见[技术栈](TECH_STACK.md)，质量门槛见[非功能需求](NON_FUNCTIONAL.md)。

## Remaining Task Baseline

截至 TASK-0023 完成，后续路线按以下 14 个独立 Review 单元推进；编号锁定顺序，具体范围仍以各 Task 文档为准：

1. TASK-0024 — Desktop Squash Trace 关联展示
2. TASK-0025 — Desktop commit graph 可视化
3. TASK-0026 — Core commit 与 branch mutation
4. TASK-0027 — Desktop commit 与 branch workflow
5. TASK-0028 — MVP 端到端质量、性能与网络透明性
6. TASK-0029 — Desktop 打包、签名与 CI/CD 发布
7. TASK-0030 — WSL、Remote SSH 与 Dev Container Core launcher
8. TASK-0031 — VS Code Host
9. TASK-0032 — JetBrains Host
10. TASK-0033 — Visual Studio Host
11. TASK-0034 — GitLab Provider
12. TASK-0035 — AI Assist 协议、隐私与 Provider 决策
13. TASK-0036 — Core AI commit draft 与操作建议
14. TASK-0037 — Desktop AI Assist 交互与安全确认

该基线用于回答剩余工作量并防止 AI Assist 提前；新增范围必须先更新 Roadmap，且不得把多个 Task 合并成一次实现。
