# TASK-0031: VS Code Host

- **Status:** In Progress
- **Priority:** P1
- **Owner:** Codex
- **Branch:** `feature/031-vscode-host`
- **Dependencies:** TASK-0030 (`0b096e5`)

## Goal

交付可构建、可测试的 VS Code Extension Host，在 Extension Host 所在环境启动 Core，并提供仓库与 Squash Trace 主路径入口。

## Scope

- VS Code extension manifest、activation、commands、status bar 与受控 Core stdio client。
- Core 从扩展 sidecar、显式绝对开发路径或环境 PATH 解析；Remote Extension Host 不跨环境回传仓库。
- 打开当前 workspace repository，显式输入 PR 编号，展示 PR original commits 与 Squash Trace 关系摘要。
- 选择 original commit 后请求 Core-owned remote diff，并在只读 webview 中安全呈现文件统计和结构化 patch。
- transport framing、超时、握手、关闭、HTML escaping 与配置边界测试。

## Non-goals

- 在扩展中执行 Git/`gh`、解析 squash 关系或调用网络。
- 完整复制 Desktop 所有面板、自动请求 PR、远端安装器或 Marketplace 发布。
- JetBrains/Visual Studio Host。

## Deliverables

- [ ] VS Code extension package, lifecycle and commands
- [ ] PR/original-commit/diff/Squash Trace presentation slice
- [ ] tests, build scripts and host documentation

## Review Checklist

- [ ] Extension Host 只启动/传输/呈现，业务请求全部进入 Core。
- [ ] workspace path 不跨 Remote Extension Host 边界。
- [ ] webview 内容转义，CSP 禁止脚本和远程资源。
- [ ] workspace checks、extension tests/build 通过。

## Done Definition

- [ ] 自主 Review 无阻塞项，状态 Done，提交推送并快进合并 main。
