# TASK-0033: Visual Studio Host

- **Status:** Done
- **Priority:** P1
- **Owner:** Codex
- **Branch:** `feature/033-visual-studio-host`
- **Dependencies:** TASK-0032 (`7d0317c`)

## Goal

交付基于 VisualStudio.Extensibility 进程外模型的 Visual Studio Host，在 Solution 所在环境启动 Core，并展示 PR original commits、per-commit diff 与 Squash Trace。

## Scope

- .NET 8、VisualStudio.Extensibility SDK、Extension/Command 注册及 Windows VSIX 构建入口。
- Host-owned Core 进程生命周期、JSON-RPC framing、handshake、timeout 与 shutdown。
- Core executable 仅允许绝对 override 或 PATH 固定名称；Solution/repository path 原样发送给 Core。
- 用户显式输入 PR，选择 original commit，并展示 Core 返回的 Squash Trace 与 remote diff。
- 跨平台 transport tests/static checks，以及 Windows CI 的完整扩展构建门槛。

## Non-goals

- 扩展内 Git/`gh`/HTTP、Squash 关系推断或仓库同步。
- 自动安装 Core、复制 Desktop UI、Marketplace 发布。
- GitLab Provider 或 AI Assist。

## Deliverables

- [x] Visual Studio extension project and command registration
- [x] Core lifecycle and Squash Trace interaction
- [x] transport tests, Windows CI build gate and documentation

## Review Checklist

- [x] Visual Studio Host 使用进程外扩展模型，不在 IDE 进程内承载业务逻辑。
- [x] 进程使用参数 API，不经 shell；stderr 不进入协议或 UI。
- [x] Provider 网络动作由用户显式触发并通过 Core。
- [x] transport tests、static checks 与适用平台 build 验证通过；macOS 交叉编译 0 warning/0 error，Windows CI 保留平台门槛。

## Done Definition

- [x] 自主 Review 无阻塞项，状态 Done，提交推送并快进合并 main。
