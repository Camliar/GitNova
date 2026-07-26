# GitHub Provider

GitHub access is a Core capability. Hosts explicitly request normalized domain data over JSON-RPC and never invoke GitHub CLI, handle tokens, or interpret GitHub responses.

## Repository metadata

`github/repository` accepts optional `remote` (default `origin`) and optional `nameWithOwner`. Without an override, Core reads the selected remote using System Git and accepts standard HTTPS, SSH URL, and SCP-like `git@github.com:owner/repo.git` forms. The adapter supports only `github.com`. For an SSH form using an alias such as `git@github-work:owner/repo.git`, Core runs bounded, non-interactive `ssh -G -- github-work` in the repository environment and accepts the identity only when its resolved `hostname` is exactly `github.com`. It never connects to SSH, returns configuration, or reads keys into the protocol.

The explicit request runs:

```text
gh api repos/<owner>/<repo> --hostname github.com
```

Core starts `gh` directly without a shell and sets non-interactive, no-pager, no-color environment controls. GitHub CLI uses the credentials already configured in the repository environment. GitNova never calls `gh auth token`, asks the Host for a token, or returns command stderr/raw JSON.

The normalized result contains `host`, `owner`, `name`, `nameWithOwner`, `url`, `defaultBranch`, and `isPrivate`.

## Network and error semantics

This method is an explicit network action. Core does not invoke it during repository open, refresh it in the background, retry it automatically, or cache it. Stable errors distinguish invalid/missing/unsupported remote identity, unavailable `gh`, required authentication, request failure, and invalid response. Error payloads never include remote input, stderr, response bodies, or credentials.

The adapter also provides normalized PR detail, original commits, and member commit file/line diffs through [`github/pullRequest`, `github/pullRequestCommitDiff` and the lazy original-commit methods](GITHUB_PULL_REQUESTS.md). [`github/squashTrace` and `github/commitSquashTrace`](SQUASH_TRACE.md) combine these Provider facts with local Git topology while preserving inference confidence. The adapter deliberately excludes GitHub Enterprise, direct REST/GraphQL transport, login flows, arbitrary remote commit reads, PR writes, and Host-side inference.

Official adapter references: [GitHub CLI `gh api`](https://cli.github.com/manual/gh_api), [GitHub CLI exit codes](https://cli.github.com/manual/gh_help_exit-codes), and [Get a repository REST response](https://docs.github.com/en/rest/repos/repos#get-a-repository).

## Desktop consent boundary

Desktop 在 repository open 后不调用任何 GitHub method。只有用户点击 Connect GitHub 才请求 `github/repository`；UI 明确说明 Core 使用仓库环境中现有的 `gh` 配置。Host 不显示 login/token UI，不读取 credentials，也不从 URL 自行解析 repository identity。

成功后 Desktop 仅显示 normalized fields，并把 `nameWithOwner` 固定传给后续显式 PR 请求。provider URL 当前仅作为文本显示，不触发 Host 网络导航。All Commits 的 **Check Squash Trace** 也是独立的显式网络动作；普通 commit 的本地详情不依赖它。repository reopen 会卸载 Host 状态并使迟到请求失效；不存在 background refresh 或自动 retry。Core 仅为用户选中的 original commit 保留一个有界内存 diff cache，用于后续按文件读取。
