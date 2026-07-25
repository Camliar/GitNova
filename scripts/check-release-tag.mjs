import { readFileSync } from "node:fs";

const tag = process.argv[2] ?? "";
const version = JSON.parse(readFileSync("apps/desktop/src-tauri/tauri.conf.json", "utf8")).version;
if (tag !== `v${version}`) {
  console.error(`Release tag ${tag || "<missing>"} must equal Desktop version v${version}`);
  process.exit(1);
}
console.log(`Release tag ${tag} matches Desktop version`);
