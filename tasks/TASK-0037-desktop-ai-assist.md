# TASK-0037: Desktop AI Assist

- **Status:** In Progress
- **Priority:** P2
- **Owner:** Codex
- **Branch:** `feature/037-desktop-ai-assist`
- **Dependencies:** TASK-0036 (`49c375d`)

## Goal

在 Desktop Host 交付可解释、安全确认的 AI commit draft 工作流，并复用既有 Core commit mutation 的独立二次确认。

## Scope

- 新增 Desktop AI Assist panel，仅在已打开非 bare 仓库且 Core capability 可用时启用。
- 用户选择 Ollama/OpenAI、显式填写 model，可选 Ollama loopback URL 与仓库相对排除路径。
- “Preview input” 只调用离线 `ai/inputPreview`，展示 destination/endpoint、文件状态、patch/prompt bytes、截断和敏感排除原因。
- OpenAI 必须勾选当前预览专属的外部披露确认后才可生成；任何配置变化立即使确认和 preview 失效。
- “Generate draft” 显式调用 `ai/generateCommitDraft`，展示可编辑 commit message、结构化建议和警告。
- “Use in commit” 只把文本填入现有 Commit panel；真正 commit 仍需要现有 Preview/Confirm 两步流程。
- 增加 Core bridge method allowlist、TypeScript client、component tests、可访问性与文档。

## Non-goals

- 保存 Provider/model/API key，或在 Desktop 读取环境凭据。
- 自动 preview/generate/commit、AI shell/tool calling 或建议的一键执行。
- VS Code/JetBrains/Visual Studio AI UI。

## Deliverables

- [ ] Desktop AI Assist input preview and disclosure confirmation
- [ ] editable commit draft, suggestions and warnings
- [ ] explicit handoff into existing two-step commit workflow
- [ ] bridge allowlist, tests, build and synchronized docs

## Review Checklist

- [ ] 配置变化清除 preview、确认和旧 draft；stale preview 可安全重试。
- [ ] OpenAI 未确认不能生成，Ollama 仍不会后台触发。
- [ ] API key、prompt 和 raw diff 不进入 Host；UI 只显示 Core disclosure/result。
- [ ] AI 不能直接调用 mutation，commit 保持独立用户确认。
- [ ] Desktop typecheck、tests、production build 与 Rust transport tests 通过。

## Done Definition

- [ ] 自主 Review 无阻塞项，状态 Done，提交推送并快进合并 main。

