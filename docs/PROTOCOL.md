# Core Protocol

## Transport

每个 Host 启动一个仓库环境内的 `gitnova-core` 子进程。Host 与 Core 使用 JSON-RPC 2.0，通过 stdin/stdout 交换消息；Core 不监听端口，也不注册 daemon。stderr 只用于不含敏感信息的诊断。

每条 UTF-8 JSON 消息使用兼容 LSP 的 framing：

```text
Content-Length: <UTF-8 byte length>\r\n
\r\n
<JSON body>
```

Core 接受 header 名大小写差异，要求恰好一个有效 `Content-Length`，并拒绝超过 16 MiB 的消息。stdout 只能包含这种 framing 的 JSON-RPC 消息。

## Lifecycle

```text
spawn → gitnova/initialize → requests and notifications → gitnova/shutdown → exit
```

- 单次进程会话只能成功初始化一次。
- initialize 前的其他请求返回 `core.not_initialized`。
- `gitnova/shutdown` 是 request；成功响应后 Core 只接受 `exit` notification。
- shutdown 后收到 `exit` 时进程以 0 退出；未 shutdown 直接 `exit` 时以非零状态退出。
- stdin 正常关闭时 Core 结束会话。Host 崩溃或退出时必须关闭 stdin 并确保子进程终止。

## Initialize

`gitnova/initialize` 参数包含 `clientInfo`、`protocolVersion` 和 Host capabilities。结果包含 `coreInfo`、协商后的协议版本和 Core capabilities。初始协议版本为 `1.0`，当前版本为 `1.19`；主版本不同即不兼容，次版本能力通过 capability 字段协商。

Core 当前另声明 `repositoryMutations`、optional `lazyCommitDiff`、optional `historySquashTrace`、optional `repositorySync` 与 optional `remoteBranchCheckout` capability；完整 capability 由 Schema 定义。支持 `lazyCommitDiff` 的 Host 应先调用 `repository/commitFiles` 获取有界文件 metadata，只在用户选择文件后调用 `repository/commitFileDiff`。支持 `historySquashTrace` 的 Host 可在用户显式触发后调用 `github/commitSquashTrace`，再通过 `github/pullRequestCommitFiles` 与 `github/pullRequestCommitFileDiff` 按需查看 original commit。支持 `repositorySync` 的 Host 可显式调用 `repository/fetch`，并在二次确认后调用绑定 branch/HEAD 的 `repository/pull` 与 `repository/push`。支持 `remoteBranchCheckout` 的 Host 可把 Core 返回的完整 remote ref 与确认时 HEAD OID 原样传给 `repository/checkoutRemoteBranch`；remote/local branch 名的解析仍由 Core 完成。仓库方法及路径语义见[仓库发现](REPOSITORIES.md)，写操作安全契约见[Repository Mutations](MUTATIONS.md)，其余读模型与 Provider 文档保持各自事实来源。

请求 id 可以是 JSON string 或 integer，Core 必须在响应中保持其类型和值。

## Errors

JSON-RPC error 使用标准数值 `code`，同时在 `data.stableCode` 提供稳定、可供 Host 分支处理的 GitNova 错误码。`data.retryable` 表示相同意图是否可能在状态变化后重试。错误不得包含 token、凭据、仓库内容或 diff。

