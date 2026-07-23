# Coding Standard

本规范补充[贡献指南](../CONTRIBUTING.md)；自动格式化工具的结果优先于个人风格。

## 通用约定

- 文件使用 UTF-8、LF 和末尾换行，遵循 [`.editorconfig`](../.editorconfig)。
- 名称表达领域含义；避免含糊缩写、隐藏副作用和跨层捷径。
- Host 不得实现业务规则，Core 不得依赖具体 Host。
- 公共协议、持久化格式和安全边界必须有测试与文档。
- 错误应携带稳定代码和可操作上下文，不泄露凭据或仓库敏感内容。

## TypeScript / React

- 开启严格类型检查；公共边界禁止无理由使用 `any`。
- 使用函数组件和组合；副作用集中在适配层。
- UI 状态与 Core 领域状态分离，JSON-RPC 类型由协议源生成或共享。
- 文件名使用 `kebab-case`，组件和类型使用 `PascalCase`，变量使用 `camelCase`。

## Rust

- 使用 `rustfmt` 和 `clippy`，新增警告视为失败。
- 公共类型和错误必须有文档；生产路径避免 `unwrap`/`expect`。
- I/O 位于适配器边界，领域逻辑保持可测试且不依赖全局状态。
- module/function 使用 `snake_case`，type/trait 使用 `PascalCase`。

## 测试与文档

单元测试覆盖领域规则，契约测试覆盖 JSON-RPC，集成测试覆盖 System Git 边界，端到端测试只验证关键 Host 流程。实现应关联对应 Task，并同步更新[架构](ARCHITECTURE.md)、[功能清单](FEATURE_LIST.md)或 ADR。

