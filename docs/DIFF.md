# Structured File Diff

## Contract

`repository/diff` returns a structured, read-only diff for one repository-relative file in the active non-bare worktree.

Parameters:

- `path`: safe repository-relative file path.
- `scope`: `workingTree` compares index to working tree; `staged` compares HEAD to index.
- `contextLines`: optional integer from 0 through 20; default 3.

Core invokes System Git with patch, no-color, no-external-diff, no-textconv, rename detection, explicit context, and `--literal-pathspecs`. The path follows `--` as a separate process argument. No shell is involved.

## Result

The result includes `oldPath`, `newPath`, `isBinary`, and ordered `hunks`. Each hunk provides old/new start and line counts, the optional function header, and ordered lines.

Line kinds are `context`, `addition`, and `deletion`. Context lines carry both old and new line numbers; deletions only carry an old line number; additions only carry a new line number. Core removes the unified-patch prefix but preserves the remaining content exactly. The patch marker for a missing final newline is metadata and is not returned as file content.

Binary changes return `isBinary: true` with no hunks or binary payload. A tracked file with no change in the selected scope returns a non-binary result with no hunks.

## Path safety

Paths cannot be empty, absolute, contain empty/`.`/`..` segments, or begin with Git pathspec magic (`:`). Core also passes `--literal-pathspecs`, so repository-controlled names cannot become glob or attribute selectors. Untracked-file synthesis is deliberately excluded; the status result can still identify such files.

## Errors

- `repository.not_open`: no active repository.
- `repository.worktree_required`: active repository is bare.
- `path.invalid_repository_relative`: path violates the safety contract.
- `path.unsupported_encoding`: patch content cannot be represented losslessly in JSON.
- `git.diff_parse_failed`: System Git returned malformed unified patch data.
- `git.unavailable` and `git.command_failed`: System Git execution failed.

Git stderr and binary content are never forwarded to Host applications.

## Deliberate limits

This method does not provide directory-wide, repository-wide, untracked synthetic, commit/tree/blob, word, image, or submodule-detail diff. It does not stage, unstage, discard, commit, or otherwise modify the repository.

## Desktop presentation

Desktop 从 Core `repository/status` entry 提供的 path 出发，让用户分别选择 staged 或 working-tree scope，并以固定 3 行 context 请求 `repository/diff`。只有对应 scope 的状态不是 `unmodified` 时才提供操作；`untracked` 不触发 synthetic diff。

Host 直接呈现 Core 返回的 old/new path、binary 标记、hunks、行类型和行号，不解析 unified patch，也不读取文件。行内容始终通过 React text node 渲染，不作为 HTML、ANSI 或其他可执行表示解释。status Refresh 会关闭旧 diff；请求失败保留 status snapshot 和选择上下文并允许 Retry。
