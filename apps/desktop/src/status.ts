import type { WorkingTreeStatus } from "@gitnova/protocol";
import { coreResult, requestCore } from "./core";

export async function getWorkingTreeStatus(): Promise<WorkingTreeStatus> {
  return coreResult(await requestCore<WorkingTreeStatus>("repository/status", null));
}
