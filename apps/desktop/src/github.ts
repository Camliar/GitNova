import type { GitHubCommitFileDiff, GitHubCommitSquashTrace, GitHubCommitSquashTraceParams, GitHubPullRequest, GitHubPullRequestCommitDiff, GitHubPullRequestCommitDiffParams, GitHubPullRequestCommitFileDiffParams, GitHubPullRequestCommitFiles, GitHubPullRequestCommitFilesParams, GitHubPullRequestParams, GitHubRepository, GitHubSquashTrace } from "@gitnova/protocol";
import { coreResult, requestCore } from "./core";

export async function getGitHubRepository(): Promise<GitHubRepository> {
  return coreResult(await requestCore<GitHubRepository>("github/repository", {}));
}

export async function getGitHubPullRequestCommitDiff(number: number, oid: string, nameWithOwner: string): Promise<GitHubPullRequestCommitDiff> {
  const params: GitHubPullRequestCommitDiffParams = { number, oid, nameWithOwner };
  return coreResult(await requestCore<GitHubPullRequestCommitDiff>("github/pullRequestCommitDiff", params));
}

export async function getGitHubPullRequest(number: number, nameWithOwner: string): Promise<GitHubPullRequest> {
  const params: GitHubPullRequestParams = { number, nameWithOwner };
  return coreResult(await requestCore<GitHubPullRequest>("github/pullRequest", params));
}

export async function getGitHubSquashTrace(number: number, nameWithOwner: string): Promise<GitHubSquashTrace> {
  const params: GitHubPullRequestParams = { number, nameWithOwner };
  return coreResult(await requestCore<GitHubSquashTrace>("github/squashTrace", params));
}

export async function getGitHubCommitSquashTrace(oid: string): Promise<GitHubCommitSquashTrace> {
  const params: GitHubCommitSquashTraceParams = { oid };
  return coreResult(await requestCore<GitHubCommitSquashTrace>("github/commitSquashTrace", params));
}

export async function getGitHubPullRequestCommitFiles(number: number, oid: string, nameWithOwner: string): Promise<GitHubPullRequestCommitFiles> {
  const params: GitHubPullRequestCommitFilesParams = { number, oid, nameWithOwner };
  return coreResult(await requestCore<GitHubPullRequestCommitFiles>("github/pullRequestCommitFiles", params));
}

export async function getGitHubPullRequestCommitFileDiff(number: number, oid: string, path: string, nameWithOwner: string): Promise<GitHubCommitFileDiff> {
  const params: GitHubPullRequestCommitFileDiffParams = { number, oid, path, nameWithOwner };
  return coreResult(await requestCore<GitHubCommitFileDiff>("github/pullRequestCommitFileDiff", params));
}
