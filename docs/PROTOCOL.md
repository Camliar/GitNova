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

`gitnova/initialize` 参数包含 `clientInfo`、`protocolVersion` 和 Host capabilities。结果包含 `coreInfo`、协商后的协议版本和 Core capabilities。初始协议版本为 `1.0`，当前版本为 `1.6`；主版本不同即不兼容，次版本能力通过 capability 字段协商。

Core 当前声明 `cancellation`、`repositoryDiscovery`、`workingTreeStatus`、`structuredFileDiff`、`paginatedCommitHistory`、`structuredCommitDiff` 和 `repositoryReferences` capability。仓库方法及路径语义见[仓库发现](REPOSITORIES.md)，状态契约见[工作区状态](STATUS.md)，工作区 diff 契约见[结构化文件 Diff](DIFF.md)，历史契约见[分页 Commit 历史](HISTORY.md)，commit-parent diff 契约见[结构化 Commit Diff](COMMIT_DIFF.md)，refs 契约见[Repository References](REFERENCES.md)。

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
| `-32800` | `request.cancelled` | 请求已取消 |

## Cancellation and timeouts

Host 为请求设置适合用户操作的超时。超时或用户取消时，Host 发送 `$/cancelRequest` notification，参数为原 request id。Core 的通用 cancellation registry 保持 id 类型，并为已取消的可取消工作返回 `request.cancelled`。具体长任务必须在各自 Task 中定义安全取消点；TASK-0002 不创建占位业务 method。

## Schema and generated types

协议事实源位于 [`sdk/protocol/gitnova-protocol.schema.json`](../sdk/protocol/gitnova-protocol.schema.json)。TypeScript 生成类型位于 [`packages/protocol/src/generated.ts`](../packages/protocol/src/generated.ts)，通过 `npm run generate:protocol` 生成并使用 `npm run check:protocol` 检查是否过期。

协议变更必须同步 Schema、Rust 类型、生成类型、契约测试和本文档；破坏性变更需要 ADR。

## Repository environment

Core 必须与仓库运行在同一环境。本地 Desktop 直接启动本机 Core；后续 WSL、Remote SSH 与 Dev Container Host 适配必须在对应远端环境定位并启动 Core，而不是把仓库数据复制回本机 Core。Windows 使用进程句柄/Job Object 等 Host 机制确保清理，macOS/Linux 使用子进程组或等效机制；具体 Host 监管在 Host Task 实现。
