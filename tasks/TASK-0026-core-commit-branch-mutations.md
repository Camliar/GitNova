# TASK-0026: Core Commit and Branch Mutations

- **Status:** In Progress
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/026-core-commit-branch-mutations`
- **Dependencies:** TASK-0025 (`f8f16f3`), Core protocol `1.11`

## Goal

由 Core 提供最小、显式且可审计的 staged commit、创建 local branch 与切换 local branch 能力，供所有 Host 复用。

## Scope

- `repository/commit` 只提交当前 index，message 必须非空且不通过进程参数暴露；允许 System Git hooks，失败不宣称成功。
- commit 前区分 no staged changes 与 unresolved conflicts，成功返回新 commit、最新 status 与 references。
- `repository/createBranch` 从当前 HEAD 创建 local branch但不切换；验证 branch name、existing branch 与 unborn HEAD。
- `repository/switchBranch` 只切换已存在的 local branch；不自动 stash、不丢弃修改，System Git 的 checkout safety 生效。
- mutation 成功返回最新 status 与 references，让 Host 不自行重建 Git 状态。
- worktree-only、显式请求、稳定错误、严格 params、协议/capability/Schema/TS/Rust/文档与 contract/integration tests。

## Non-goals

- staging/unstaging、amend、author override、签名配置、绕过 hooks、detached checkout。
- delete/rename branch、force、reset、restore、stash、merge/rebase、fetch/pull/push。
- Desktop UI、GitHub/PR/Squash Trace 或 AI。

## Deliverables

- [ ] staged commit mutation and structured result
- [ ] create/switch local branch mutations and structured result
- [ ] protocol, stable errors, integration/contract tests and docs

## Review Checklist

- [ ] mutation 仅由显式 JSON-RPC 请求触发，message 通过 stdin 交给 Git。
- [ ] branch 操作无 force/guess/stash/discard，失败保持 Git 的安全边界。
- [ ] Core 返回 authoritative post-mutation snapshots，Host 无业务逻辑。
- [ ] fmt、Rust tests、Clippy、protocol check 与 Desktop tests 通过。

## Done Definition

- [ ] 自主 Review 无阻塞项，状态 Done，提交推送并快进合并 main。
