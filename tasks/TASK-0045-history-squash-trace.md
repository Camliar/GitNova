# TASK-0045: History-integrated Squash Trace

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `codex/task-045-history-squash-trace`
- **Dependencies:** completed TASK-0044 baseline (`85ef85d`)

## Goal

把 GitNova 的核心差异化能力放进 All Commits 主工作流：普通/root/merge commit 继续显示本地 Commit 与按文件 Changes；对用户显式检查并由 GitHub + 本地证据关联到 PR 的 squash candidate，默认先显示 PR original commits，再点击 original commit 查看其提交信息、changed files 与文件 diff。

## Evidence and Decisions

- GitHub REST `GET /repos/{owner}/{repo}/commits/{commit_sha}/pulls` 返回把该 commit 引入仓库的关联 PR。
- GitHub 文档明确：已 squash merge 的 PR，其 `merge_commit_sha` 是 base branch 上的 squashed commit；普通 merge 是 merge commit，rebase 是 base branch 更新到的 commit。
- Core 只接受“merged PR 且 `merge_commit_sha` 与所选完整 OID 精确相等”的候选；没有精确候选是正常的 ordinary commit，多个精确候选 fail closed，不由 Host 猜测。
- GitHub 不提供单独按 changed-file 获取 commit patch 的端点。Core 可在显式加载 original commit 时获取、验证并有界缓存完整 Provider 响应，但只把 file metadata 返回 Host；点击文件时才从 Core 缓存返回该文件 patch。

官方依据：

- https://docs.github.com/en/rest/commits/commits#list-pull-requests-associated-with-a-commit
- https://docs.github.com/en/rest/pulls/pulls#get-a-pull-request

## Scope

- 协议 1.17 增加 optional `historySquashTrace` capability。
- 增加 Core-owned `github/commitSquashTrace`，用完整本地 commit OID 发现精确关联 PR，并返回 nullable Squash Trace。
- 增加 lazy `github/pullRequestCommitFiles` 与 `github/pullRequestCommitFileDiff`，Provider 原始 patch 只保存在仓库环境内的有界 Core memory cache。
- Desktop commit 详情提供显式 `Check Squash Trace`；请求期间本地 Commit/Changes 始终可用。
- confirmed/candidate trace 自动打开 Squash Trace tab：顶部显示 `original commits → final commit` 关系，左侧按 PR 顺序显示紧凑 original commit 列表，右侧显示所选 original commit metadata/Changes。
- original commit 的 Changes 先显示纵向文件列表；点击文件名才显示 patch/unavailable 状态。
- 选择另一 timeline commit、关闭详情或切换仓库时使所有迟到 Provider response 失效。

## Non-goals

- 自动后台访问 GitHub，或仅凭 commit message 中的 `#number` 推断 PR。
- 把 GitHub/Git/Squash 关联逻辑放进 Desktop。
- 宣称 single-parent relationship 必然是 squash；没有 Provider merge-method metadata 时继续显示 `Squash candidate` 与 medium confidence。
- GitLab history association（现有 GitLab Squash Trace Core API 保持不变；Desktop history integration 后续按独立 Provider parity Task 扩展）。
- Fetch/Pull/Push（TASK-0046）与 branch context menu（TASK-0047）。

## Deliverables

- [x] protocol 1.17 association and lazy original-commit diff contract
- [x] exact GitHub commit-to-PR association with ambiguous/no-match handling
- [x] bounded Core cache that never exposes all original patches to Host
- [x] history Squash Trace/original commits/detail/Changes UI
- [x] network disclosure, stale-response and error recovery tests
- [x] synchronized docs and Host protocol versions

## Review Checklist

- [x] Network access occurs only after direct user action and remains Core-owned.
- [x] Association uses full OIDs and Provider-confirmed `merge_commit_sha`; Host parses no PR hints.
- [x] Ordinary and Core-classified merge commits remain in local Commit/Changes when association is absent, fails, or is not a squash candidate.
- [x] Original commit order comes from Provider, and only listed commits/files can be opened.
- [x] Provider response/cache/file counts and bytes are bounded; repository environment switch replaces the Core process/cache.
- [x] Protocol, Rust, Desktop, Host, quality and delivery checks pass.

## Done Definition

- [x] Deliverables and review checklist complete.
- [x] Autonomous review has no blocking findings.
- [x] Status Done; branch committed/pushed and fast-forwarded into verified remote `main`.

## Verification

- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- Desktop TypeScript, 76 Vitest tests and production Vite build
- protocol generation check, quality/delivery gates, VS Code tests/build syntax, JetBrains and Visual Studio Host checks
