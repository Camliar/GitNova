# TASK-0029: Desktop Packaging, Signing and CI/CD Release

- **Status:** In Progress
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/029-desktop-packaging-release`
- **Dependencies:** TASK-0028 (`265f9a1`)

## Goal

建立可复现的 Desktop 跨平台质量检查、安装包构建与受控发布流程，并明确签名凭据边界。

## Scope

- 启用 Tauri bundle，配置 macOS、Windows、Linux 安装包元数据与现有品牌图标。
- 增加 Windows、macOS、Linux CI matrix，运行前端、Rust、Clippy、rustfmt 与 Tauri build。
- 增加仅由版本 tag 触发的 release workflow，构建并上传平台产物。
- 签名、公证凭据只通过 GitHub Actions secrets 注入；fork/普通 push 不接触发布凭据。
- 文档化本地打包、版本/tag 规则、平台产物、签名 secrets 与故障回退。

## Non-goals

- 自动更新服务、GitNova 账户、中心发布后端。
- 实际配置用户的 Apple/Microsoft/Linux 私钥或创建正式 GitHub Release。
- 远程 Core launcher、IDE Host、Provider 或业务功能。

## Deliverables

- [ ] active Tauri bundle configuration and package metadata
- [ ] cross-platform CI and tag-gated release workflows
- [ ] release/signing runbook and validation

## Review Checklist

- [ ] 普通 CI 无发布权限且不读取签名 secrets。
- [ ] release 只接受 `v*` tag，并使用最小 `contents: write` 权限。
- [ ] 三平台命令、产物和签名状态诚实记录。
- [ ] 本地质量门与 macOS bundle 构建通过。

## Done Definition

- [ ] 自主 Review 无阻塞项，状态 Done，提交推送并快进合并 main。
