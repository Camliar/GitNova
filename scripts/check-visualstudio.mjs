import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";

const root = "apps/visualstudio";
const run = spawnSync("dotnet", ["run", "--project", `${root}/GitNova.VisualStudio.Transport.Tests`, "--configuration", "Release"], {
  stdio: "inherit",
  env: { ...process.env, DOTNET_ROLL_FORWARD: "Major" },
});
if (run.status !== 0) process.exit(run.status ?? 1);

const client = readFileSync(`${root}/GitNova.VisualStudio.Transport/CoreProtocolClient.cs`, "utf8");
const service = readFileSync(`${root}/GitNova.VisualStudio.Transport/GitNovaCoreService.cs`, "utf8");
const command = readFileSync(`${root}/GitNova.VisualStudio/InspectPullRequestCommand.cs`, "utf8");
const extension = readFileSync(`${root}/GitNova.VisualStudio/GitNovaExtension.cs`, "utf8");
const project = readFileSync(`${root}/GitNova.VisualStudio/GitNova.VisualStudio.csproj`, "utf8");
const schema = JSON.parse(readFileSync("sdk/protocol/gitnova-protocol.schema.json", "utf8"));
if (client.includes("cmd.exe") || client.includes("powershell") || client.includes("UseShellExecute = true")) throw new Error("Visual Studio Host must not launch through a shell");
if (!client.includes("RedirectStandardError = true") || !client.includes("ReadToEndAsync")) throw new Error("Core stderr must be drained away from protocol/UI");
if (!extension.includes("class GitNovaExtension : Extension") || !project.includes("Microsoft.VisualStudio.Extensibility.Sdk")) throw new Error("VisualStudio.Extensibility registration is incomplete");
if (!command.includes("github/squashTrace") && !service.includes("github/squashTrace")) throw new Error("Squash Trace flow is missing");
if (!command.includes("InputPromptOptions") || !command.includes("GetActiveProjectAsync")) throw new Error("Explicit PR/commit input or project environment binding is missing");
if (!service.includes(`ProtocolVersion = "${schema.properties.protocolVersion.const}"`)) throw new Error("Visual Studio protocol version is out of date");
console.log("Visual Studio Host static checks passed");
