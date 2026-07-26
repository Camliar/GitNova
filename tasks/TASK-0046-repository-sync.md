# TASK-0046: Repository Fetch, Pull and Push

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `codex/task-046-repository-sync`
- **Dependencies:** completed TASK-0045 baseline (`a257983`)

## Goal

在 Desktop 顶部提供可用的 Fetch、Pull、Push 操作，同时保持 System Git、远程/跟踪分支解析、并发校验和失败分类全部属于 `gitnova-core`。操作完成后返回一致的 status/references snapshot，并刷新 history；Host 不拼接 refspec、不运行 Git，也不解析 stderr。

## Safety Decisions

- 协议 1.18 增加 optional `repositorySync` capability 与 Core-owned `repository/fetch`、`repository/pull`、`repository/push`。
- 本 Task 的三个操作只支持 non-bare worktree；bare repository sync 留给独立 Task。
- Fetch 明确选择 remote，默认优先当前 branch upstream remote，其次 `origin`；不 prune、不删除 refs。
- Pull 必须已有 upstream。Core 先 fetch 指定 upstream remote，再验证本地/远程关系；只允许 up-to-date、local-ahead 或 fast-forward，分叉时拒绝，不自动 merge/rebase/stash/reset。
- Push 使用已有 upstream；没有 upstream 时只允许把当前 branch 推送到 `origin` 的同名 `refs/heads/*`，不隐式改写 tracking 配置。永不 force、delete 或推送额外 refspec。
- Pull/Push 参数绑定用户确认时看到的完整 HEAD OID 与 local branch；HEAD 变化时返回稳定 stale error，不执行网络 mutation。
- 所有网络 Git 子进程禁用交互式凭据提示，参数以 argv 传递；Core/Tauri 错误不返回 stderr、URL 或 credentials。
- Fetch 是显式按钮动作；Pull 和 Push 必须在 Host 显示 exact branch/HEAD、安全语义并二次确认。

## Scope

- protocol/schema/Rust/TypeScript 1.18 sync contract、capability 与稳定错误。
- Core remote/upstream 解析、stale guard、non-interactive Fetch、fast-forward-only Pull 与 non-force Push。
- 本地 bare remote 集成测试，覆盖 fetch、fast-forward pull、push、divergence/non-fast-forward 与 stale HEAD。
- Desktop 顶部紧凑 Fetch/Pull/Push controls、确认/进度/错误/成功反馈与 stale-response protection。
- Sync 成功后统一刷新 working tree、references、history 与详情选择。
- Provider-style 45-second Desktop transport ceiling，避免远程 Git 被本地 15-second read timeout 错杀。

## Non-goals

- Force push、delete remote branch、prune、auto-stash、merge pull、rebase pull、冲突解决。
- 凭据/login UI、后台 polling、自动 fetch、网络连接探测。
- remote 管理、upstream 编辑、bare repository sync。
- branch context menu（TASK-0047）。

## Deliverables

- [x] protocol 1.18 repository sync contract
- [x] Core safe fetch/pull/push implementation
- [x] local remote integration and failure regression tests
- [x] Desktop toolbar workflow and confirmation states
- [x] synchronized privacy, mutation, transport and UI docs

## Review Checklist

- [x] Host executes no Git and constructs no remote refspec.
- [x] Pull cannot create merge commits or rebase/stash/reset user work.
- [x] Push cannot force/delete or target an unrelated branch.
- [x] Stale branch/HEAD prevents Pull/Push before network execution; Push additionally uses the confirmed full OID as its source ref.
- [x] Errors and logs expose no stderr, URL, path, or credential material.
- [x] Protocol, Rust, Desktop, Host, quality and delivery checks pass.

## Done Definition

- [x] Deliverables and review checklist complete.
- [x] Autonomous review has no blocking findings.
- [x] Status Done; branch committed/pushed and fast-forwarded into verified remote `main`.

## Verification

- `cargo test --workspace` (40 Core contract tests, including local bare remote sync)
- `cargo clippy --workspace --all-targets -- -D warnings`
- Desktop TypeScript, 79 Vitest tests and production Vite build
- protocol generation check, quality/delivery gates, VS Code tests/build syntax, JetBrains and Visual Studio Host checks
