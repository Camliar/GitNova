# TASK-0032: JetBrains Host

- **Status:** In Progress
- **Priority:** P1
- **Owner:** Codex
- **Branch:** `feature/032-jetbrains-host`
- **Dependencies:** TASK-0031 (`a074f34`)

## Goal

交付 IntelliJ Platform 插件 Host，在项目后端环境运行 Core，并以 IDE Action 展示 PR original commits、per-commit diff 与 Squash Trace。

## Scope

- 使用 IntelliJ Platform Gradle Plugin 2.x、Java 17、plugin.xml 与 Tools menu action。
- project-scoped Core service 管理进程、JSON-RPC framing、handshake、timeout 和 disposal。
- Core executable 仅允许绝对 override 或环境 PATH 固定名称；项目 base path 原样发送给 Core。
- 用户显式输入 PR，选择 original commit，展示 Core 返回的 Squash Trace 与 remote diff。
- 纯 JDK transport/framing tests 与 plugin project build/check 配置。

## Non-goals

- 插件内 Git/`gh`/HTTP、Squash 关系推断或仓库同步。
- 自动安装远端 Core、完整 Desktop UI 复制、Marketplace 发布。
- Visual Studio Host。

## Deliverables

- [ ] IntelliJ plugin project and action registration
- [ ] project Core lifecycle and Squash Trace interaction
- [ ] transport tests, Gradle build and documentation

## Review Checklist

- [ ] Project service 在 IDE backend 环境运行，关闭项目时清理 Core。
- [ ] 进程使用参数 API，不经 shell；stderr 不进入 UI。
- [ ] Provider 网络动作由用户显式触发并通过 Core。
- [ ] transport tests、static checks 与可用范围验证通过。

## Done Definition

- [ ] 自主 Review 无阻塞项，状态 Done，提交推送并快进合并 main。
