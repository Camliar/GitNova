import { fireEvent, render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { HistorySquashTrace } from "./HistorySquashTrace";

const github = vi.hoisted(() => ({ getGitHubPullRequestCommitFiles: vi.fn(), getGitHubPullRequestCommitFileDiff: vi.fn() }));
vi.mock("./github", () => github);

const original = { oid: "a".repeat(40), parents: ["9".repeat(40)], summary: "Original one", message: "Original one\n\nBody", author: { name: "Ada", email: "a@b.c", timestamp: "2026-01-01T00:00:00Z", login: "ada" }, committer: { name: "Ada", email: "a@b.c", timestamp: "2026-01-01T00:00:00Z", login: null }, url: "https://github.com/owner/repo/commit/a" };
const pullRequest = { host: "github.com" as const, nameWithOwner: "owner/repo", number: 49, title: "Ship feature", body: null, state: "merged" as const, isDraft: false, authorLogin: "ada", url: "https://github.com/owner/repo/pull/49", createdAt: "2026-01-01T00:00:00Z", updatedAt: "2026-01-03T00:00:00Z", closedAt: "2026-01-03T00:00:00Z", mergedAt: "2026-01-03T00:00:00Z", base: { name: "main", oid: "8".repeat(40), repository: "owner/repo" }, head: { name: "topic", oid: original.oid, repository: "owner/repo" }, mergeCommitOid: "c".repeat(40), commits: [original] };
const trace = { pullRequest, relationship: { classification: "squashCandidate" as const, confidence: "medium" as const, mergeCommitOid: "c".repeat(40), localAvailability: "available" as const, localParentOids: ["b".repeat(40)], evidence: ["providerMergeStrategyUnavailable" as const] } };

describe("history Squash Trace", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    github.getGitHubPullRequestCommitFiles.mockResolvedValue({ host: "github.com", nameWithOwner: "owner/repo", pullRequestNumber: 49, commit: original, files: [{ oldPath: "old.ts", newPath: "new.ts", status: "renamed", additions: 1, deletions: 1, changes: 2, patchState: "available" }] });
    github.getGitHubPullRequestCommitFileDiff.mockResolvedValue({ oldPath: "old.ts", newPath: "new.ts", status: "renamed", additions: 1, deletions: 1, changes: 2, patchState: "available", hunks: [{ oldStart: 1, oldLines: 1, newStart: 1, newLines: 1, header: "", lines: [{ kind: "deletion", content: "old", oldLine: 1, newLine: null }, { kind: "addition", content: "new", oldLine: null, newLine: 1 }] }] });
  });

  it("shows ordered originals then loads only the selected file patch", async () => {
    render(<HistorySquashTrace trace={trace} />);
    fireEvent.click(screen.getByRole("button", { name: /Original one/ }));
    expect(await screen.findByText(/Body/)).toBeInTheDocument();
    expect(github.getGitHubPullRequestCommitFiles).toHaveBeenCalledWith(49, original.oid, "owner/repo");
    expect(github.getGitHubPullRequestCommitFileDiff).not.toHaveBeenCalled();

    const tabs = screen.getByRole("tablist", { name: "Original commit detail views" });
    fireEvent.click(within(tabs).getByRole("tab", { name: "Changes · 1" }));
    fireEvent.click(await screen.findByRole("button", { name: /new\.ts/ }));
    expect(await screen.findByRole("region", { name: "Original commit diff for new.ts" })).toHaveTextContent("new");
    expect(github.getGitHubPullRequestCommitFileDiff).toHaveBeenCalledWith(49, original.oid, "new.ts", "owner/repo");
  });
});
