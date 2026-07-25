# TASK-0035: AI Assist Contract

- **Status:** In Progress
- **Priority:** P2
- **Owner:** Codex
- **Branch:** `feature/035-ai-assist-contract`
- **Dependencies:** TASK-0034 (`6d0cb14`)

## Goal

锁定 AI Assist 的 Core 协议、最小披露隐私模型和首批 Provider 边界，使后续实现可以在不引入 GitNova 账户或中心服务的前提下安全生成 commit message 草稿与操作建议。

## Scope

- 定义 `ai/inputPreview` 与 `ai/generateCommitDraft` JSON-RPC 请求/响应模型；本 Task 只建立契约，不连接模型。
- 预览绑定当前 index 指纹、Provider、模型和排除路径；生成前必须再次匹配，避免预览后 staged 内容变化。
- 输入仅限 staged diff 与最小仓库状态；二进制、敏感路径、超限内容默认排除并明确披露。
- 首批 Provider 为本地 Ollama 和可选的 OpenAI Responses API 直连；模型由用户配置，不设置随时间变化的默认模型。
- OpenAI 凭据仅由 Core 从仓库环境的 `OPENAI_API_KEY` 读取，固定官方 endpoint、`store: false`，不得进入 Host、协议、日志或 SQLite。
- AI 结果只包含可编辑 commit message 草稿、结构化操作建议和警告；不得直接执行 Git、shell 或任意工具。
- 新增 ADR、隐私/安全说明、协议 Schema/Rust/TypeScript 类型与契约测试。

## Non-goals

- 调用 Ollama/OpenAI、prompt 实现、Provider HTTP client 或凭据设置 UI。
- Desktop/IDE Host 交互或自动提交。
- AI 驱动的 shell、Git mutation、远端写操作、工具调用或后台自动触发。
- 保存 prompt、diff、模型响应或 API key。

## Deliverables

- [ ] AI Assist protocol types and version update
- [ ] disclosure, confirmation and stale-preview contract
- [ ] Ollama/OpenAI Provider decision and ADR
- [ ] generated types, contract tests and synchronized product docs

## Review Checklist

- [ ] Host 不能读取或传递 Provider 凭据，Core 是唯一 AI 业务层。
- [ ] 外部披露在生成前可见且需要显式确认；本地 Provider 也只能显式触发。
- [ ] 排除、截断、二进制处理和输入上限是 fail-closed 且可解释的。
- [ ] AI 输出不具备 mutation 权限，不返回可直接执行的任意命令。
- [ ] 协议生成物、Rust tests、fmt 与 clippy 通过。

## Done Definition

- [ ] 自主 Review 无阻塞项，状态 Done，提交推送并快进合并 main。

