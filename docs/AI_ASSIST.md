# AI Assist Contract

AI Assist 是 Core-owned、显式触发的草稿能力。它不会提交代码、修改 index、运行 shell、调用任意工具或在后台发送仓库数据。当前协议 1.17 的 Core 通过 capability `aiAssist: true` 提供预览与生成方法。

## Provider boundary

| Provider | 目标 | 凭据 | 数据边界 |
| --- | --- | --- | --- |
| Ollama | 默认 `http://127.0.0.1:11434`；只允许 loopback HTTP | 无 GitNova 凭据 | 保留在仓库环境，但仍需显式触发 |
| OpenAI | 固定 `https://api.openai.com/v1/responses` | Core 从 `OPENAI_API_KEY` 读取 | 离开仓库环境，生成前必须逐次确认 |
| Claude (Anthropic) | 固定 `https://api.anthropic.com/v1/messages` | Core 从 `ANTHROPIC_API_KEY` 读取 | 离开仓库环境，生成前必须逐次确认 |
| DeepSeek | 固定 `https://api.deepseek.com/chat/completions` | Core 从 `DEEPSEEK_API_KEY` 读取 | 离开仓库环境，生成前必须逐次确认 |
| Qwen (Alibaba Cloud) | 固定北京区 `https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions` | Core 从 `DASHSCOPE_API_KEY` 读取 | 离开仓库环境，生成前必须逐次确认；key 必须属于该区域 |
| Kimi (Moonshot AI) | 固定 `https://api.moonshot.ai/v1/chat/completions` | Core 从 `MOONSHOT_API_KEY` 读取 | 离开仓库环境，生成前必须逐次确认 |

所有 Provider 的 `model` 都由用户显式配置；协议不定义会过时的默认模型。OpenAI adapter 使用 Responses API、`store: false`、无工具和严格 JSON Schema 输出；Claude 使用 Messages API；DeepSeek、Qwen 与 Kimi 使用 OpenAI-compatible Chat Completions。API key 不得出现在 Host、JSON-RPC、SQLite、错误或日志中。

所有 adapter 均由 Core 通过仓库环境的 System `curl` 发起 POST；`curl` 参数只包含固定控制项，endpoint、JSON body 和 authorization header 通过 stdin config 传入，避免凭据或 diff 出现在进程参数。Core 限制连接/总超时与最大 response，并丢弃 stderr/非 2xx body。除 Ollama loopback URL 外，协议不接受自定义 endpoint。

## Input preview

`ai/inputPreview` 接受 `provider` 和可选 `excludedPaths`，不访问网络。Core 从当前 index 构造最小输入：staged textual patch，以及生成安全建议所需的结构化 staged 状态。工作区未暂存内容、历史、完整文件、remote 内容和 PR/MR 数据均不进入 prompt。

响应 `AiInputPreview` 明确返回：

- 与 index 指纹、Provider、model 和规范化排除路径绑定的 opaque `previewId`；
- `local`/`external` 目标、实际 endpoint 与是否需要外部披露确认；
- 每个 staged path 的 additions/deletions、将发送的 patch bytes、`included`/`excluded`/`binary`/`truncated` 状态和原因；
- staged diff/prompt 总字节数，以及是否发生截断。

常见敏感文件（例如 `.env`、private key、registry credential 文件）默认排除；用户排除项是规范化、无 `..` 的仓库相对路径。二进制永不发送。实现必须设置每文件、总 payload 和文件数上限；无法解析、超限且无法安全截断、路径不安全或 staged 内容为空时 fail closed。

首个实现最多接受 200 个 staged paths、每文件 64 KiB patch、总 prompt 256 KiB；Provider response 最多 128 KiB。超长单文件 patch 可在 UTF-8 边界截断并在预览标记，其余无法安全容纳的总输入直接拒绝。

## Generate contract

`ai/generateCommitDraft` 重复提交 `previewId`、Provider、排除路径和 `externalDisclosureConfirmed`。Core 在网络请求前重建预览：任何 index、Provider、model、endpoint 或排除范围变化都返回 stale-preview 错误，要求 Host 重新展示预览。任一外部 Provider 请求若未确认外部披露必须拒绝；确认不能跨预览复用。

结果 `AiCommitDraft` 只包含：

- 可编辑且受长度限制的 `commitMessage`；
- 固定枚举 `splitCommit`、`runTests`、`resolveConflicts`、`reviewSensitiveData`、`reviewLargeChange` 的结构化建议；
- 非敏感警告、Provider kind、model 和原 `previewId`。

Core 必须验证模型 JSON 和字段长度，拒绝未知 suggestion kind，不回显原始响应。建议的 title/detail 仅是文本，不能包含可自动执行字段。生成完成不会调用 `repository/commit`；Host 必须让用户编辑草稿，并通过既有 mutation 二次确认。

## Desktop workflow

Desktop 把 Provider、model、loopback URL、凭据环境变量说明和排除路径集中在 Settings；Local Changes 的 commit composer 只提供默认折叠的“Generate message with AI”入口，history、PR 和其他工作区不展示 AI 功能。配置任一变化都会立即丢弃旧 preview、外部确认和 draft。Host 先显示 Core 返回的 endpoint、字节数、index binding 与逐文件状态；任一外部 Provider 只有在用户勾选当前 preview 的披露确认后才可生成，Ollama 也只能由按钮显式触发。

生成结果在 Host 中可编辑；“Use in commit”仅把文本送入现有 Commit 表单并聚焦输入框。它不会调用 mutation。用户仍需点击“Review commit”，查看当前 staged index 的确认描述，再点击“Confirm action”才会调用 `repository/commit`。

## Stable errors

实现提供以下稳定错误族：`ai.nothing_staged`、`ai.invalid_provider`、`ai.preview_stale`、`ai.external_confirmation_required`、`ai.credential_missing`、`ai.provider_unavailable`、`ai.request_failed`、`ai.response_invalid`、`ai.input_limit_exceeded`。错误不得携带 diff、prompt、response 或凭据。

安全决策见 [ADR-0008](../adr/ADR-0008-AI-Assist-Providers.md)，通用 framing 与错误模型见[协议](PROTOCOL.md)。
