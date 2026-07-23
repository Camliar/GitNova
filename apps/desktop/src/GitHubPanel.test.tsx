import { fireEvent, render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { GitHubPanel } from "./GitHubPanel";

const github = vi.hoisted(() => ({ getGitHubRepository: vi.fn(), getGitHubPullRequest: vi.fn(), getGitHubPullRequestCommitDiff: vi.fn() }));
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
});
