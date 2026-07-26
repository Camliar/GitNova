# Lazy Structured Commit Diff

协议 1.16 的主路径将 commit metadata/file discovery 与 patch materialization 分开，避免大 commit 把数千个 patch 塞入单次 16 MiB JSON-RPC frame，或在 Desktop 的交互超时内启动数千个 Git 进程。它适用于 worktree、detached 和 bare repository，且不读取 working-tree state。

## Request

- `oid` is a required full 40- or 64-character hexadecimal commit OID.
- `parentOid` is optional. A root commit uses the empty-tree comparison and rejects a parent. A single-parent commit selects its only parent by default. A merge commit requires one of its direct parents explicitly.
- `repository/commitFiles` 接受 `oid` 与 optional `parentOid`，返回 commit metadata、实际 parent edge 及有序 changed-file metadata，不读取 patch。
- `repository/commitFileDiff` 额外接受已选文件的 repository-relative `path`；`contextLines` 默认为 3，范围为 0–20，并只返回该文件的 structured diff。
- `repository/commitDiff` 保留为兼容旧 Host 的 eager 方法；声明 `lazyCommitDiff: true` 的 Core/Host 不应在交互路径中使用它。

Core verifies the raw commit object and direct parent relationship before diffing. Invalid syntax is `protocol.invalid_params`; a missing object is `commit.not_found`; an omitted merge parent is `commit.parent_required`; a non-parent is `commit.invalid_parent`.

## Result and implementation

`repository/commitFiles` 的结果包含 `commit`、nullable `parentOid` 和 `CommitChangedFile[]`；每项仅含 old/new path 与 status。`repository/commitFileDiff` 返回一个 `FileDiff`，继续复用 [`repository/diff`](DIFF.md) 的 hunk、line kind、line number、rename、binary 与 empty-file 契约。

Core 直接调用 System Git。文件发现使用 NUL-delimited `--name-status -z`；单文件请求先重新验证 path 属于同一个 commit-parent edge，再用 literal pathspec 请求 patch，并禁用 external diff/text conversion。Core 不解析面向人的 `git log` 输出，也不调用 shell。

文件 metadata 最多 20,000 项且 path payload 最多 4 MiB；单文件 patch 最多 8 MiB。超限分别返回 `commit.file_limit` 与 `commit.file_diff_limit`，而不是让 transport 超时或超过 frame 限制。

## Deliberate limits

The method compares exactly one commit-parent edge. It does not produce combined merge diffs, choose a preferred merge parent, compare arbitrary trees/ranges, include working-tree changes, apply patches, or retrieve remote-only objects.

## Desktop presentation

Desktop 从 Core graph node 获取完整 commit OID 与 ordered parents。root 和 single-parent commit 省略 `parentOid`，由 Core 按契约选择 empty tree 或唯一 parent；merge commit 在发送请求前要求用户明确选择一个 direct parent，Host 不猜测 first/preferred parent。请求固定使用 3 行 context。

详情先展示 Core 返回的完整 message、author/committer、实际 `parentOid` 和 ordered file metadata。Changes 使用固定纵向文件列表；只有点击文件名才加载该文件 patch，并复用 structured `FileDiff` renderer。单文件错误保留 timeline、commit metadata、文件列表与选择状态，可独立 Retry；关闭或切换 selection 后迟到 response 不会恢复旧详情。repository/history snapshot reload 会清除详情，working-tree status Refresh 不会。
