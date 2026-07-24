# Repository Mutations

Core protocol 1.12 exposes three explicit, worktree-only mutations. Hosts must show their own confirmation flow and call these methods only after direct user intent. Core never runs them during repository open, status refresh, history loading, GitHub access, or Squash Trace.

## Staged commit

`repository/commit` accepts `{ "message": string }`. The message must contain non-whitespace text and be no larger than 65,536 UTF-8 bytes. Core checks unresolved index conflicts and verifies that the index has staged paths, then invokes System Git with `git commit --file=- --cleanup=verbatim`. The message is written over stdin and is not placed in process arguments. System Git identity, hooks and signing configuration remain authoritative; Core does not bypass them.

Success returns `CommitResult`: the parsed new `CommitSummary` plus an authoritative post-mutation snapshot containing `WorkingTreeStatus` and `RepositoryReferences`. Failure does not fabricate a commit result. Untracked and unstaged changes are not added to the index.

## Local branches

`repository/createBranch` and `repository/switchBranch` accept `{ "name": string }`. Core validates names with System Git.

- create starts at current `HEAD`, creates only `refs/heads/<name>`, and does not switch; existing names and unborn HEAD are stable errors;
- switch accepts only an existing local branch and uses `git switch --no-guess`; it does not guess a remote, stash, force, reset, restore, or discard changes;
- Git checkout safety remains authoritative, so a conflicting working tree causes `git.mutation_failed` and stays available for retry after the user resolves the state.

Both methods return the post-mutation status/reference snapshot. Bare repositories return `repository.worktree_required`.

## Deliberate limits

This contract does not stage paths, amend, override author, bypass hooks, configure signing, detach HEAD, delete/rename branches, set upstreams, or run reset/restore/stash/merge/rebase/fetch/pull/push. Those require separate Tasks and explicit safety contracts.

## Desktop workflow

Desktop 只在 Core 声明 `repositoryMutations` capability、已打开非 bare worktree 且 status 可用时显示操作区。每项 mutation 都经过 Review 和 Confirm 两次明确操作；打开仓库、刷新 status/history 或读取 refs 不会触发 mutation。

Commit 预览显示 Core status 中 index 非 `unmodified` 的 path 数量，但是否允许 commit 仍由 Core 决定。Branch switch 下拉只包含 Core `repository/references` 返回的 `localBranch`，不把 remote refs 猜测为可切换分支。成功后 Desktop 直接采用 Core 返回的 status/references snapshot、清除可能失效的 diff/detail 并刷新 graph；失败保留输入与 action，可 Retry 或 Cancel，且不显示成功状态。
