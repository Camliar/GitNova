import { readFile, readdir } from "node:fs/promises";
import { extname, join } from "node:path";

const failures = [];
async function productionFiles(directory, extensions) {
  const files = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) files.push(...await productionFiles(path, extensions));
    else if (extensions.includes(extname(entry.name)) && !entry.name.includes(".test.")) files.push(path);
  }
  return files;
}
const desktopFiles = await productionFiles("apps/desktop/src", [".ts", ".tsx"]);
for (const path of desktopFiles) {
  const source = await readFile(path, "utf8");
  for (const [label, pattern] of [["direct network API", /\b(?:fetch|XMLHttpRequest|WebSocket|EventSource)\s*\(/], ["debug console output", /console\.(?:log|debug|info)\s*\(/]]) {
    if (pattern.test(source)) failures.push(`${path}: ${label}`);
  }
}
if ((await readFile("crates/gitnova-core/src/main.rs", "utf8")).includes("{error}")) failures.push("Core stderr exposes raw transport error detail");
const tauri = JSON.parse(await readFile("apps/desktop/src-tauri/tauri.conf.json", "utf8"));
const csp = tauri.app?.security?.csp;
if (typeof csp !== "string" || !csp.includes("default-src 'self'") || /https?:/.test(csp)) failures.push("Tauri CSP allows remote HTTP origins");
if (failures.length) { failures.forEach((failure) => process.stderr.write(`quality: ${failure}\n`)); process.exit(1); }
process.stdout.write(`quality: checked ${desktopFiles.length} Desktop files, Core stderr, and Tauri CSP\n`);
