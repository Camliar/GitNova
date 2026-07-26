import type { CommitDiff, CommitDiffParams, CommitFileDiffParams, CommitFiles, CommitFilesParams, FileDiff } from "@gitnova/protocol";
import { coreResult, requestCore } from "./core";

export async function getCommitDiff(oid: string, parentOid?: string): Promise<CommitDiff> {
  const params: CommitDiffParams = parentOid
    ? { oid, parentOid, contextLines: 3 }
    : { oid, contextLines: 3 };
  return coreResult(await requestCore<CommitDiff>("repository/commitDiff", params));
}

export async function getCommitFiles(oid: string, parentOid?: string): Promise<CommitFiles> {
  const params: CommitFilesParams = parentOid ? { oid, parentOid } : { oid };
  return coreResult(await requestCore<CommitFiles>("repository/commitFiles", params));
}

export async function getCommitFileDiff(oid: string, path: string, parentOid?: string): Promise<FileDiff> {
  const params: CommitFileDiffParams = parentOid
    ? { oid, parentOid, path, contextLines: 3 }
    : { oid, path, contextLines: 3 };
  return coreResult(await requestCore<FileDiff>("repository/commitFileDiff", params));
}
