# Roadmap

路线图表达顺序，不承诺具体日期。每一阶段都必须通过独立 Task、ADR（如需要）和 Review 才能进入实现。

## Phase 0 — Foundation（当前）

建立 Monorepo、文档、ADR、协作规范、品牌和占位 CI；不实现业务功能。

## Phase 1 — Core Contract

定义 `gitnova-core` 进程生命周期、JSON-RPC 基础协议、能力协商、错误模型与 SDK 生成方式。协议决策需遵循 [ADR-0004](../adr/ADR-0004-Core-Process.md)。

## Phase 2 — Local Git Foundation

按经批准的 Task 增量实现本地 Git 能力；使用 System Git，保持离线和可撤销。具体候选范围见[功能清单](FEATURE_LIST.md)。

## Phase 3 — Desktop Squash Trace MVP

交付 Tauri 2 Desktop Host 的端到端核心工作流：在 Core 中接入 GitHub Provider（`gh`、REST 或 GraphQL），获取 PR 原始 commits，展示指定 commit 的文件与行级 diff，并关联 PR、原始 commits 与最终 squash commit。该 Squash Trace 主路径是 MVP 验证门槛。Host 不得承载或复制任何 Git/GitHub 业务逻辑。

## Phase 4 — MVP Quality & Delivery

对 Desktop Squash Trace 主路径补齐跨平台测试、性能预算、凭据与网络访问透明性、签名、打包、发布和 CI/CD。GitHub 访问必须由用户明确配置或触发，结果与派生数据默认仅保存在仓库所在环境。

## Phase 5 — Post-MVP Hosts & Providers

扩展 VS Code、JetBrains 与 Visual Studio Host，并按独立 Task 接入其他托管平台 Provider。无论 Core 运行在本机、WSL、Remote SSH 还是 Dev Container，都必须保持“仓库在哪里，Core 就运行在哪里”。

产品目标见[愿景](VISION.md)，技术选择见[技术栈](TECH_STACK.md)，质量门槛见[非功能需求](NON_FUNCTIONAL.md)。
