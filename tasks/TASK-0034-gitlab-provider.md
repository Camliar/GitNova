# TASK-0034: GitLab Provider

- **Status:** In Progress
- **Priority:** P1
- **Owner:** Codex
- **Branch:** `feature/034-gitlab-provider`
- **Dependencies:** TASK-0033 (`fd75ae6`)

## Goal

在 Core 内增加 GitLab.com 与 Self-Managed Provider，使相同的 original commits、per-commit 文件/行级 diff 和 Squash Trace 价值可用于 Merge Request。

## Scope

- 通过非交互 `glab api` 接入 GitLab REST API，并依据仓库 remote 选择明确 hostname。
- 解析 GitLab HTTPS/SSH/scp-style remote，支持多级 namespace 与显式 project path override。
- 新增 `gitlab/repository`、`gitlab/mergeRequest`、`gitlab/mergeRequestCommitDiff`、`gitlab/squashTrace` 协议及 capability。
- Merge Request original commits、指定 commit diff、`squash_commit_sha`/`merge_commit_sha` 关系与本地 commit parents 证据。
- 响应/commit/file 上限、分页完整性、错误稳定码、协议生成物和 contract/unit tests。

## Non-goals

- Host UI、GitLab mutation、OAuth UI 或 GitNova 中心代理。
- GitHub Provider 重写或泛化全部现有协议类型。
- AI Assist。

## Deliverables

- [ ] GitLab Provider and remote/project identity
- [ ] Merge Request original commits, commit diff and Squash Trace
- [ ] protocol/capability, errors, tests and documentation

## Review Checklist

- [ ] 仅 Core 调用 `git`/`glab`，无 shell 与交互式认证。
- [ ] Self-Managed hostname 来自已验证 remote，不允许 endpoint/argument 注入。
- [ ] original commit membership 先验证，diff 截断不会伪装成完整结果。
- [ ] protocol generation、Rust tests、fmt 与 clippy 通过。

## Done Definition

- [ ] 自主 Review 无阻塞项，状态 Done，提交推送并快进合并 main。
