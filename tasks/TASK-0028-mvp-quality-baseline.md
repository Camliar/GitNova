# TASK-0028: MVP Quality Baseline

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/028-mvp-quality-baseline`
- **Dependencies:** TASK-0027 (`61c8648`)

## Goal

把 Desktop Squash Trace MVP 的性能、隐私、网络透明性和端到端可靠性要求固化为本地可重复运行的质量门槛。

## Scope

- Core 冷启动进程基线：多次 initialize/EOF，记录 p95 并执行 500ms 目标与宽松防抖上限。
- 保持 history/graph 分页上限、16MiB frame、15s Host timeout、Core 子进程清理等既有边界，并在质量文档中形成矩阵。
- GitHub UI 明确显示 network off/user-triggered 状态，以及 connect、PR、commit patch、Squash Trace 各动作的数据范围。
- Core stderr 不输出底层 transport error 细节；自动检查生产源码无 Desktop 直连网络 API、调试输出或宽松 CSP。
- 增加本地 `check:quality`，并与现有 protocol/frontend/Rust checks 一起验证。
- 明确 Windows/macOS/Linux 验证矩阵与尚未执行项；CI、签名、安装包和发布仍属于 TASK-0029。

## Non-goals

- CI/CD workflow、代码签名、公证、installer、自动更新、发布上传。
- 更换 Provider、远程 Core launcher、IDE Host、业务功能或性能优化猜测。

## Deliverables

- [x] cold-start performance regression test and quality matrix
- [x] network disclosure UI and tests
- [x] privacy/static quality gate and documented commands

## Review Checklist

- [x] 性能门槛可重复且避免单次调度抖动误报。
- [x] 每次 GitHub 网络动作在触发前说明范围，无自动请求。
- [x] Host 无直接 fetch/HTTP，CSP 默认拒绝远程连接，stderr 无敏感细节。
- [x] quality/frontend/Rust/Clippy/build/Tauri 验证通过；未提前实现 TASK-0029。

## Done Definition

- [x] 自主 Review 无阻塞项，状态 Done，提交推送并快进合并 main。
