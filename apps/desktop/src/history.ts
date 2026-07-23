import type { CommitGraphPage, HistoryParams } from "@gitnova/protocol";
import { coreResult, requestCore } from "./core";

export const HISTORY_PAGE_SIZE = 30;

export async function getCommitGraph(cursor?: string): Promise<CommitGraphPage> {
  const params: HistoryParams = cursor
    ? { limit: HISTORY_PAGE_SIZE, cursor }
    : { limit: HISTORY_PAGE_SIZE };
  return coreResult(await requestCore<CommitGraphPage>("repository/graph", params));
}