| JSON-RPC code | Stable code | 含义 |
| --- | --- | --- |
| `-32700` | `protocol.parse_error` | JSON 无法解析 |
| `-32600` | `protocol.invalid_request` | JSON-RPC envelope 无效 |
| `-32601` | `protocol.method_not_found` | method 不存在 |
| `-32602` | `protocol.invalid_params` | 参数无效 |
| `-32001` | `protocol.incompatible_version` | 协议主版本不兼容 |
| `-32002` | `core.not_initialized` | Core 尚未初始化 |
| `-32003` | `core.already_initialized` | 重复初始化 |
| `-32100` | `path.invalid` / `path.unsupported_encoding` | 路径无效或无法表示 |
| `-32101` | `repository.not_found` | 未发现 Git repository |
| `-32102` | `git.unavailable` | System Git 不可用 |
| `-32103` | `git.command_failed` | Git 只读检查失败 |
| `-32104` | `repository.unsafe_ownership` | Git 拒绝不安全所有权 |
| `-32105` | `repository.different_repository_open` | 会话已打开另一仓库 |
| `-32106` | `repository.not_open` | 会话尚未打开仓库 |
| `-32107` | `repository.worktree_required` | 操作不支持 bare repository |
| `-32108` | `git.status_parse_failed` | porcelain status 无法解析 |
| `-32109` | `git.diff_parse_failed` | unified patch 无法解析 |
| `-32110` | `path.invalid_repository_relative` | 文件路径不是安全的仓库相对路径 |
| `-32111` | `history.invalid_cursor` | 历史 cursor 无效或快照不可用 |
| `-32112` | `git.commit_parse_failed` | raw commit object 无法解析 |
| `-32113` | `history.unsupported_encoding` | commit metadata/message 不是受支持的 UTF-8 |
| `-32114` | `commit.not_found` | 指定 object 不存在或不是 commit |
| `-32115` | `commit.parent_required` | merge commit 必须明确选择直接 parent |
| `-32116` | `commit.invalid_parent` | 指定 OID 不是该 commit 的直接 parent |
| `-32117` | `git.commit_diff_parse_failed` | NUL-delimited commit change list 无法解析 |
| `-32118` | `git.reference_parse_failed` | System Git reference payload 无法解析 |
| `-32119` | `reference.unsupported_encoding` | reference metadata 不是 UTF-8 |
| `-32120` | `github.invalid_remote` | remote name 无效 |
| `-32121` | `github.remote_not_found` | 未配置指定 GitHub remote |
| `-32122` | `github.unsupported_remote` | remote/override 不是受支持的 github.com identity |
| `-32123` | `github.gh_unavailable` | 仓库环境未提供 GitHub CLI |
| `-32124` | `github.authentication_required` | GitHub CLI 需要认证 |
| `-32125` | `github.request_failed` | GitHub 请求失败 |
| `-32126` | `github.response_parse_failed` | GitHub repository 响应无效 |
| `-32127` | `github.pr_commit_limit_exceeded` | PR original commits 超出可证明完整性的 REST 上限 |
| `-32128` | `github.commit_not_in_pull_request` | commit 不是指定 PR 的 original commit |
| `-32129` | `github.commit_file_limit_exceeded` | commit 文件达到 GitHub 上限，无法证明列表完整 |
| `-32130` | `commit.message_required` | commit message 为空或超过大小限制 |
| `-32131` | `commit.nothing_staged` | index 没有 staged changes |
| `-32132` | `commit.unresolved_conflicts` | index 仍有未解决冲突 |
| `-32133` | `branch.invalid_name` | local branch 名称无效 |
| `-32134` | `branch.already_exists` | local branch 已存在 |
| `-32135` | `branch.not_found` | local branch 不存在 |
| `-32136` | `branch.unborn_head` | HEAD 尚无首个 commit |
| `-32137` | `git.mutation_failed` | hook、checkout safety 或 System Git 拒绝 mutation |
| `-32138` | `gitlab.invalid_remote` | remote name 无效 |
| `-32139` | `gitlab.remote_not_found` | 未配置指定 GitLab remote |
| `-32140` | `gitlab.unsupported_remote` | remote/project identity 不受支持 |
| `-32141` | `gitlab.glab_unavailable` | 仓库环境未提供 GitLab CLI |
| `-32142` | `gitlab.authentication_required` | GitLab CLI 需要认证 |
| `-32143` | `gitlab.request_failed` | GitLab 请求失败 |
| `-32144` | `gitlab.response_parse_failed` | GitLab 响应无效 |
| `-32145` | `gitlab.mr_commit_limit_exceeded` | MR original commits 超出支持上限 |
| `-32146` | `gitlab.commit_not_in_merge_request` | commit 不是指定 MR 的 original commit |
| `-32147` | `gitlab.commit_file_limit_exceeded` | commit 文件超出支持上限 |
| `-32148` | `ai.nothing_staged` | 没有 staged changes 可供 AI 预览 |
| `-32149` | `ai.invalid_provider` | Provider/model/loopback endpoint 无效 |
| `-32150` | `ai.preview_stale` | index、Provider 或排除范围已变化 |
| `-32151` | `ai.external_confirmation_required` | 当前外部披露预览尚未确认 |
| `-32152` | `ai.credential_missing` | Core 环境缺少 Provider 凭据 |
| `-32153` | `ai.provider_unavailable` | System curl 或 Provider 不可达 |
| `-32154` | `ai.request_failed` | Provider 拒绝请求 |
| `-32155` | `ai.response_invalid` | Provider 响应不符合结构化契约 |
| `-32156` | `ai.input_limit_exceeded` | staged 输入无法在安全上限内披露 |
| `-32157` | `commit.file_limit` | commit changed-file metadata 超出安全上限 |
| `-32158` | `commit.file_diff_limit` | 单个文件的 commit patch 超出安全上限 |
| `-32159` | `github.commit_association_ambiguous` | 多个 PR 将所选 commit 报告为最终 merge commit，Core 拒绝猜测 |
| `-32160` | `sync.invalid_remote` | remote 名称无效 |
| `-32161` | `sync.remote_not_found` | 目标 remote 不存在 |
| `-32162` | `sync.branch_required` | sync 需要 attached local branch |
| `-32163` | `sync.upstream_required` | Pull 需要已配置 upstream |
| `-32164` | `sync.stale_head` | 确认后的 branch 或 HEAD 已变化 |
| `-32165` | `sync.diverged` | local/upstream 已分叉，拒绝非 fast-forward Pull |
| `-32166` | `sync.fetch_failed` | Fetch 失败 |
| `-32167` | `sync.pull_failed` | fast-forward 更新工作树失败 |
| `-32168` | `sync.push_failed` | non-force Push 被拒绝或不可达 |
| `-32169` | `branch.stale_head` | remote checkout 确认后的 current HEAD 已变化 |
| `-32800` | `request.cancelled` | 请求已取消 |

