# GitHub Pull Requests and Original Commits

`github/pullRequest` is an explicit Core-owned network request that returns one normalized pull request and its ordered original commit list. This list remains available from GitHub after a squash merge and is the source sequence for later per-commit diff and Squash Trace Tasks.

## Request

The request requires a positive `number`. Optional `remote` and `nameWithOwner` select repository identity exactly as described in [GitHub Provider](GITHUB_PROVIDER.md). The Host never supplies credentials or constructs API endpoints.

Core requests PR detail and then calls the PR commits REST endpoint with `per_page=100`, `--paginate`, and `--slurp`. Commit pages are flattened without reordering. The normalized PR includes state, draft/author metadata, base/head refs and OIDs, lifecycle timestamps, and `mergeCommitOid`. A merged PR is reported as `merged` even though the REST `state` value is `closed`.

Each original commit includes its OID, ordered parent OIDs, Git author/committer identity, optional corresponding GitHub login, first-line summary, complete message, and GitHub URL. A missing GitHub user (for example a deleted account or unmatched email) does not remove the Git identity.

## Completeness and limits

GitHub documents a maximum of 250 commits for the PR commits endpoint. Core compares the PR detail count with the flattened list. A count above the supported limit or a capped 250-item response that cannot prove completeness returns `github.pr_commit_limit_exceeded`; any other mismatch is an invalid Provider response. GitNova never silently presents a partial original commit sequence.

Detail responses are limited to 1 MiB and commit pages to 16 MiB. Core does not return raw GitHub JSON, response headers, or command stderr. This Task does not return changed files or patches, infer a squash relationship, cache data, or update a PR.

Official references: [Get a pull request and list PR commits](https://docs.github.com/en/rest/pulls/pulls) and [GitHub CLI pagination/slurp](https://cli.github.com/manual/gh_api).
