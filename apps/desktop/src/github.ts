import type { GitHubPullRequest, GitHubPullRequestCommitDiff, GitHubPullRequestCommitDiffParams, GitHubPullRequestParams, GitHubRepository } from "@gitnova/protocol";
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
