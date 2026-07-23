# TASK-0008: Repository References

- **Status:** In Progress
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/008-repository-references`
- **Estimate:** 1–2 days
- **Dependencies:** TASK-0007 structured commit diff (`1c9db18`)

## Goal

为已打开仓库返回结构化 HEAD、local branches、remote-tracking branches 和 tags，提供后续 commit graph decorations、分支操作与 PR/Squash Trace 关联所需的只读 reference 基线。

## Scope

- 新增无参数 `repository/references`。
- 返回 `head`：当前 OID 可空、symbolic ref 可空；unborn branch 保留 symbolic ref 且 OID 为 null，detached HEAD 保留 OID 且 symbolic ref 为 null。
- 返回 ordered references：`localBranch`、`remoteBranch`、`tag`。
- 每项包含短名称、完整 refname、直接 target OID、可选 peeled target OID、可选 symbolic target 和可选 upstream ref。
- annotated tag 保留 tag object OID 与 peeled target；lightweight tag 的 peeled target 为空。
- remote symbolic ref（例如 `refs/remotes/origin/HEAD`）作为 remote branch 返回并保留 symbolic target。
- 使用 System Git `symbolic-ref`、`rev-parse` 和 `for-each-ref`；固定 `LC_ALL=C`，使用 NUL field separator，不解析装饰性 `git branch` 输出。
- 支持 worktree、linked worktree、detached HEAD、unborn/empty 和 bare repository。
- 非 UTF-8 reference/upstream/symbolic target 返回稳定错误，不进行损坏替换。
- 协议升级至 `1.6`，增加 `repositoryReferences` capability，同步 Rust、Schema、TypeScript 和文档。
- 测试 local/remote/annotated/lightweight/symbolic refs、upstream、detached、unborn、bare 和错误映射。

## Non-goals

- create/delete/rename branch 或 tag、checkout/switch、设置 upstream、fetch/push。
- commit graph lane/layout、分页历史或 commit diff。
- reflog、stash、notes、replace refs、bisect refs 或任意隐藏 refs。
- GitHub、PR、Squash Trace、SQLite 或 Host UI。

## Deliverables

- [ ] `repository/references` 与 HEAD 状态模型
- [ ] NUL-field `for-each-ref` parser 与 reference 分类
- [ ] Rust、Schema、TypeScript、capability 与稳定错误同步
- [ ] branch/tag/symbolic/upstream/detached/unborn/bare 契约测试
- [ ] reference 语义和刻意限制文档

## Review Checklist

- [ ] 只调用 System Git，不经 shell，无 repository 写操作。
- [ ] HEAD 的 attached/detached/unborn 状态无歧义。
- [ ] annotated tag 的直接与 peeled OID 不丢失。
- [ ] refname、symbolic target 和 upstream 使用稳定结构化字段。
- [ ] 未实现 graph、branch/tag 写操作、fetch/push 或 GitHub。
- [ ] fmt、Clippy、Rust tests、`npm run check` 与文档检查通过。

## Done Definition

- [ ] Deliverables 和 Review Checklist 完成。
- [ ] 自主 Review 无阻塞项。
- [ ] 状态更新 Done，提交并推送。

## References

[Paginated History](../docs/HISTORY.md) · [Repositories](../docs/REPOSITORIES.md) · [Protocol](../docs/PROTOCOL.md) · [Roadmap](../docs/ROADMAP.md#phase-2--local-git-foundation当前)
