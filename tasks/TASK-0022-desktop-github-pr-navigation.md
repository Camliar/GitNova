# TASK-0022: Desktop GitHub PR Navigation

- **Status:** In Progress
- **Priority:** P0
- **Owner:** Codex
- **Branch:** `feature/022-desktop-github-pr-navigation`
- **Dependencies:** TASK-0021 Desktop commit diff (`861fe09`), GitHub PR protocol `1.11`

## Goal

通过用户明确触发的 Core GitHub 请求，在 Desktop 识别当前 GitHub repository，并按 PR number 展示 pull request metadata 与完整 original commit sequence。

## Scope

- repository 打开后显示 Connect GitHub，但不自动访问网络。
- 明确触发 `github/repository`，使用 Core 的默认 `origin` 解析与现有 `gh` credentials。
- 展示 normalized repository identity、visibility、default branch 与 URL（仅文本，不由 Host 发起导航）。
- 接受正整数 PR number，提交时调用 `github/pullRequest` 并固定使用已解析 `nameWithOwner`。
- 展示 PR state/draft、author、base/head、timestamps、merge commit OID、body 与 ordered original commits。
- original commit 展示完整 Git identity、可选 login、summary、OID 和 parent count；不读取 diff。
- 网络请求显式、无自动 retry/cache/background refresh；loading 禁用重复操作。
- stable auth/provider errors 不泄漏 raw stderr/token/response；保留 repository identity 和 PR number 以便 Retry。
- repository reopen 通过组件 identity 重置 GitHub/PR state，迟到响应不得回写。
- 测试显式触发、参数、validation、merged/squashed PR、original order、empty/error/retry/stale。

## Non-goals

- login/token UI、GitHub Enterprise、direct REST/GraphQL、PR 搜索/list/create/edit。
- PR original commit file/line diff、Squash Trace relationship 或本地 squash commit 呈现。
- Host 调用 gh、解析 remote URL/provider JSON 或处理 credentials。

## Deliverables

- [ ] typed GitHub repository/PR client boundary
- [ ] explicit network consent and PR-number navigation
- [ ] normalized PR and original commit presentation
- [ ] frontend tests and documentation

## Review Checklist

- [ ] 无 repository open 后自动网络请求；每次 provider action 都由用户触发。
- [ ] Host 不执行 gh、不处理 token、不解析 remote/provider payload。
- [ ] PR commits 保持 Core order，不宣称 partial result。
- [ ] errors sanitized，retry 保留安全 selection，stale response 不回写。
- [ ] 未提前实现 remote commit diff 或 Squash Trace UI。
- [ ] Rust fmt/Clippy/tests、frontend checks 与 Tauri build 通过。

## Done Definition

- [ ] Deliverables 和 Review Checklist 完成。
- [ ] 自主 Review 无阻塞项。
- [ ] 状态更新 Done，提交、推送并快进合并至 `main`。

## References

[GitHub Provider](../docs/GITHUB_PROVIDER.md) · [GitHub Pull Requests](../docs/GITHUB_PULL_REQUESTS.md) · [Desktop Core Transport](../docs/DESKTOP_CORE_TRANSPORT.md)
