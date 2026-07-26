# TASK-0047: Branch Context Menu and Remote Checkout

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `codex/task-047-branch-context-menu`
- **Dependencies:** completed TASK-0046 baseline (`01cd82d`)

## Goal

让左侧 Branches/Remotes 树成为唯一、完整的分支操作入口：右击或使用可访问的 actions 按钮打开菜单，本地分支可 Checkout，远程分支可安全创建并切换到同名 local tracking branch。当前分支继续高亮；顶部不恢复重复 branch selector。

## Decisions

- 协议 1.19 增加 optional `remoteBranchCheckout` capability 与 Core-owned `repository/checkoutRemoteBranch`。
- 请求只接受 Core references 返回的完整 `refs/remotes/*` 与确认时的完整 current HEAD OID；Host 不解析 remote/local branch 名，也不构造 Git 参数。
- Core 重新枚举 refs/remotes，拒绝 symbolic remote HEAD、已消失 ref、stale HEAD、无匹配 configured remote 以及已存在的目标 local branch。
- Core 从最长匹配 configured remote 前缀导出 remote branch 名，经 System Git `check-ref-format` 验证，再使用显式 full ref 创建 direct tracking branch 并切换。
- 现有 local checkout 继续复用 `repository/switchBranch` 与确认框；remote checkout 使用独立确认说明将创建 local tracking branch。
- 菜单支持鼠标右键、可聚焦 actions 按钮、Escape/外部点击关闭；不把 mutation 放进 Host。

## Scope

- protocol/schema/Rust/TypeScript 1.19 remote checkout contract、capability 与 stale error。
- Core remote-ref membership、remote prefix、local name collision、tracking checkout 与 snapshot。
- Desktop branch context menu、local/remote confirmation、loading/error/success refresh。
- Context-menu accessibility and interaction tests plus local remote Git integration test.
- synchronized mutation, protocol, Desktop and UI docs.

## Non-goals

- Delete/rename branch、force checkout、detach at tag/commit、remote delete/prune。
- 任意 local branch 名输入、remote/upstream 管理或冲突自动处理。
- 顶部 branch selector 或 Host-side ref parsing。

## Deliverables

- [x] protocol 1.19 remote branch checkout contract
- [x] Core validated tracking checkout implementation
- [x] accessible branch context menu and unified confirmation
- [x] integration/UI regression tests
- [x] synchronized docs and Host versions

## Review Checklist

- [x] Context menu only offers operations supported for the exact ref kind/state.
- [x] Host passes opaque full ref and confirmation OID without deriving branch semantics.
- [x] Core rejects symbolic/missing/stale/colliding targets before checkout.
- [x] Checkout never force-resets, stashes, discards or guesses a remote.
- [x] Successful checkout refreshes status, references, history and clears stale detail.
- [x] Protocol, Rust, Desktop, Host, quality and delivery checks pass.

## Done Definition

- [x] Deliverables and review checklist complete.
- [x] Autonomous review has no blocking findings.
- [x] Status Done; branch committed/pushed and fast-forwarded into verified remote `main`.

## Verification

- `cargo test --workspace` (96 tests)
- `cargo clippy --workspace --all-targets -- -D warnings`
- Desktop TypeScript, Vite build and 82 Vitest tests
- protocol generation, quality, delivery, VS Code, JetBrains and Visual Studio checks
- local bare-remote contract covering direct tracking, symbolic remote HEAD, local collision, stale HEAD and invalid ref rejection
