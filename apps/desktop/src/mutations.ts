import type { BranchParams, CommitParams, CommitResult, RemoteBranchCheckoutParams, RepositoryMutationSnapshot, RepositoryReferences } from "@gitnova/protocol";
import { coreResult, requestCore } from "./core";

export async function getRepositoryReferences(): Promise<RepositoryReferences> {
  return coreResult(await requestCore<RepositoryReferences>("repository/references", null));
}

export async function commitStaged(message: string): Promise<CommitResult> {
  const params: CommitParams = { message };
  return coreResult(await requestCore<CommitResult>("repository/commit", params));
}

export async function createLocalBranch(name: string): Promise<RepositoryMutationSnapshot> {
  const params: BranchParams = { name };
  return coreResult(await requestCore<RepositoryMutationSnapshot>("repository/createBranch", params));
}

export async function switchLocalBranch(name: string): Promise<RepositoryMutationSnapshot> {
  const params: BranchParams = { name };
  return coreResult(await requestCore<RepositoryMutationSnapshot>("repository/switchBranch", params));
}

export async function checkoutRemoteBranch(fullName: string, expectedHeadOid: string): Promise<RepositoryMutationSnapshot> {
  const params: RemoteBranchCheckoutParams = { fullName, expectedHeadOid };
  return coreResult(await requestCore<RepositoryMutationSnapshot>("repository/checkoutRemoteBranch", params));
}
