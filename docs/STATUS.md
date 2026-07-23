# Working Tree Status

## Contract

`repository/status` returns the read-only status of the repository previously selected with `repository/open`. The method accepts no parameters and requires a non-bare worktree.

Core executes:

```text
git -C <worktree> status --porcelain=v2 -z --branch --untracked-files=all --renames
```

The command is launched with a parameterized process API, `GIT_OPTIONAL_LOCKS=0`, and no shell. Host applications receive structured protocol values and must not execute or parse Git themselves.

## Result

The result contains `branch` and ordered `entries`.

`branch` provides:

- `head`: branch name, or `null` for detached HEAD.
- `oid`: current commit object ID, or `null` for an initial unborn branch.
- `upstream`: upstream ref, or `null` when none is configured.
- `ahead` / `behind`: divergence from upstream; both are zero without an upstream.

Each entry provides:

- `path` and optional `originalPath` for a rename or copy.
- `kind`: `ordinary`, `renameOrCopy`, `unmerged`, or `untracked`.
- independent `indexStatus` and `worktreeStatus` values.

Status values are `unmodified`, `modified`, `added`, `deleted`, `renamed`, `copied`, `unmerged`, `untracked`, `typeChanged`, or `unknown`. `unknown` preserves forward compatibility when a newer System Git introduces an unrecognized XY code.

Core preserves porcelain output order. Paths must be losslessly representable as JSON strings; otherwise the request fails with `path.unsupported_encoding` rather than returning partial or corrupted results.

## Errors

- `repository.not_open`: call `repository/open` first.
- `repository.worktree_required`: the active repository is bare.
- `git.status_parse_failed`: System Git returned a malformed porcelain payload.
- Existing `git.unavailable`, `git.command_failed`, `repository.unsafe_ownership`, and `path.unsupported_encoding` errors retain their meanings.

Raw Git stderr and repository content are never included in the response.

## Deliberate limits

Status is a snapshot requested by the Host. This Task does not add file watching, automatic refresh, ignored files, submodule detail, staging, discard, commit, branch operations, history, or GitHub data. Single-file content changes are available through the separate [structured diff contract](DIFF.md).

## Desktop presentation

Desktop 在 `repository/open` 成功后为非 bare 仓库请求一次 snapshot，并只在用户点击 Refresh 时再次请求。Host 按 Core 返回顺序展示 entries，分别呈现 `indexStatus` 与 `worktreeStatus`；rename/copy 的 `originalPath`、untracked 和 unmerged 状态不会被折叠或重新推断。

bare repository 不发送 `repository/status`。请求失败时，已打开的 `RepositoryDescriptor` 保持有效，UI 仅显示稳定错误并允许重试。Desktop 不监听文件系统、不轮询、不解析 porcelain，也不在此视图提供任何修改仓库的操作。
