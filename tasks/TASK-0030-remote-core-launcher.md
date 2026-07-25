# TASK-0030: WSL, Remote SSH and Dev Container Core Launcher

- **Status:** Done
- **Priority:** P1
- **Owner:** Codex
- **Branch:** `feature/030-remote-core-launcher`
- **Dependencies:** TASK-0029 (`e121af8`)

## Goal

让 Desktop 能显式选择本机、WSL、Remote SSH 或 Dev Container Core 运行环境，确保 Core 与仓库处于同一环境并继续使用 stdio JSON-RPC。

## Scope

- Tauri Host 提供结构化 launcher target 配置与当前环境状态，不接受任意命令行或 shell 字符串。
- 本机运行 bundled Core；WSL 使用 `wsl.exe --exec`；SSH 使用 batch mode `ssh`；Dev Container 使用 `devcontainer exec`。
- 连接前可重新配置；Core 运行后拒绝切换，避免仓库 session 跨环境漂移。
- 严格验证 distribution、SSH destination、remote Core path 与 workspace folder，并使用参数数组直接 spawn。
- Desktop 显示环境选择、环境特定 repository path 输入与明确的工具/远端安装前置条件。
- 单元测试覆盖命令投影、注入拒绝、锁定和既有 transport lifecycle。

## Non-goals

- 自动安装远端 Core、同步仓库、复制凭据、端口转发或远端 daemon。
- 远程文件选择器、SSH config 管理、容器发现或业务协议变化。
- IDE Host 实现。

## Deliverables

- [x] structured launcher model, validation and process projection
- [x] Desktop environment selection and remote repository path workflow
- [x] launcher security/lifecycle tests and documentation

## Review Checklist

- [x] 不经 shell，不接受任意 executable/arguments。
- [x] 远端连接为 batch/non-interactive，失败不会泄漏 stderr。
- [x] Core 运行环境和 repository path 由用户明确选择。
- [x] frontend/Rust/Clippy/rustfmt/Tauri checks 通过。

## Done Definition

- [x] 自主 Review 无阻塞项，状态 Done，提交推送并快进合并 main。
