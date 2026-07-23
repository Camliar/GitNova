# Repository Discovery

## Core ownership

仓库发现和打开是 `gitnova-core` 的领域能力。Host 只提交用户选择的路径并呈现 `RepositoryDescriptor`，不得自行查找 `.git`、运行 Git、推断 worktree 类型或改写远端环境路径。

Core 仅通过参数化进程 API 调用 System Git，不使用 shell，也不直接解析 Git 内部格式。检查命令设置 `GIT_OPTIONAL_LOCKS=0`，避免只读发现产生可选锁。

## Protocol methods

- `repository/discover`：输入 `{ "path": string }`，从现有文件或目录向上发现仓库，不修改会话状态。
- `repository/open`：执行相同发现，并把结果设置为本 Core 会话的活动仓库。重复打开同一仓库是幂等操作；一个会话不能切换到另一仓库。

两者只能在 `gitnova/initialize` 成功后调用。

## Repository descriptor

| Field | Meaning |
| --- | --- |
| `worktreeRoot` | canonical worktree 根目录；bare repository 为 `null` |
| `gitDirectory` | 当前 worktree 的 canonical Git directory |
| `commonGitDirectory` | 多 worktree 共享的 canonical Git directory |
| `kind` | `worktree`、`linkedWorktree` 或 `bare` |
| `gitVersion` | System Git 报告的版本字符串 |

当非 bare 仓库的 Git directory 与 common Git directory 不同时，Core 将其识别为 linked worktree。仓库事实来自 `git rev-parse`，而不是 Core 对 `.git` 文件结构的猜测。

## Paths and environments

输入必须是 Core 所在环境中已经存在的文件或目录。Core 解析 `.`、`..` 和符号链接并返回该环境中的绝对 canonical 路径。路径不能在 Desktop 与 WSL、Remote SSH 或 Dev Container 之间转换；Host 必须在仓库所在环境启动 Core，并把返回路径视为该环境的 opaque path。

JSON-RPC 只能无损承载 Unicode string。若平台路径无法无损转换，Core 返回 `path.unsupported_encoding`，不会用替换字符改变路径身份。

## Stable errors

| Stable code | Meaning |
| --- | --- |
| `path.invalid` | 输入为空、不存在或不是可检查路径 |
| `path.unsupported_encoding` | 路径不能被协议无损表示 |
| `repository.not_found` | 路径不属于 Git repository |
| `git.unavailable` | Core 环境中找不到 System Git |
| `git.command_failed` | System Git 无法完成只读检查 |
| `repository.unsafe_ownership` | Git 的 dubious ownership 检查拒绝仓库 |
| `repository.different_repository_open` | 当前会话已经打开另一仓库 |

Core 不自动添加 `safe.directory`，不修改 global/system Git config，也不把 Git stderr 原文返回 Host。用户应使用自己的 Git 管理流程明确解决所有权问题。

## Deliberate limits

当前能力不读取 Status、HEAD、branch、remote、history 或 diff，不初始化、clone 或修改仓库，也不保存最近仓库。后续能力必须通过独立 Task 复用活动仓库上下文。
