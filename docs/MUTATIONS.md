# Repository Mutations

Core protocol 1.19 exposes explicit, worktree-only local and remote mutations. Hosts must call them only after direct user intent; branch checkout, Pull and Push require a visible confirmation flow. Core never runs them during repository open, status refresh, history loading, Provider access, Squash Trace, or AI Assist.

## Staged commit

`repository/commit` accepts `{ "message": string }`. The message must contain non-whitespace text and be no larger than 65,536 UTF-8 bytes. Core checks unresolved index conflicts and verifies that the index has staged paths, then invokes System Git with `git commit --file=- --cleanup=verbatim`. The message is written over stdin and is not placed in process arguments. System Git identity, hooks and signing configuration remain authoritative; Core does not bypass them.

Success returns `CommitResult`: the parsed new `CommitSummary` plus an authoritative post-mutation snapshot containing `WorkingTreeStatus` and `RepositoryReferences`. Failure does not fabricate a commit result. Untracked and unstaged changes are not added to the index.

## Local branches

`repository/createBranch` and `repository/switchBranch` accept `{ "name": string }`. Core validates names with System Git.

- create starts at current `HEAD`, creates only `refs/heads/<name>`, and does not switch; existing names and unborn HEAD are stable errors;
- switch accepts only an existing local branch and uses `git switch --no-guess`; it does not guess a remote, stash, force, reset, restore, or discard changes;
- Git checkout safety remains authoritative, so a conflicting working tree causes `git.mutation_failed` and stays available for retry after the user resolves the state.

Both methods return the post-mutation status/reference snapshot. Bare repositories return `repository.worktree_required`.

## Remote branch checkout

Core advertises `repository/checkoutRemoteBranch` through optional `remoteBranchCheckout`. It accepts `{ "fullName", "expectedHeadOid" }`: `fullName` must be the opaque full ref previously returned by Core, and `expectedHeadOid` binds execution to the HEAD shown during confirmation.

Before mutation, Core re-enumerates references and requires an exact, non-symbolic `remoteBranch` match. It resolves the configured remote with the longest matching name prefix, derives and validates the corresponding local branch inside Core, rejects any local-name collision, and rechecks current HEAD. It then uses System Git to create and switch to a direct tracking local branch from that explicit full ref. Core never guesses a remote, overwrites a local branch, detaches HEAD, forces checkout, stashes, resets, restores, or discards changes. Success returns the authoritative post-mutation snapshot; missing/symbolic refs use `branch.not_found`, collisions use `branch.already_exists`, and confirmation drift uses `branch.stale_head`.

## Fetch, Pull and Push

Core advertises these methods through optional `repositorySync`:

- `repository/fetch` accepts optional `{ "remote": string }`. Without it, Core uses the current upstream remote or `origin`. It runs non-interactive `git fetch --no-recurse-submodules <remote>`, without prune or ref deletion.
- `repository/pull` accepts `{ "expectedBranch", "expectedHeadOid" }`. Core rejects stale confirmation, requires a configured upstream, fetches that exact remote, then permits only up-to-date/local-ahead or `git merge --ff-only` against the verified tracking ref. Divergence is `sync.diverged`; Core never creates a merge commit or invokes rebase/stash/reset.
- `repository/push` uses the same confirmation-bound parameters. Core pushes the confirmed full OID to only the configured upstream branch; without upstream, it targets only `origin` plus the same local branch name. The refspec contains no force/delete form, and Core does not change tracking configuration.

All three methods require an attached, born branch in a non-bare worktree and return `RepositorySyncResult`: operation, normalized remote/branch target and an authoritative post-operation snapshot. Remote names must be present in `git remote`; Hosts neither parse upstreams nor construct refspecs. Git runs with interactive terminal/credential prompting disabled. stderr, remote URLs and credential material never cross JSON-RPC.

## Deliberate limits

This contract does not stage paths, amend, override author, bypass hooks, configure signing, detach HEAD, delete/rename branches, edit remotes/upstreams, force/delete/prune remote refs, auto-stash, merge-pull, rebase-pull, reset or restore. Bare-repository sync also remains outside this slice.

## Desktop workflow

Desktop 只在 Core 声明 `repositoryMutations` capability、已打开非 bare worktree 且 status 可用时显示操作区。每项 mutation 都经过 Review 和 Confirm 两次明确操作；打开仓库、刷新 status/history 或读取 refs 不会触发 mutation。

Commit 预览显示 Core status 中 index 非 `unmodified` 的 path 数量，但是否允许 commit 仍由 Core 决定。左侧分支树是唯一分支上下文入口：本地分支 Checkout 复用 `repository/switchBranch`；非 symbolic remote branch 的右键/actions 菜单调用 `repository/checkoutRemoteBranch`。Host 只回传完整 remote ref、显示名称和确认时 HEAD OID，不拆分 remote 名或构造 local branch。成功后 Desktop 直接采用 Core 返回的 status/references snapshot、清除可能失效的 diff/detail 并刷新 graph；失败保留 action，可 Retry 或 Cancel，且不显示成功状态。

Desktop 顶部只在 capability、non-bare worktree 与 current branch/HEAD 同时可用时显示 Fetch/Pull/Push。Fetch 点击即执行并显示进度；Pull 在没有 upstream 时禁用；Pull/Push 的确认框固定显示 branch、短 OID 与禁止的 merge/rebase/stash/force/delete 语义。成功统一采用 Core snapshot 并刷新 history；网络失败保留仓库工作区和明确 Retry。
