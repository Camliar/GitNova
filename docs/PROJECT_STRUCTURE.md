# Project Structure

```text
GitNova/
├── apps/
│   ├── desktop/       # Tauri 2 + React Host
│   ├── vscode/        # VS Code Host
│   ├── idea/          # JetBrains Host
│   └── visualstudio/  # Visual Studio Host
├── crates/
│   ├── gitnova-core/      # 独立 Core 二进制
│   └── gitnova-protocol/  # Rust 协议类型
├── packages/protocol/ # 生成的 TypeScript 协议类型
├── sdk/protocol/      # 版本化 JSON Schema 事实源
├── docs/              # 产品和工程活文档
├── tasks/             # Task 规范、模板与记录
├── adr/               # 架构决策记录
├── scripts/           # 仓库维护脚本
├── assets/
│   ├── logo/          # 主 Logo
│   ├── icons/         # 应用图标
│   └── brand/         # 品牌令牌和预览
└── .github/           # 协作模板与占位 CI
```

根目录的 Cargo 与 pnpm 清单定义 Monorepo 边界。Core 契约见[协议文档](PROTOCOL.md)，Host/Core 依赖规则见[架构](ARCHITECTURE.md)，贡献和新增目录规则见[编码规范](CODING_STANDARD.md)及[贡献指南](../CONTRIBUTING.md)。
