# Rust crates

- `gitnova-core`：独立 Core 二进制，承载 JSON-RPC/stdio 生命周期、本地 Git 与 GitHub/GitLab Squash Trace 能力。
- `gitnova-protocol`：跨 Core 契约测试共享的版本化协议类型。

Host 只能通过版本化协议使用这些能力，不得复制 Git 或 Provider 业务逻辑。
