# ADR-0008: AI Assist 使用本地 Ollama 或用户配置的外部 Provider 直连

- **Status:** Accepted
- **Date:** 2026-07-26
- **Decision owners:** GitNova maintainers

## Context

GitNova 需要根据 staged diff 生成 commit message 草稿和操作建议，同时保持 Local-first、无 GitNova 账户、无中心代理、Core-only 业务逻辑与明确的外部数据披露。模型和云端 API 会演进，因此产品不能把“最新模型”写死为协议默认值。

## Decision

- Provider 包含仓库环境中的本地 Ollama，以及用户选择的 OpenAI、Claude、DeepSeek、Qwen 和 Kimi 直连。Core 是唯一 Provider client；Host 不接触 prompt、diff payload 或凭据。
- Ollama 默认仅允许 `http://127.0.0.1:11434`，自定义 URL 也必须是 loopback HTTP。外部 endpoint 固定为各 Provider 官方 HTTPS API，模型由用户显式填写；协议不接受自定义外部 endpoint。
- API key 仅由 Core 在仓库环境读取 Provider 对应环境变量；不得经 JSON-RPC 传递、持久化或写入日志。OpenAI 使用 Responses API 与 `store: false`，Claude 使用 Messages API，DeepSeek/Qwen/Kimi 使用 OpenAI-compatible Chat Completions；请求不声明工具，输出必须通过 Core 严格校验。
- Core 使用仓库环境的 System `curl` 作为首个 HTTP transport；endpoint/body/header 通过 stdin config 传递，固定参数中不包含 API key 或 diff，响应受硬上限约束。
- `ai/inputPreview` 是完全离线的披露预检。它只读取 staged diff 与最小状态，返回目标 endpoint、文件、字节数、排除/截断原因和绑定 index/provider/model/exclusion 的 `previewId`。
- `ai/generateCommitDraft` 必须由用户显式触发。外部 Provider 还要求确认披露；Core 重新计算预览并拒绝 stale/mismatched `previewId`。
- 默认排除常见 credential 文件，用户可再排除安全的仓库相对路径。二进制和超限内容不发送，任何无法证明范围安全的情况都 fail closed。
- Provider 结果只可映射为有长度限制的 commit message 草稿、固定枚举的操作建议和警告；不接受 shell command、工具调用或 mutation 指令。真正 commit 仍走既有确定性 Core mutation 和单独确认。

## Consequences

本地 Ollama 可使内容不离开仓库环境；外部直连提供多种云模型选择但需要逐次可见确认。没有 GitNova 代理或账户，也不保存 AI 内容。代价是用户必须自行运行 Ollama 或配置所选 Provider 的 key/model，Qwen 固定 endpoint 还要求同区域 key；模型不可用、凭据缺失和预览过期都需要稳定错误处理。

## Alternatives considered

- **GitNova Cloud proxy：** 违反 Local-first 并引入账户、凭据和数据托管责任，拒绝。
- **Host 直接调用模型：** 会复制数据选择、安全规则与凭据处理，拒绝。
- **AI 自动执行 Git 或 shell：** 输出不可确定且扩大权限边界，拒绝。
- **默认发送完整仓库上下文：** 违反最小披露，拒绝。

## Links

[AI Assist Contract](../docs/AI_ASSIST.md) · [Architecture](../docs/ARCHITECTURE.md) · [OpenAI Responses API](https://developers.openai.com/api/docs/guides/migrate-to-responses) · [Claude Messages API](https://platform.claude.com/docs/en/api/messages) · [DeepSeek API](https://api-docs.deepseek.com/) · [Qwen OpenAI compatibility](https://www.alibabacloud.com/help/en/model-studio/compatibility-of-openai-with-dashscope) · [Kimi API](https://platform.kimi.ai/docs/api/chat)
