# GitHub Provider

GitHub access is a Core capability. Hosts explicitly request normalized domain data over JSON-RPC and never invoke GitHub CLI, handle tokens, or interpret GitHub responses.

## Repository metadata

`github/repository` accepts optional `remote` (default `origin`) and optional `nameWithOwner`. Without an override, Core reads the selected remote using System Git and accepts standard HTTPS, SSH URL, and SCP-like `git@github.com:owner/repo.git` forms. The first adapter supports only `github.com`; an SSH hostname alias can be handled by explicitly supplying `nameWithOwner`.

The explicit request runs:

```text
gh api repos/<owner>/<repo> --hostname github.com
```

Core starts `gh` directly without a shell and sets non-interactive, no-pager, no-color environment controls. GitHub CLI uses the credentials already configured in the repository environment. GitNova never calls `gh auth token`, asks the Host for a token, or returns command stderr/raw JSON.

The normalized result contains `host`, `owner`, `name`, `nameWithOwner`, `url`, `defaultBranch`, and `isPrivate`.

## Network and error semantics

This method is an explicit network action. Core does not invoke it during repository open, refresh it in the background, retry it automatically, or cache it. Stable errors distinguish invalid/missing/unsupported remote identity, unavailable `gh`, required authentication, request failure, and invalid response. Error payloads never include remote input, stderr, response bodies, or credentials.

The adapter also provides normalized PR detail, original commits, and member commit file/line diffs through [`github/pullRequest` and `github/pullRequestCommitDiff`](GITHUB_PULL_REQUESTS.md). [`github/squashTrace`](SQUASH_TRACE.md) combines these Provider facts with local Git topology while preserving inference confidence. The adapter deliberately excludes GitHub Enterprise, direct REST/GraphQL transport, login flows, arbitrary remote commit reads, PR writes, and Host visualization.

Official adapter references: [GitHub CLI `gh api`](https://cli.github.com/manual/gh_api), [GitHub CLI exit codes](https://cli.github.com/manual/gh_help_exit-codes), and [Get a repository REST response](https://docs.github.com/en/rest/repos/repos#get-a-repository).
