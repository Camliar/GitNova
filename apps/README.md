# Hosts

本目录保留 GitNova 的 Host 应用。Desktop、VS Code、JetBrains 与 Visual Studio 都是交互和平台适配层，不承载业务逻辑；它们统一通过 JSON-RPC/stdio 调用独立的 `gitnova-core`。参见 [`docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md)。

