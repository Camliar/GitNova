import { fireEvent, render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { GitHubPanel } from "./GitHubPanel";

const github = vi.hoisted(() => ({ getGitHubRepository: vi.fn(), getGitHubPullRequest: vi.fn(), getGitHubPullRequestCommitDiff: vi.fn(), getGitHubSquashTrace: vi.fn() }));
vi.mock("./github", () => github);
const repository = { host: "github.com" as const, owner: "owner", name: "repo", nameWithOwner: "owner/repo", url: "https://github.com/owner/repo", defaultBranch: "main", isPrivate: true };
const identity = { name: "Ada", email: "ada@example.com", timestamp: "2026-01-01T00:00:00Z", login: "ada" };

describe("GitHub PR navigation", () => {
  beforeEach(() => { vi.clearAllMocks(); github.getGitHubRepository.mockResolvedValue(repository); });

  it("does not access GitHub until the user explicitly connects", () => {
    render(<GitHubPanel />);
    expect(screen.getByRole("button", { name: "Connect GitHub" })).toBeEnabled();
    expect(github.getGitHubRepository).not.toHaveBeenCalled();
  });

  it("connects explicitly and validates PR number before a network request", async () => {
    render(<GitHubPanel />);
    fireEvent.click(screen.getByRole("button", { name: "Connect GitHub" }));
    expect(await screen.findByText("owner/repo")).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("Pull request number"), { target: { value: "0" } });
    fireEvent.click(screen.getByRole("button", { name: "Open PR" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Enter a positive pull request number");
    expect(github.getGitHubPullRequest).not.toHaveBeenCalled();
  });

  it("renders a merged PR and preserves original commit order", async () => {
    github.getGitHubPullRequest.mockResolvedValue({ host: "github.com", nameWithOwner: "owner/repo", number: 42, title: "Ship feature", body: "PR body <safe>", state: "merged", isDraft: false, authorLogin: "ada", url: "https://github.com/owner/repo/pull/42", createdAt: "2026-01-01", updatedAt: "2026-01-02", closedAt: "2026-01-03", mergedAt: "2026-01-03", base: { name: "main", oid: "1".repeat(40), repository: "owner/repo" }, head: { name: "topic", oid: "2".repeat(40), repository: "owner/repo" }, mergeCommitOid: "3".repeat(40), commits: [
      { oid: "a".repeat(40), parents: [], author: identity, committer: identity, summary: "First", message: "First", url: "https://example/1" },
      { oid: "b".repeat(40), parents: ["a".repeat(40)], author: identity, committer: identity, summary: "Second", message: "Second", url: "https://example/2" },
    ] });
    render(<GitHubPanel />);
    fireEvent.click(screen.getByRole("button", { name: "Connect GitHub" }));
    fireEvent.change(await screen.findByLabelText("Pull request number"), { target: { value: "42" } });
    fireEvent.click(screen.getByRole("button", { name: "Open PR" }));
    expect(github.getGitHubPullRequest).toHaveBeenCalledWith(42, "owner/repo");
    expect(await screen.findByRole("heading", { name: "#42 Ship feature" })).toBeInTheDocument();
    expect(screen.getByText("PR body <safe>")).toBeInTheDocument();
    const commits = within(screen.getByRole("list")).getAllByRole("listitem");
    expect(commits[0]).toHaveTextContent("First"); expect(commits[1]).toHaveTextContent("Second");
  });

  it("retains repository and PR number when a request fails and retries", async () => {
    github.getGitHubPullRequest.mockRejectedValueOnce({ code: "github.auth_required", message: "GitHub authentication is required", retryable: true }).mockResolvedValueOnce({ host: "github.com", nameWithOwner: "owner/repo", number: 7, title: "Retry", body: null, state: "open", isDraft: false, authorLogin: null, url: "url", createdAt: "", updatedAt: "", closedAt: null, mergedAt: null, base: { name: "main", oid: "1".repeat(40), repository: null }, head: { name: "topic", oid: "2".repeat(40), repository: null }, mergeCommitOid: null, commits: [] });
    render(<GitHubPanel />); fireEvent.click(screen.getByRole("button", { name: "Connect GitHub" }));
    fireEvent.change(await screen.findByLabelText("Pull request number"), { target: { value: "7" } }); fireEvent.click(screen.getByRole("button", { name: "Open PR" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("GitHub authentication is required");
    expect(screen.getByText("owner/repo")).toBeInTheDocument(); fireEvent.click(screen.getByRole("button", { name: "Retry PR" }));
    expect(await screen.findByRole("heading", { name: "#7 Retry" })).toBeInTheDocument();
  });

  it("loads only a listed original commit and distinguishes unavailable patch", async () => {
    const commit = { oid: "a".repeat(40), parents: [], author: identity, committer: identity, summary: "Original", message: "Original", url: "url" };
    github.getGitHubPullRequest.mockResolvedValue({ host: "github.com", nameWithOwner: "owner/repo", number: 3, title: "PR", body: null, state: "merged", isDraft: false, authorLogin: "ada", url: "url", createdAt: "", updatedAt: "", closedAt: "", mergedAt: "", base: { name: "main", oid: "1".repeat(40), repository: null }, head: { name: "topic", oid: "2".repeat(40), repository: null }, mergeCommitOid: "3".repeat(40), commits: [commit] });
    github.getGitHubPullRequestCommitDiff.mockResolvedValue({ host: "github.com", nameWithOwner: "owner/repo", pullRequestNumber: 3, commit, files: [{ oldPath: "old.bin", newPath: "new.bin", status: "renamed", additions: 2, deletions: 1, changes: 3, patchState: "unavailable", hunks: [] }] });
    render(<GitHubPanel />); fireEvent.click(screen.getByRole("button", { name: "Connect GitHub" })); fireEvent.change(await screen.findByLabelText("Pull request number"), { target: { value: "3" } }); fireEvent.click(screen.getByRole("button", { name: "Open PR" }));
    fireEvent.click(await screen.findByRole("button", { name: `View remote diff ${"a".repeat(8)}` }));
    expect(github.getGitHubPullRequestCommitDiff).toHaveBeenCalledWith(3, commit.oid, "owner/repo");
    expect(await screen.findByText("old.bin → new.bin")).toBeInTheDocument();
    expect(screen.getByText("Provider status: renamed · +2 −1 · 3 changes")).toBeInTheDocument();
    expect(screen.getByText("GitHub did not provide a patch for this file. Binary content is not assumed.")).toBeInTheDocument();
  });

  it("explicitly requests and conservatively presents a squash candidate", async () => {
    const commit = { oid: "a".repeat(40), parents: [], author: identity, committer: identity, summary: "Original", message: "Original", url: "url" };
    const pullRequest = { host: "github.com", nameWithOwner: "owner/repo", number: 8, title: "Trace", body: null, state: "merged", isDraft: false, authorLogin: "ada", url: "url", createdAt: "", updatedAt: "", closedAt: "", mergedAt: "", base: { name: "main", oid: "1".repeat(40), repository: null }, head: { name: "topic", oid: "2".repeat(40), repository: null }, mergeCommitOid: "3".repeat(40), commits: [commit] };
    github.getGitHubPullRequest.mockResolvedValue(pullRequest);
    github.getGitHubSquashTrace.mockResolvedValue({ pullRequest, relationship: { classification: "squashCandidate", confidence: "medium", mergeCommitOid: "3".repeat(40), localAvailability: "available", localParentOids: ["4".repeat(40)], evidence: ["mergeOidDistinctFromOriginalCommits", "localCommitAvailable", "localCommitHasAtMostOneParent", "providerMergeStrategyUnavailable"] } });
    render(<GitHubPanel />); fireEvent.click(screen.getByRole("button", { name: "Connect GitHub" })); fireEvent.change(await screen.findByLabelText("Pull request number"), { target: { value: "8" } }); fireEvent.click(screen.getByRole("button", { name: "Open PR" }));
    expect(await screen.findByRole("button", { name: "Analyze Squash Trace" })).toBeEnabled();
    expect(github.getGitHubSquashTrace).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Analyze Squash Trace" }));
    expect(github.getGitHubSquashTrace).toHaveBeenCalledWith(8, "owner/repo");
    expect(await screen.findByRole("heading", { name: "Squash candidate" })).toBeInTheDocument();
    expect(screen.getByText("medium confidence")).toBeInTheDocument();
    expect(screen.getByText(/candidate relationship, not a verified merge strategy/)).toBeInTheDocument();
    expect(screen.getByText("GitHub merge-strategy metadata is unavailable")).toBeInTheDocument();
  });

  it("keeps PR details on trace failure and retries", async () => {
    const pullRequest = { host: "github.com", nameWithOwner: "owner/repo", number: 9, title: "Retry trace", body: null, state: "merged", isDraft: false, authorLogin: null, url: "url", createdAt: "", updatedAt: "", closedAt: "", mergedAt: "", base: { name: "main", oid: "1".repeat(40), repository: null }, head: { name: "topic", oid: "2".repeat(40), repository: null }, mergeCommitOid: null, commits: [] };
    github.getGitHubPullRequest.mockResolvedValue(pullRequest);
    github.getGitHubSquashTrace.mockRejectedValueOnce({ code: "github.command_failed", message: "GitHub request failed", retryable: true }).mockResolvedValueOnce({ pullRequest, relationship: { classification: "unresolved", confidence: "none", mergeCommitOid: null, localAvailability: "notInspected", localParentOids: [], evidence: ["providerMergeOidMissing"] } });
    render(<GitHubPanel />); fireEvent.click(screen.getByRole("button", { name: "Connect GitHub" })); fireEvent.change(await screen.findByLabelText("Pull request number"), { target: { value: "9" } }); fireEvent.click(screen.getByRole("button", { name: "Open PR" })); fireEvent.click(await screen.findByRole("button", { name: "Analyze Squash Trace" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("GitHub request failed");
    expect(screen.getByRole("heading", { name: "#9 Retry trace" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Retry Squash Trace" }));
    expect(await screen.findByRole("heading", { name: "Relationship unresolved" })).toBeInTheDocument();
    expect(github.getGitHubSquashTrace).toHaveBeenCalledTimes(2);
  });

  it("clears trace evidence when another PR is opened", async () => {
    const makePullRequest = (number: number) => ({ host: "github.com", nameWithOwner: "owner/repo", number, title: `PR ${number}`, body: null, state: "merged", isDraft: false, authorLogin: null, url: "url", createdAt: "", updatedAt: "", closedAt: "", mergedAt: "", base: { name: "main", oid: "1".repeat(40), repository: null }, head: { name: "topic", oid: "2".repeat(40), repository: null }, mergeCommitOid: "3".repeat(40), commits: [] });
    github.getGitHubPullRequest.mockImplementation((number: number) => Promise.resolve(makePullRequest(number)));
    github.getGitHubSquashTrace.mockResolvedValue({ pullRequest: makePullRequest(10), relationship: { classification: "mergeCommit", confidence: "high", mergeCommitOid: "3".repeat(40), localAvailability: "available", localParentOids: ["4".repeat(40), "5".repeat(40)], evidence: ["localCommitHasMultipleParents"] } });
    render(<GitHubPanel />); fireEvent.click(screen.getByRole("button", { name: "Connect GitHub" })); const input = await screen.findByLabelText("Pull request number"); fireEvent.change(input, { target: { value: "10" } }); fireEvent.click(screen.getByRole("button", { name: "Open PR" })); fireEvent.click(await screen.findByRole("button", { name: "Analyze Squash Trace" }));
    expect(await screen.findByText("Final commit has multiple local parents")).toBeInTheDocument();
    fireEvent.change(input, { target: { value: "11" } }); fireEvent.click(screen.getByRole("button", { name: "Open PR" }));
    expect(await screen.findByRole("heading", { name: "#11 PR 11" })).toBeInTheDocument();
    expect(screen.queryByText("Final commit has multiple local parents")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Analyze Squash Trace" })).toBeEnabled();
  });
});
