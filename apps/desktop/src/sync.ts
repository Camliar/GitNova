import type { RepositoryFetchParams, RepositorySyncParams, RepositorySyncResult } from "@gitnova/protocol";
import { coreResult, requestCore } from "./core";

export async function fetchRepository(remote?: string): Promise<RepositorySyncResult> {
  const params: RepositoryFetchParams = remote ? { remote } : {};
  return coreResult(await requestCore<RepositorySyncResult>("repository/fetch", params));
}

export async function pullRepository(expectedBranch: string, expectedHeadOid: string): Promise<RepositorySyncResult> {
  const params: RepositorySyncParams = { expectedBranch, expectedHeadOid };
  return coreResult(await requestCore<RepositorySyncResult>("repository/pull", params));
}

export async function pushRepository(expectedBranch: string, expectedHeadOid: string): Promise<RepositorySyncResult> {
  const params: RepositorySyncParams = { expectedBranch, expectedHeadOid };
  return coreResult(await requestCore<RepositorySyncResult>("repository/push", params));
}
