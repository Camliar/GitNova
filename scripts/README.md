# Scripts

`check-visualstudio.mjs` 在所有开发平台验证 .NET transport/framing、安全进程启动和协议版本；Windows CI 另外构建完整 VisualStudio.Extensibility 工程。

- `generate-protocol.mjs`：从协议 JSON Schema 生成或检查 TypeScript 类型。
- `check-quality.mjs`：检查 Desktop 无直连网络/调试输出、Core stderr 固定化和 Tauri CSP 本地优先边界。
- `check-delivery.mjs`：检查 bundle、Windows 多尺寸图标、Core sidecar、CI 和 release 权限边界。
- `prepare-sidecar.mjs`：把 release Core 复制到 Tauri target-qualified external binary 位置。
- `check-release-tag.mjs`：阻止 tag 与 Desktop 版本不一致的发布。
- `check-idea.mjs`：使用纯 JDK 编译/运行 JetBrains framing tests，并检查 launcher、协议版本与 plugin registration 边界。

脚本必须可复现、非交互且有文档，不得承载业务逻辑。
