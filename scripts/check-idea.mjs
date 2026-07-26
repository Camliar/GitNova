import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";

const root = "apps/idea";
const transport = `${root}/src/main/java/dev/gitnova/idea/transport`;
const output = mkdtempSync(join(tmpdir(), "gitnova-idea-check-"));
try {
  const compile = spawnSync("javac", ["-encoding", "UTF-8", "-d", output, `${transport}/Framing.java`, `${transport}/CoreProtocolClient.java`, `${root}/tools/FramingSelfTest.java`], { stdio: "inherit" });
  if (compile.status !== 0) process.exit(compile.status ?? 1);
  const run = spawnSync("java", ["-cp", output, "dev.gitnova.idea.transport.FramingSelfTest"], { stdio: "inherit" });
  if (run.status !== 0) process.exit(run.status ?? 1);
  const client = readFileSync(`${transport}/CoreProtocolClient.java`, "utf8");
  const plugin = readFileSync(`${root}/src/main/resources/META-INF/plugin.xml`, "utf8");
  const build = readFileSync(`${root}/build.gradle.kts`, "utf8");
  const ci = readFileSync(".github/workflows/ci.yml", "utf8");
  const schema = JSON.parse(readFileSync("sdk/protocol/gitnova-protocol.schema.json", "utf8"));
  const service = readFileSync(`${root}/src/main/java/dev/gitnova/idea/CoreService.java`, "utf8");
  if (client.includes("Runtime.getRuntime().exec") || client.includes("sh -c") || client.includes("cmd /c")) throw new Error("JetBrains Host must not launch through a shell");
  if (!client.includes("Redirect.DISCARD")) throw new Error("Core stderr must not enter the IDE UI");
  if (!plugin.includes("dev.gitnova.idea.InspectPullRequest") || !build.includes("org.jetbrains.intellij.platform")) throw new Error("JetBrains plugin registration is incomplete");
  if (!build.includes("JavaLanguageVersion.of(25)") || !ci.includes("java-version: 25")) throw new Error("IntelliJ IDEA 2026.2 builds must use Java 25 in Gradle and CI");
  if (!service.includes(`PROTOCOL_VERSION = "${schema.properties.protocolVersion.const}"`)) throw new Error("JetBrains protocol version is out of date");
  console.log("JetBrains Host static checks passed");
} finally {
  rmSync(output, { recursive: true, force: true });
}
