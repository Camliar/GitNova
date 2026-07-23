# TASK-0004: Working Tree Status

- **Status:** Done
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/004-working-tree-status`
- **Estimate:** 2–3 days
- **Dependencies:** TASK-0003 repository discovery baseline (`42ac9d4`)

## Goal

为已打开的 non-bare repository 提供只读、结构化且可跨 Host 复用的工作区状态，让 Host 无需运行或解析 Git 命令即可展示 staged、unstaged、untracked、rename/copy、conflict 与分支上下游摘要。

## Context

TASK-0003 已在 Core 会话中建立唯一活动仓库。本 Task 是 Local Git Foundation 的下一个只读垂直切片，使用 System Git `status --porcelain=v2 -z --branch`。porcelain 是稳定机器协议；人类可读输出、本地化 stderr 或 Host 侧解析均不得成为业务契约。

## Scope

- 定义 `repository/status` request，仅在 initialize 且 `repository/open` 成功后可用。
- 调用 System Git `status --porcelain=v2 -z --branch --untracked-files=all`，不经 shell，不获取 ignored files。
- 返回 branch HEAD、OID、upstream、ahead/behind 以及 ordered status entries。
- 解析 ordinary、rename/copy、unmerged 和 untracked records；保留 index/worktree 两维状态。
- 将 Git porcelain XY 字符映射为稳定 enum：unmodified、modified、added、deleted、renamed、copied、unmerged、untracked、typeChanged、unknown。
- rename/copy entry 返回 `originalPath`；其他 entry 为 `null`。
- 保持 Git 输出顺序；不在 Core 或 Host 重新排序。
- 路径必须可由 JSON string 无损表示；否则整个请求返回 `path.unsupported_encoding`。
- bare repository 返回 `repository.worktree_required`；未 open 返回 `repository.not_open`。
- 扩展 Core capability、Rust 类型、JSON Schema 与生成 TypeScript 类型。
- 覆盖 clean、staged、unstaged、untracked、rename、conflict、detached HEAD、upstream ahead/behind、bare 和未 open 测试。

## Protocol Baseline

- Method: `repository/status`；params 为空 object 或省略。
- Result: `{ branch, entries }`。
- `branch`: `{ head, oid, upstream, ahead, behind }`；detached HEAD 时 `head` 为 `null`，无 upstream 时 upstream/ahead/behind 分别为 `null`/`0`/`0`。
- entry: `{ path, originalPath, kind, indexStatus, worktreeStatus }`。
- 协议版本从 `1.1` 增量升级为 `1.2`，并声明 `workingTreeStatus` capability。

## Non-goals

- 文件内容、patch、行级 diff 或 binary diff。
- stage、unstage、discard、commit、checkout 或其他写操作。
- branch/tag/remote 列表或历史图。
- ignored files、submodule 详细状态或 file watcher/自动刷新。
- GitHub、PR、Squash Trace、SQLite 或 Host UI。

## Deliverables

- [x] `repository/status` Core method 与 System Git 适配
- [x] porcelain v2 `-z` branch/status parser
- [x] Rust、Schema 与 TypeScript 契约
- [x] clean/变更/rename/conflict/branch/bare/error 自动测试
- [x] 工作区状态协议和边界文档
- [x] README、Feature List 与 Roadmap 状态同步

## Review Checklist

- [x] 只调用 System Git porcelain v2，Host 无 Git 调用或解析逻辑。
- [x] 命令不经 shell，无仓库写操作。
- [x] XY、rename/copy、unmerged、untracked 和 branch headers 解析可测试。
- [x] 未 open、bare、非 UTF-8 路径与 Git 失败具有稳定错误。
- [x] 未实现 Diff、stage、commit、branch、history、GitHub 或 UI。
- [x] Rust/Schema/TypeScript 类型与 capability 一致。
- [x] fmt、Clippy `-D warnings`、Rust tests、`npm run check` 与文档检查通过。

## Done Definition

- [x] Deliverables 和 Review Checklist 全部完成。
- [x] 自主 Review 无阻塞意见。
- [x] 状态更新为 Done 并提交推送。

## Review Record

- 2026-07-23：根据用户授权执行自主 Review；范围、架构、安全、协议一致性和自动验证均通过。

## References

[Repositories](../docs/REPOSITORIES.md) · [Protocol](../docs/PROTOCOL.md) · [Roadmap](../docs/ROADMAP.md#phase-2--local-git-foundation当前) · [Coding Standard](../docs/CODING_STANDARD.md)
