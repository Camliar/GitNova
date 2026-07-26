# TASK-0038: CI Platform Build Repair

- **Status:** In Progress
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `codex/task-038-ci-platform-builds`
- **Dependencies:** completed TASK-0037 baseline (`29cf00d`)

## Goal

恢复 Desktop Windows 与 JetBrains 插件的真实 CI 构建门，确保已声明的平台配置能由干净 runner 完成编译。

## Scope

- 从既有 Desktop 图标生成并纳入 Tauri Windows resource 所需的 `icons/icon.ico`，同时在 bundle 配置中显式声明。
- 扩展 delivery 静态检查，缺失 Windows 图标时在跨平台编译之前失败并给出明确原因。
- 将 JetBrains CI 与 Gradle Java toolchain 从 Java 17 统一到 IntelliJ IDEA 2026.2 class files 所需的 Java 25。
- 同步 JetBrains Host 与交付文档中的工具链说明。
- 运行 repository checks、Rust/Tauri build-script 和 JetBrains Gradle 构建；本机没有 Java 25 时，以静态检查和远程 CI 作为 JetBrains 真实编译门。

## Non-goals

- 修改 Desktop 品牌设计、应用标识或签名策略。
- 升降级 IntelliJ IDEA、Gradle 或 IntelliJ Platform Gradle Plugin。
- 修改 Git、Provider、Squash Trace 或 AI Assist 业务能力。
- 扩展新的 CI runner 或发布目标。

## Deliverables

- [x] valid multi-resolution Windows ICO and explicit Tauri bundle entry
- [x] delivery regression check for the required Windows resource
- [x] Java 25 toolchain alignment for IDEA 2026.2 in Gradle and CI
- [x] synchronized Host documentation and passing local validation

## Review Checklist

- [x] Windows `tauri-build` no longer depends on an absent conventional icon path.
- [x] ICO contains appropriate Windows application sizes and remains derived from the checked-in GitNova mark.
- [x] CI configuration selects Java 25 for the Gradle daemon and Java compiler toolchain.
- [x] Local checks introduce no unrelated product or architecture changes.
- [ ] Remote CI confirms Windows Desktop and JetBrains jobs.

## Done Definition

- [ ] 自主 Review 无阻塞项，状态 Done，提交推送并快进合并 main。