GitLab Provider 方法为 `gitlab/project`、`gitlab/mergeRequest`、`gitlab/mergeRequestCommitDiff` 和 `gitlab/squashTrace`。`pathWithNamespace` 可覆盖 remote path，但 hostname 始终来自已验证 remote；SSH Host alias 由仓库环境中的 Core 通过有界、非交互 `ssh -G` 解析，Host 不读取 SSH config。任何网络动作仍必须由 Host 显式触发。

AI Assist 提供 `ai/inputPreview` 与 `ai/generateCommitDraft`。1.14 定义其类型和 `aiAssist` capability，1.15 扩展 Claude、DeepSeek、Qwen 与 Kimi Provider；Host 仅在 capability 为 true 时调用。输入披露、预览绑定与 Provider 凭据规则见 [AI Assist Contract](AI_ASSIST.md)。

## Cancellation and timeouts

Host 为请求设置适合用户操作的超时。超时或用户取消时，Host 发送 `$/cancelRequest` notification，参数为原 request id。Core 的通用 cancellation registry 保持 id 类型，并为已取消的可取消工作返回 `request.cancelled`。具体长任务必须在各自 Task 中定义安全取消点；TASK-0002 不创建占位业务 method。

## Schema and generated types

协议事实源位于 [`sdk/protocol/gitnova-protocol.schema.json`](../sdk/protocol/gitnova-protocol.schema.json)。TypeScript 生成类型位于 [`packages/protocol/src/generated.ts`](../packages/protocol/src/generated.ts)，通过 `npm run generate:protocol` 生成并使用 `npm run check:protocol` 检查是否过期。

协议变更必须同步 Schema、Rust 类型、生成类型、契约测试和本文档；破坏性变更需要 ADR。

## Repository environment

Core 必须与仓库运行在同一环境。本地 Desktop 直接启动本机 Core；后续 WSL、Remote SSH 与 Dev Container Host 适配必须在对应远端环境定位并启动 Core，而不是把仓库数据复制回本机 Core。Windows 使用进程句柄/Job Object 等 Host 机制确保清理，macOS/Linux 使用子进程组或等效机制；具体 Host 监管在 Host Task 实现。
