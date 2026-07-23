# TASK-0005: Structured File Diff

- **Status:** In Progress
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/005-structured-file-diff`
- **Estimate:** 2–3 days
- **Dependencies:** TASK-0004 working tree status baseline (`64f4085`)

## Goal

为已打开仓库中的单个文件返回 staged 或 working-tree 的结构化行级 diff，使所有 Host 可直接呈现 hunk 和新旧行号，而不复制 Git patch 解析逻辑。

## Context

TASK-0004 已提供工作区状态和变更路径。本 Task 只扩展单文件只读 diff，调用 System Git 稳定选项并在 Core 中解析 unified patch。

## Scope

- 新增 `repository/diff` request，参数为 `path`、`scope` 和可选 `contextLines`。
- `scope` 为 `workingTree`（index 对 working tree）或 `staged`（HEAD 对 index）。
- `contextLines` 默认 3，范围 0–20；超出范围返回 invalid params。
- 路径必须是 repository-relative 的单个文件路径；禁止绝对路径、空路径、`.`/`..` 逃逸和 pathspec magic。
- 使用 System Git `diff --patch --no-color --no-ext-diff --no-textconv --find-renames --unified=N -- <path>`；staged 增加 `--cached`。
- 返回 `isBinary`、old/new path 与 structured hunks；hunk 包含新旧起始/行数、header 和 ordered lines。
- line 返回 `context`/`addition`/`deletion`、content 以及 nullable old/new line number。
- 正确忽略 `\ No newline at end of file` marker，不把它当作用户内容。
- binary diff 返回 `isBinary: true` 和空 hunks，不返回二进制内容。
- 无变更返回 `isBinary: false` 和空 hunks，不作为错误。
- 未 open、bare、非 UTF-8 patch、无效路径和 parser 失败返回稳定错误。
- 协议升级至 `1.3`，增加 `structuredFileDiff` capability，同步 Rust/Schema/TypeScript。
- 为 staged、unstaged、context=0、新增/删除行、无换行、binary、无变更和安全路径提供测试。

## Non-goals

- untracked 文件的合成 diff、目录或全仓库批量 diff。
- 任意 commit/tree/blob 之间的历史 diff。
- stage、unstage、discard、commit 或任何写操作。
- word diff、语法高亮、图片预览或 submodule 详细 diff。
- GitHub、PR、Squash Trace、SQLite 或 Host UI。

## Deliverables

- [ ] `repository/diff` method 与安全 path validation
- [ ] unified patch parser 与结构化 diff 契约
- [ ] Rust、JSON Schema、TypeScript 与 capability 同步
- [ ] 文本、binary、空 diff、参数和仓库边界测试
- [ ] Diff 协议、安全边界和刻意限制文档

## Review Checklist

- [ ] Git 调用与 patch 解析只存在于 Core，不经 shell。
- [ ] path 不能逃逸仓库或被解释为 pathspec magic。
- [ ] hunk 范围、行类型和新旧行号通过测试。
- [ ] binary 不泄露内容，Git stderr 不返回 Host。
- [ ] 未实现批量/历史 diff 或任何写操作。
- [ ] fmt、Clippy、Rust tests、`npm run check` 与文档检查通过。

## Done Definition

- [ ] Deliverables 与 Review Checklist 完成。
- [ ] 自主 Review 无阻塞意见。
- [ ] 状态更新 Done，提交并推送。

## References

[Status](../docs/STATUS.md) · [Repositories](../docs/REPOSITORIES.md) · [Protocol](../docs/PROTOCOL.md) · [Coding Standard](../docs/CODING_STANDARD.md)
