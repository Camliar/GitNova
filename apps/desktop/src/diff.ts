import type { DiffParams, DiffScope, FileDiff } from "@gitnova/protocol";
import { coreResult, requestCore } from "./core";

export async function getFileDiff(path: string, scope: DiffScope): Promise<FileDiff> {
  const params: DiffParams = { path, scope, contextLines: 3 };
  return coreResult(await requestCore<FileDiff>("repository/diff", params));
}
