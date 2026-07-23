# TASK-0003: Repository Discovery and Open

- **Status:** Draft
- **Priority:** P0
- **Owner:** Unassigned
- **Branch:** `feature/003-repository-discovery`
- **Estimate:** 2–3 days
- **Dependencies:** TASK-0002 approved Core Contract baseline (`1c91856`)

## Goal

通过 `gitnova-core` 为 Host 提供可验证的本地 Git 仓库发现与打开能力：用户给定一个本地路径后，Core 使用 System Git 识别普通 worktree、linked worktree 或 bare repository，返回稳定的仓库描述并在当前 Core 会话中建立仓库上下文。

## Context

[Phase 2](../docs/ROADMAP.md#phase-2--local-git-foundation) 必须将所有 Git 能力保持在 Core 内，并且只调用用户环境中的 System Git。仓库发现是后续 Status、Diff、History 和 Squash Trace 的共同入口；如果路径标识、worktree 语义或错误模型在 Host 中重复，多 Host 行为会立即分叉。

## Scope

- 在 Core 内创建 System Git 命令适配边界；使用参数化进程 API，不经 shell 拼接命令。
- 在 initialize 成功后提供 `repository/discover` request，从给定文件或目录向上发现所属仓库。
- 提供 `repository/open` request，验证并在当前 Core 会话中设置唯一活动仓库上下文；重复打开同一仓库必须幂等。
- 识别并返回 normal worktree、linked worktree 与 bare repository，包括 canonical worktree root（bare 时为 `null`）、Git directory、common Git directory 和 repository kind。
- 通过 System Git 查询 Git 版本与 repository facts，不自行解析 `.git` 文件或内部对象格式。
- 定义路径规范化和身份规则：返回所在环境的绝对路径，解析 `.`/`..` 和符号链接，但不做平台间路径转换。
- 对无法用 JSON string 无损表示的本地路径返回稳定的 unsupported-path-encoding 错误，不静默替换字节。
- 定义稳定错误：路径不存在/无效、未发现仓库、System Git 不可用、Git 命令失败、不安全仓库所有权，以及会话已打开其他仓库。
- 保留 Git `safe.directory` 安全边界；Core 不自动修改用户的 global/system Git config，不绕过 dubious ownership 检查。
- 扩展 JSON Schema、Rust 协议类型、生成的 TypeScript 类型和 Core capability，并保持三者一致。
- 为命令参数、路径规范化、仓库类型、错误映射、幂等 open 和 JSON-RPC 端到端路径提供单元/集成/契约测试。
- 在 Windows、macOS 和 Linux 可表达的范围内保持路径和进程行为无单平台假设。

## Protocol Baseline

- `repository/discover` 输入为 `{ "path": string }`，成功时返回 `RepositoryDescriptor`，但不改变会话状态。
- `repository/open` 输入为 `{ "path": string }`，成功时返回同一 `RepositoryDescriptor` 并设置活动仓库。
- `RepositoryDescriptor` 至少包含 `worktreeRoot: string | null`、`gitDirectory: string`、`commonGitDirectory: string`、`kind: "worktree" | "linkedWorktree" | "bare"` 和 `gitVersion: string`。
- 路径仅在产生结果的 Core 所在环境内有意义；Host 不得假定它是本机路径或自行改写 WSL/Remote/Container 路径。
- Core 不接受来自 Host 的 Git executable 任意路径覆盖；可执行文件定位策略由后续配置 Task 扩展。

## Non-goals

- 列出“最近仓库”、扫描整个磁盘、文件选择器或 Desktop UI。
- Git Status、staging、Diff、Commit、Branch、Tag、Remote、History 或 Graph。
- 初始化新仓库、clone、fetch、pull、push 或任何写操作。
- GitHub Provider、身份验证、PR、original commits 或 Squash Trace。
- SQLite、最近记录、设置、缓存或索引。
- 自动修复 `safe.directory`、Git config、文件权限或损坏仓库。
- 同一 Core 会话同时打开多个仓库，或在不重启 Core 的情况下切换到另一仓库。

## Deliverables

- [ ] Core System Git 命令适配器与可测试边界
- [ ] `repository/discover` 与 `repository/open` JSON-RPC methods
- [ ] RepositoryDescriptor、capability 与稳定错误契约
- [ ] JSON Schema、Rust 类型与生成 TypeScript 类型同步
- [ ] normal、linked worktree 与 bare repository 测试覆盖
- [ ] System Git 不可用、非仓库、无效路径与安全边界错误覆盖
- [ ] 仓库发现、路径语义、System Git 与会话状态文档
- [ ] README、协议、项目结构、功能清单和路线图状态同步

## Review Checklist

- [ ] 仓库发现和打开逻辑仅存在于 Core，未引入 Host 业务实现。
- [ ] 只调用 System Git，未实现或直接解析 Git 内部格式。
- [ ] Git 命令不经 shell，路径只作为独立参数传递，无命令注入路径。
- [ ] normal、linked worktree、bare、子目录发现和符号链接路径行为已验证。
- [ ] `safe.directory` 与用户 Git config 未被绕过或修改。
- [ ] discover 无会话副作用，open 对同一仓库幂等，对其他仓库稳定拒绝。
- [ ] 错误不泄露仓库内容、凭据或无界 stderr，并提供可操作 stable code。
- [ ] Rust、JSON Schema 与 TypeScript 类型一致，无手写 Host 并行类型。
- [ ] 未实现 Status、Diff、History、GitHub、PR、Squash Trace、SQLite 或 UI。
- [ ] `cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace` 与 `npm run check` 通过。
- [ ] Markdown 链接、JSON/Schema、生成结果和目录说明已验证。

## Done Definition

- [ ] Deliverables 全部完成。
- [ ] Review Checklist 全部通过且无未声明范围扩张。
- [ ] 非作者 Reviewer 批准并合并对应 PR。
- [ ] Task 状态更新为 Done。

## Notes

- 该 Task 只建立只读仓库身份与会话上下文，不因“顺便”读取 Status、HEAD、remote 或 branch。
- 若 System Git 在目标平台无法稳定区分 linked worktree，先在本 Task 记录可移植证据并调整契约，不在 Host 中添加推断。

## References

[Roadmap](../docs/ROADMAP.md#phase-2--local-git-foundation) · [Feature List](../docs/FEATURE_LIST.md#mvp-必备后续-task) · [Architecture](../docs/ARCHITECTURE.md) · [Protocol](../docs/PROTOCOL.md) · [ADR-0001](../adr/ADR-0001-Architecture.md) · [Non-functional Requirements](../docs/NON_FUNCTIONAL.md)
