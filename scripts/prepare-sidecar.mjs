import { copyFileSync, mkdirSync } from "node:fs";
import { platform } from "node:os";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";

const explicitTarget = process.argv[2];
const rustc = spawnSync("rustc", ["-vV"], { encoding: "utf8" });
if (rustc.status !== 0) throw new Error("rustc -vV failed");
const host = /^host: (.+)$/m.exec(rustc.stdout)?.[1];
const target = explicitTarget || host;
if (!target) throw new Error("Rust host target is unavailable");

const suffix = platform() === "win32" || target.includes("windows") ? ".exe" : "";
const source = resolve("target", explicitTarget ? target : "", "release", `gitnova-core${suffix}`);
const directory = resolve("apps/desktop/src-tauri/bin");
const destination = resolve(directory, `gitnova-core-${target}${suffix}`);
mkdirSync(directory, { recursive: true });
copyFileSync(source, destination);
console.log(`Prepared Core sidecar for ${target}`);
