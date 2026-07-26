# TASK-0043: AI Provider Expansion

- **Status:** Done
- **Priority:** P1
- **Owner:** Codex
- **Branch:** `codex/task-043-ai-provider-expansion`
- **Dependencies:** completed TASK-0042 baseline (`88b0202`)

## Goal

在不改变 Local-first、Core-only 业务边界的前提下，让 Commit 工作流可选择 Ollama、OpenAI、Claude、DeepSeek、Qwen 和 Kimi，并把所有 Provider 配置集中在 Settings。

## Scope

- 协议 1.15 新增 Anthropic/Claude、DeepSeek、Qwen 和 Kimi Provider kind/config；协议只传 Provider 与 model，不传凭据。
- Core 为各外部 Provider 使用固定官方 HTTPS endpoint，并只从仓库所在环境读取对应环境变量。
- OpenAI 继续使用 Responses API；Claude 使用 Messages API；DeepSeek、Qwen 与 Kimi 使用各自 OpenAI-compatible Chat Completions API。
- 所有外部 Provider 复用现有 staged-diff 预览、逐次外部披露确认、响应大小限制和严格输出验证。
- Desktop Settings 展示 Provider、model、Ollama loopback URL、凭据环境变量说明与排除路径；Commit composer 是唯一 AI 入口。
- 更新协议生成物、契约/单元/UI 测试、AI 文档、产品文档与 ADR。

## Non-goals

- 在 Host、JSON-RPC、SQLite 或日志中接收、保存或显示 API key。
- 支持自定义外部 Provider endpoint、代理、兼容服务或 Provider SDK。
- 定义容易过时的默认模型、查询模型列表、估算费用或展示余额。
- AI 自动执行 commit、Git 命令、shell、tool call 或后台请求。
- 在 history、PR 或其他非 commit 场景展示 AI 功能。

## Deliverables

- [x] protocol 1.15 Provider contract and generated TypeScript SDK
- [x] Core adapters for Claude, DeepSeek, Qwen and Kimi
- [x] Settings provider selection and credential-boundary guidance
- [x] commit-only AI workflow support for every Provider
- [x] contract, Core and Desktop test coverage
- [x] synchronized AI, protocol, product and architecture documentation

## Review Checklist

- [x] Host never receives credentials and every network call remains Core-owned.
- [x] External endpoints are fixed HTTPS destinations and require confirmation for the current preview.
- [x] Provider responses remain untrusted and pass existing strict validation before reaching Host.
- [x] Provider/model/settings changes invalidate preview, confirmation and draft.
- [x] AI remains visible only inside the commit composer.
- [x] Protocol, Rust, Desktop, quality and delivery checks pass.

## Done Definition

- [x] Deliverables and review checklist are complete.
- [x] Autonomous review has no blocking findings.
- [x] Status is Done; branch is committed, pushed, fast-forwarded into `main`, and remote `main` is verified.

## Verification

- `cargo fmt --all -- --check`: pass
- `cargo test --workspace`: 44 Core unit + 36 Core contract + 5 Desktop transport + 4 protocol tests pass
- Desktop `tsc --noEmit`: pass
- Desktop `vitest run`: 71 tests pass
- Desktop `vite build`: pass
- protocol generation, quality and delivery scripts: pass
- VS Code: 4 tests and JavaScript syntax checks pass
- JetBrains and Visual Studio Host checks: pass
- macOS x86_64 `.app`: strict code-sign verification pass; Desktop and Core sidecar are x86_64 Mach-O
- DMG: `hdiutil verify` pass; SHA-256 `d1b4864141ec1000ea381551be1f8a580914536ed645f5a23d2af19ca2206aa3`
