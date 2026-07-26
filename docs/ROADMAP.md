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

本地自动质量门、Core 冷启动 p95、网络披露、CSP/敏感输出检查与诚实的平台矩阵已由 TASK-0028 固化。TASK-0029 已建立 Core sidecar 安装包、跨平台 CI、签名凭据边界与 tag-gated draft release；正式发布仍需项目所有者配置凭据并完成各平台安装验证。

## Phase 5 — Post-MVP Hosts & Providers

扩展 VS Code、JetBrains 与 Visual Studio Host，并按独立 Task 接入其他托管平台 Provider。无论 Core 运行在本机、WSL、Remote SSH 还是 Dev Container，都必须保持“仓库在哪里，Core 就运行在哪里”。

TASK-0030 已为 Desktop 建立本机、WSL、Remote SSH 与 Dev Container 的结构化 launcher 和远端 repository path 工作流；后续 IDE Host 复用同一进程与安全边界。

TASK-0031 已交付 VS Code Extension Host：Core 在当前 Extension Host（含 Remote SSH/WSL/Container）环境运行，提供 workspace repository、PR original commits、per-commit diff 与 Squash Trace 只读入口。

TASK-0032 已交付 IntelliJ Platform Host 项目、project-scoped Core lifecycle、Tools action 与 Squash Trace/original commit diff 交互；Gradle wrapper 与 CI `buildPlugin` 固化真实平台编译门。

TASK-0033 已交付 VisualStudio.Extensibility 进程外 Host、Solution 环境 Core lifecycle、显式 PR/commit 输入与 Windows CI 构建门。TASK-0034 已在 Core 中通过非交互 `glab api` 交付 GitLab.com/Self-Managed project、MR original commits、per-commit diff 与 Provider-confirmed Squash Trace。

## Phase 6 — AI Assist（最终阶段）

在 Squash Trace MVP、Desktop 交付质量和 Post-MVP Host/Provider 全部完成后，再以独立 Task 引入 AI Assist。候选能力包括根据 staged diff 生成 commit message 草稿，以及根据仓库状态给出拆分 commit、测试、冲突处理等操作建议。

AI 编排和 Git 语义属于 Core；Host 只展示输入范围、建议、可编辑草稿和确认步骤。功能必须显式触发，默认只生成建议，不自动 commit，也不自动执行 reset、rebase、push 等高风险操作。模型可为本地模型或用户自行配置的直连 Provider；不得引入 GitNova 账户或中心代理，发送前必须展示并最小化将离开仓库环境的数据。

TASK-0035 已锁定协议 1.14 的预览/生成类型、index-bound 确认、本地 Ollama 与可选 OpenAI Responses API 直连边界。TASK-0036 已在 Core 交付 staged input builder、敏感/二进制排除、首批 Provider adapter、严格结构化输出和稳定错误。TASK-0037 已交付 Desktop 披露预览、外发确认、可编辑草稿、建议呈现，以及进入既有 commit 两步确认的安全 handoff。TASK-0043 以协议 1.15 增加 Claude、DeepSeek、Qwen 与 Kimi 固定官方 endpoint，并把多 Provider 配置集中在 Settings；AI 仍只在 commit composer 出现。

产品目标见[愿景](VISION.md)，技术选择见[技术栈](TECH_STACK.md)，质量门槛见[非功能需求](NON_FUNCTIONAL.md)。

## Completed Task Baseline

从 TASK-0023 之后锁定的 14 个独立 Review 单元均已完成；以下编号保留为交付记录，具体范围以各 Task 文档为准：

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
15. TASK-0043 — AI Provider 扩展与 Settings 配置

该基线已全部交付。任何新增范围必须先更新 Roadmap，并继续使用独立 Task 作为开发与 Review 单元。

## Maintenance Tasks

- TASK-0038 — 已修复 Desktop Windows Tauri 图标资源、跨平台 Dev Container 路径验证和 IntelliJ IDEA 2026.2/Java 25 CI 工具链一致性；完整平台矩阵已通过，不改变产品能力与架构边界。
