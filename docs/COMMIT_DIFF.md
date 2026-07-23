# Structured Commit Diff

`repository/commitDiff` returns commit metadata plus an ordered list of structured file diffs from the repository opened in the current Core session. It works in worktree, detached, and bare repositories and never reads working-tree state.

## Request

- `oid` is a required full 40- or 64-character hexadecimal commit OID.
- `parentOid` is optional. A root commit uses the empty-tree comparison and rejects a parent. A single-parent commit selects its only parent by default. A merge commit requires one of its direct parents explicitly.
- `contextLines` defaults to 3 and accepts 0–20.

Core verifies the raw commit object and direct parent relationship before diffing. Invalid syntax is `protocol.invalid_params`; a missing object is `commit.not_found`; an omitted merge parent is `commit.parent_required`; a non-parent is `commit.invalid_parent`.

## Result and implementation

The result contains `commit`, nullable `parentOid`, and `files`. Each file uses the same `FileDiff`, hunk, line-kind, and line-number contract as [`repository/diff`](DIFF.md), including rename, binary, and empty-file behavior.

Core invokes System Git directly. It obtains the ordered change list with NUL-delimited `--name-status -z`, then requests a patch for each literal repository path with external diff and text conversion disabled. It does not parse human-oriented `git log` output or invoke a shell.

## Deliberate limits

The method compares exactly one commit-parent edge. It does not produce combined merge diffs, choose a preferred merge parent, compare arbitrary trees/ranges, include working-tree changes, apply patches, or retrieve remote-only objects.

## Desktop presentation

Desktop 从 Core graph node 获取完整 commit OID 与 ordered parents。root 和 single-parent commit 省略 `parentOid`，由 Core 按契约选择 empty tree 或唯一 parent；merge commit 在发送请求前要求用户明确选择一个 direct parent，Host 不猜测 first/preferred parent。请求固定使用 3 行 context。

详情展示 Core 返回的完整 message、author/committer、实际 `parentOid` 和 ordered files。文件选择复用 structured `FileDiff` renderer，包括 rename path、binary、empty、hunks 与行号；message 和 line content 仅作为 text 呈现。错误保留 timeline 和 oid/parent selection 以便 Retry，关闭或切换 selection 后迟到 response 不会恢复旧详情。repository/history snapshot reload 会清除详情，working-tree status Refresh 不会。
