# AI Assist Contract

AI Assist 是 Core-owned、显式触发的草稿能力。它不会提交代码、修改 index、运行 shell、调用任意工具或在后台发送仓库数据。首批协议版本是 1.14；TASK-0035 只发布类型与安全契约，Core 在 capability `aiAssist: false` 时必须拒绝业务方法。

## Provider boundary

| Provider | 目标 | 凭据 | 数据边界 |
| --- | --- | --- | --- |
| Ollama | 默认 `http://127.0.0.1:11434`；只允许 loopback HTTP | 无 GitNova 凭据 | 保留在仓库环境，但仍需显式触发 |
| OpenAI | 固定 `https://api.openai.com/v1/responses` | Core 从 `OPENAI_API_KEY` 读取 | 离开仓库环境，生成前必须逐次确认 |

两种 Provider 的 `model` 都由用户显式配置；协议不定义会过时的默认模型。OpenAI adapter 必须使用 Responses API、`store: false`、无工具和严格 JSON Schema 输出。API key 不得出现在 Host、JSON-RPC、SQLite、错误或日志中。

## Input preview

`ai/inputPreview` 接受 `provider` 和可选 `excludedPaths`，不访问网络。Core 从当前 index 构造最小输入：staged textual patch，以及生成安全建议所需的结构化 staged 状态。工作区未暂存内容、历史、完整文件、remote 内容和 PR/MR 数据均不进入 prompt。

响应 `AiInputPreview` 明确返回：

- 与 index 指纹、Provider、model 和规范化排除路径绑定的 opaque `previewId`；
- `local`/`external` 目标、实际 endpoint 与是否需要外部披露确认；
- 每个 staged path 的 additions/deletions、将发送的 patch bytes、`included`/`excluded`/`binary`/`truncated` 状态和原因；
- staged diff/prompt 总字节数，以及是否发生截断。

常见敏感文件（例如 `.env`、private key、registry credential 文件）默认排除；用户排除项是规范化、无 `..` 的仓库相对路径。二进制永不发送。实现必须设置每文件、总 payload 和文件数上限；无法解析、超限且无法安全截断、路径不安全或 staged 内容为空时 fail closed。

## Generate contract

`ai/generateCommitDraft` 重复提交 `previewId`、Provider、排除路径和 `externalDisclosureConfirmed`。Core 在网络请求前重建预览：任何 index、Provider、model、endpoint 或排除范围变化都返回 stale-preview 错误，要求 Host 重新展示预览。OpenAI 请求若未确认外部披露必须拒绝；确认不能跨预览复用。

结果 `AiCommitDraft` 只包含：

- 可编辑且受长度限制的 `commitMessage`；
- 固定枚举 `splitCommit`、`runTests`、`resolveConflicts`、`reviewSensitiveData`、`reviewLargeChange` 的结构化建议；
- 非敏感警告、Provider kind、model 和原 `previewId`。

Core 必须验证模型 JSON 和字段长度，拒绝未知 suggestion kind，不回显原始响应。建议的 title/detail 仅是文本，不能包含可自动执行字段。生成完成不会调用 `repository/commit`；Host 必须让用户编辑草稿，并通过既有 mutation 二次确认。

## Stable errors

实现阶段保留以下稳定错误族：`ai.nothing_staged`、`ai.invalid_provider`、`ai.preview_stale`、`ai.external_confirmation_required`、`ai.credential_missing`、`ai.provider_unavailable`、`ai.request_failed`、`ai.response_invalid`、`ai.input_limit_exceeded`。错误不得携带 diff、prompt、response 或凭据。

安全决策见 [ADR-0008](../adr/ADR-0008-AI-Assist-Providers.md)，通用 framing 与错误模型见[协议](PROTOCOL.md)。

