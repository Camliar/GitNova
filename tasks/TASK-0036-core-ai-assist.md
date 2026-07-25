# TASK-0036: Core AI Assist

- **Status:** In Progress
- **Priority:** P2
- **Owner:** Codex
- **Branch:** `feature/036-core-ai-assist`
- **Dependencies:** TASK-0035 (`8341c78`)

## Goal

在 `gitnova-core` 实现已批准的 AI 输入预览、Ollama/OpenAI commit draft 和结构化操作建议，同时保持显式触发、最小披露与零 mutation 权限。

## Scope

- 实现 `ai/inputPreview`：从当前 index 构造 staged textual patch、披露清单、上限状态与稳定 `previewId`，不访问网络。
- 默认与用户指定的敏感路径排除；拒绝不安全路径，排除二进制，限制文件数、每文件和总 prompt 大小。
- 实现 `ai/generateCommitDraft`：请求前重建预览并拒绝 stale/mismatched 输入；OpenAI 必须确认外部披露。
- Core-owned Ollama loopback adapter 和固定官方 OpenAI Responses API adapter；模型由参数指定，OpenAI key 只从环境读取。
- Provider 无工具调用；OpenAI `store: false`；严格验证模型 JSON、字段长度和 suggestion enum。
- 开启 `aiAssist` capability，新增稳定错误、unit/contract tests 和运行说明。

## Non-goals

- Desktop 或 IDE Host UI、设置持久化、模型下载或账户管理。
- 自动 commit、自动运行测试、shell/tool calling 或任意 Git mutation。
- GitNova proxy、遥测或保存 prompt/diff/response/API key。

## Deliverables

- [ ] deterministic staged input builder and disclosure preview
- [ ] Ollama/OpenAI Provider adapters and strict output validation
- [ ] JSON-RPC dispatch, capability and stable errors
- [ ] unit/contract tests and synchronized documentation

## Review Checklist

- [ ] 预览不联网，生成显式触发，外部确认与当前 index/provider/model/exclusions 绑定。
- [ ] endpoint、路径、payload 与响应均有 fail-closed 校验和硬上限。
- [ ] 凭据不会进入参数、日志、错误或存储；测试不访问真实 Provider。
- [ ] AI 结果无可执行命令或 mutation 字段。
- [ ] Rust tests、contract、fmt 与 clippy 通过。

## Done Definition

- [ ] 自主 Review 无阻塞项，状态 Done，提交推送并快进合并 main。

