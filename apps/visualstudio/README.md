# GitNova Visual Studio Host

Visual Studio 2022 17.14+ 的进程外 `VisualStudio.Extensibility` Host。它只管理 Core 生命周期、把 Solution 所在目录发送给 Core，并通过显式输入提示展示 GitHub PR original commits、所选 commit 的文件/行级 diff 与 Squash Trace。

## 开发

```powershell
dotnet build GitNova.VisualStudio.slnx
dotnet run --project GitNova.VisualStudio.Transport.Tests
```

完整扩展/VSIX 构建需要 Windows、Visual Studio Extension Development workload 和 .NET 8 SDK；跨平台环境可构建和运行 transport tests。Core 默认从 `PATH` 启动 `gitnova-core.exe`，也可设置绝对路径环境变量 `GITNOVA_CORE_PATH`。

Host 不调用 Git、`gh` 或 HTTP，也不推断 squash 关系。关闭扩展时会先发送 `gitnova/shutdown` 和 `exit`，超时后才终止子进程。Core stderr 被隔离，不进入 JSON-RPC 或 IDE 展示。

参考：Microsoft 的 [VisualStudio.Extensibility overview](https://learn.microsoft.com/visualstudio/extensibility/visualstudio.extensibility/visualstudio-extensibility)、[command](https://learn.microsoft.com/visualstudio/extensibility/visualstudio.extensibility/command/command) 与 [user prompt](https://learn.microsoft.com/visualstudio/extensibility/visualstudio.extensibility/user-prompt/user-prompts) 文档。
