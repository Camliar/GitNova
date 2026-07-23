# GitHub Pull Requests and Original Commits

`github/pullRequest` is an explicit Core-owned network request that returns one normalized pull request and its ordered original commit list. This list remains available from GitHub after a squash merge and is the membership source for per-commit diff and later Squash Trace Tasks.

## Request

The request requires a positive `number`. Optional `remote` and `nameWithOwner` select repository identity exactly as described in [GitHub Provider](GITHUB_PROVIDER.md). The Host never supplies credentials or constructs API endpoints.

Core requests PR detail and then calls the PR commits REST endpoint with `per_page=100`, `--paginate`, and `--slurp`. Commit pages are flattened without reordering. The normalized PR includes state, draft/author metadata, base/head refs and OIDs, lifecycle timestamps, and `mergeCommitOid`. A merged PR is reported as `merged` even though the REST `state` value is `closed`.

Each original commit includes its OID, ordered parent OIDs, Git author/committer identity, optional corresponding GitHub login, first-line summary, complete message, and GitHub URL. A missing GitHub user (for example a deleted account or unmatched email) does not remove the Git identity.

## Completeness and limits

GitHub documents a maximum of 250 commits for the PR commits endpoint. Core compares the PR detail count with the flattened list. A count above the supported limit or a capped 250-item response that cannot prove completeness returns `github.pr_commit_limit_exceeded`; any other mismatch is an invalid Provider response. GitNova never silently presents a partial original commit sequence.

Detail responses are limited to 1 MiB and commit pages to 16 MiB. Core does not return raw GitHub JSON, response headers, or command stderr.

## Original commit file and line diff

`github/pullRequestCommitDiff` requires a positive PR `number` and a full 40- or 64-character hexadecimal `oid`, plus the same optional repository selectors. Core first obtains the PR's complete original commit list and rejects an OID outside that list with `github.commit_not_in_pull_request`. This prevents the PR method from becoming an unrestricted remote commit reader.

For a member commit, Core requests `repos/{owner}/{repo}/commits/{oid}?per_page=100` with `--paginate --slurp`. It verifies every page belongs to the requested commit and returns the matching normalized commit plus ordered file records. Each file has old/new paths, normalized status, additions/deletions/changes, patch availability, and structured hunks with old/new line numbers. Renames preserve `previous_filename` as `oldPath`.

GitHub does not include `patch` for every file. GitNova reports those records as `patchState: unavailable` with no hunks; it does not claim that every missing patch is binary. Duplicate paths, inconsistent page OIDs, invalid statistics or malformed patches produce `github.response_parse_failed`. A response reaching GitHub's documented 3000-file limit returns `github.commit_file_limit_exceeded`, because completeness can no longer be proven. Commit-file responses are limited to 32 MiB.

The PR and original commit model feeds the conservative [Squash Trace relationship](SQUASH_TRACE.md). Relationship inference is not duplicated in this Provider response or in Hosts.

Official references: [Get a pull request and list PR commits](https://docs.github.com/en/rest/pulls/pulls), [Get a commit](https://docs.github.com/en/rest/commits/commits#get-a-commit), and [GitHub CLI pagination/slurp](https://cli.github.com/manual/gh_api).

## Desktop PR navigation

Desktop 要求用户输入正整数 PR number，并在 submit 后调用 `github/pullRequest`。请求绑定已由 Core normalized 的 `nameWithOwner`，Host 不构造 endpoint。错误保留 repository identity 与 number，只在用户点击 Retry 时重复同一请求。

UI 展示 state/draft、author、base/head、timestamps、merge commit、body 和 Core ordered original commits。commit identity、login、OID 与 parent count 均来自协议，Host 不重排、不补全也不声称 partial sequence。本阶段不调用 `github/pullRequestCommitDiff`，original commit 的文件与行级远程 diff 属于后续 Task。
