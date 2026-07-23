import type { CommitDiff, CommitDiffParams } from "@gitnova/protocol";
import { coreResult, requestCore } from "./core";

export async function getCommitDiff(oid: string, parentOid?: string): Promise<CommitDiff> {
  const params: CommitDiffParams = parentOid
    ? { oid, parentOid, contextLines: 3 }
    : { oid, contextLines: 3 };
  return coreResult(await requestCore<CommitDiff>("repository/commitDiff", params));
}
