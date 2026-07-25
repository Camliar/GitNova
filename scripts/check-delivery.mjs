import { readFileSync } from "node:fs";

const read = (path) => readFileSync(path, "utf8");
const config = JSON.parse(read("apps/desktop/src-tauri/tauri.conf.json"));
const bundleConfig = JSON.parse(read("apps/desktop/src-tauri/tauri.bundle.conf.json"));
const ci = read(".github/workflows/ci.yml");
const release = read(".github/workflows/release.yml");

const failures = [];
if (config.bundle?.active !== true) failures.push("Tauri bundle must be active");
if (!bundleConfig.bundle?.externalBin?.includes("bin/gitnova-core")) failures.push("Desktop bundle must include the Core sidecar");
for (const runner of ["ubuntu-22.04", "windows-latest", "macos-latest"]) {
  if (!ci.includes(runner)) failures.push(`CI runner missing: ${runner}`);
  if (!release.includes(runner)) failures.push(`release runner missing: ${runner}`);
}
if (!ci.includes("contents: read") || /secrets\.[A-Z_]+/.test(ci)) failures.push("CI must be read-only and use no repository secrets");
if (!release.includes("contents: write") || !release.includes("tags:") || !release.includes("- 'v*'")) failures.push("release must be tag-gated with contents write permission");
if (failures.length) {
  console.error(failures.join("\n"));
  process.exit(1);
}
console.log("Delivery configuration is valid");
