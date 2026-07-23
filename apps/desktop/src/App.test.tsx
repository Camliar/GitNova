import { fireEvent, render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";

const core = vi.hoisted(() => ({ getCoreStatus: vi.fn(), startCore: vi.fn() }));
const repository = vi.hoisted(() => ({ selectRepositoryDirectory: vi.fn(), openRepository: vi.fn() }));
const status = vi.hoisted(() => ({ getWorkingTreeStatus: vi.fn() }));
const diff = vi.hoisted(() => ({ getFileDiff: vi.fn() }));
const history = vi.hoisted(() => ({ getCommitGraph: vi.fn() }));
const commitDiff = vi.hoisted(() => ({ getCommitDiff: vi.fn() }));

vi.mock("./core", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./core")>()),
  getCoreStatus: core.getCoreStatus,
  startCore: core.startCore,
}));
vi.mock("./repository", () => repository);
vi.mock("./status", () => status);
vi.mock("./diff", () => diff);
vi.mock("./history", () => history);
vi.mock("./commitDiff", () => commitDiff);

const descriptor = {
  worktreeRoot: "/work/project",
  gitDirectory: "/work/project/.git",
  commonGitDirectory: "/work/project/.git",
  kind: "worktree" as const,
  gitVersion: "git version 2.50.0",
};

describe("Desktop repository open", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    core.getCoreStatus.mockResolvedValue({ connected: true, protocolVersion: "1.11", capabilities: null });
    core.startCore.mockResolvedValue({ connected: true, protocolVersion: "1.11", capabilities: null });
    repository.selectRepositoryDirectory.mockResolvedValue("/work/project");
    repository.openRepository.mockResolvedValue(descriptor);
    status.getWorkingTreeStatus.mockResolvedValue({
      branch: { head: "main", oid: "a".repeat(40), upstream: "origin/main", ahead: 1, behind: 2 },
      entries: [],
    });
    diff.getFileDiff.mockResolvedValue({ oldPath: "src/app.ts", newPath: "src/app.ts", isBinary: false, hunks: [] });
    history.getCommitGraph.mockResolvedValue({ nodes: [], nextCursor: null });
    commitDiff.getCommitDiff.mockResolvedValue({ commit: null, parentOid: null, files: [] });
  });

  it("opens only the explicitly selected directory and presents Core facts", async () => {
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));

    expect(await screen.findByRole("heading", { name: "Worktree" })).toBeInTheDocument();
    expect(repository.openRepository).toHaveBeenCalledWith("/work/project");
    expect(screen.getByText("/work/project/.git")).toBeInTheDocument();
    expect(screen.getByText("git version 2.50.0")).toBeInTheDocument();
    expect(await screen.findByText("Working tree clean")).toBeInTheDocument();
    expect(status.getWorkingTreeStatus).toHaveBeenCalledOnce();
  });

  it("renders Core-projected commit order, HEAD, refs, and merge parents", async () => {
    history.getCommitGraph.mockResolvedValue({
      nodes: [
        {
          commit: { oid: "1".repeat(40), parents: ["2".repeat(40), "3".repeat(40)], summary: "Merge topic", message: "Merge topic\n", author: { name: "Ada", email: "ada@example.com", timestamp: "2026-01-02T03:04:05+08:00" }, committer: { name: "Ada", email: "ada@example.com", timestamp: "2026-01-02T03:04:05+08:00" } },
          isHead: true,
          references: [
            { name: "main", fullName: "refs/heads/main", kind: "localBranch", targetOid: "1".repeat(40), peeledTargetOid: null, symbolicTarget: null, upstream: "origin/main" },
            { name: "v1.0", fullName: "refs/tags/v1.0", kind: "tag", targetOid: "1".repeat(40), peeledTargetOid: null, symbolicTarget: null, upstream: null },
          ],
        },
        {
          commit: { oid: "2".repeat(40), parents: [], summary: "Initial commit", message: "Initial commit\n", author: { name: "Lin", email: "lin@example.com", timestamp: "2025-12-01T00:00:00Z" }, committer: { name: "Lin", email: "lin@example.com", timestamp: "2025-12-01T00:00:00Z" } },
          isHead: false,
          references: [],
        },
      ],
      nextCursor: null,
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));

    await screen.findByText("Merge topic");
    const historyList = screen.getByRole("list", { name: "Commit history" });
    const commits = within(historyList).getAllByRole("listitem");
    expect(commits[0]).toHaveTextContent("Merge topic");
    expect(commits[1]).toHaveTextContent("Initial commit");
    expect(within(historyList).getByText("HEAD")).toBeInTheDocument();
    expect(within(historyList).getByText("main")).toBeInTheDocument();
    expect(within(historyList).getByText("v1.0")).toBeInTheDocument();
    expect(commits[0]).toHaveTextContent("Merge (2 parents)");
  });

  it("loads a root commit without choosing a parent and renders full metadata", async () => {
    const root = { oid: "a".repeat(40), parents: [], summary: "Root", message: "Root\n\nFull message", author: { name: "Ada", email: "ada@example.com", timestamp: "2026-01-01T01:00:00Z" }, committer: { name: "Lin", email: "lin@example.com", timestamp: "2026-01-01T02:00:00Z" } };
    history.getCommitGraph.mockResolvedValue({ nodes: [{ commit: root, isHead: true, references: [] }], nextCursor: null });
    commitDiff.getCommitDiff.mockResolvedValue({ commit: root, parentOid: null, files: [] });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));
    fireEvent.click(await screen.findByRole("button", { name: `View commit ${"a".repeat(8)}` }));

    expect(commitDiff.getCommitDiff).toHaveBeenCalledWith(root.oid, undefined);
    expect(await screen.findByText("Root commit (empty tree)")).toBeInTheDocument();
    expect(screen.getByText("Ada <ada@example.com> · 2026-01-01T01:00:00Z")).toBeInTheDocument();
    expect(screen.getByText("Lin <lin@example.com> · 2026-01-01T02:00:00Z")).toBeInTheDocument();
    expect(screen.getByText(/Full message/)).toBeInTheDocument();
  });

  it("requires explicit direct-parent choice for a merge commit", async () => {
    const parents = ["b".repeat(40), "c".repeat(40)];
    const merge = { oid: "a".repeat(40), parents, summary: "Merge", message: "Merge\n", author: { name: "Ada", email: "a@b.c", timestamp: "2026-01-01T00:00:00Z" }, committer: { name: "Ada", email: "a@b.c", timestamp: "2026-01-01T00:00:00Z" } };
    history.getCommitGraph.mockResolvedValue({ nodes: [{ commit: merge, isHead: true, references: [] }], nextCursor: null });
    commitDiff.getCommitDiff.mockResolvedValue({ commit: merge, parentOid: parents[1], files: [] });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));
    fireEvent.click(await screen.findByRole("button", { name: `View commit ${"a".repeat(8)}` }));

    expect(await screen.findByText("This merge has multiple parents. Choose the parent edge to compare.")).toBeInTheDocument();
    expect(commitDiff.getCommitDiff).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: `Compare parent ${"c".repeat(8)}` }));
    expect(commitDiff.getCommitDiff).toHaveBeenCalledWith(merge.oid, parents[1]);
    expect(await screen.findByText(parents[1])).toBeInTheDocument();
  });

  it("renders ordered commit files and reusable text and binary diff states", async () => {
    const commit = { oid: "d".repeat(40), parents: ["e".repeat(40)], summary: "Change files", message: "Change files\n", author: { name: "Ada", email: "a@b.c", timestamp: "2026-01-01T00:00:00Z" }, committer: { name: "Ada", email: "a@b.c", timestamp: "2026-01-01T00:00:00Z" } };
    history.getCommitGraph.mockResolvedValue({ nodes: [{ commit, isHead: true, references: [] }], nextCursor: null });
    commitDiff.getCommitDiff.mockResolvedValue({ commit, parentOid: commit.parents[0], files: [
      { oldPath: "old.ts", newPath: "new.ts", isBinary: false, hunks: [{ oldStart: 1, oldLines: 1, newStart: 1, newLines: 1, header: "", lines: [{ kind: "addition", content: "safe text", oldLine: null, newLine: 1 }] }] },
      { oldPath: "image.png", newPath: "image.png", isBinary: true, hunks: [] },
    ] });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));
    fireEvent.click(await screen.findByRole("button", { name: `View commit ${"d".repeat(8)}` }));

    expect(await screen.findByRole("region", { name: "Commit diff for new.ts" })).toHaveTextContent("safe text");
    expect(screen.getByText("from old.ts")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "image.png" }));
    expect(await screen.findByText("Binary file changed. Content is not returned by Core.")).toBeInTheDocument();
  });

  it("keeps timeline context when commit diff fails and retries the same edge", async () => {
    const commit = { oid: "f".repeat(40), parents: ["e".repeat(40)], summary: "Retry me", message: "Retry me\n", author: { name: "Ada", email: "a@b.c", timestamp: "2026-01-01T00:00:00Z" }, committer: { name: "Ada", email: "a@b.c", timestamp: "2026-01-01T00:00:00Z" } };
    history.getCommitGraph.mockResolvedValue({ nodes: [{ commit, isHead: true, references: [] }], nextCursor: null });
    commitDiff.getCommitDiff.mockRejectedValueOnce({ code: "commit.not_found", message: "Commit unavailable", retryable: true }).mockResolvedValueOnce({ commit, parentOid: commit.parents[0], files: [] });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));
    fireEvent.click(await screen.findByRole("button", { name: `View commit ${"f".repeat(8)}` }));

    expect(await screen.findByRole("alert")).toHaveTextContent("Commit unavailable. Commit history is still available.");
    expect(screen.getByText("Retry me", { selector: ".commit-summary strong" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Retry commit diff" }));
    expect(await screen.findByText("No changed files in this comparison.")).toBeInTheDocument();
    expect(commitDiff.getCommitDiff).toHaveBeenLastCalledWith(commit.oid, undefined);
  });

  it("does not restore a stale commit response after the detail is closed", async () => {
    const commit = { oid: "9".repeat(40), parents: [], summary: "Slow commit", message: "Slow commit\n", author: { name: "Ada", email: "a@b.c", timestamp: "2026-01-01T00:00:00Z" }, committer: { name: "Ada", email: "a@b.c", timestamp: "2026-01-01T00:00:00Z" } };
    history.getCommitGraph.mockResolvedValue({ nodes: [{ commit, isHead: true, references: [] }], nextCursor: null });
    let resolveDiff!: (value: unknown) => void;
    commitDiff.getCommitDiff.mockReturnValue(new Promise((resolve) => { resolveDiff = resolve; }));
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));
    fireEvent.click(await screen.findByRole("button", { name: `View commit ${"9".repeat(8)}` }));
    fireEvent.click(await screen.findByRole("button", { name: "Close commit" }));
    resolveDiff({ commit, parentOid: null, files: [] });
    await Promise.resolve();

    expect(screen.queryByRole("heading", { name: "Slow commit" })).not.toBeInTheDocument();
    expect(screen.getByText("Slow commit", { selector: ".commit-summary strong" })).toBeInTheDocument();
  });

  it("appends an opaque cursor page without changing existing commits", async () => {
    const node = (oid: string, summary: string) => ({
      commit: { oid: oid.repeat(40), parents: [], summary, message: `${summary}\n`, author: { name: "Ada", email: "ada@example.com", timestamp: "2026-01-01T00:00:00Z" }, committer: { name: "Ada", email: "ada@example.com", timestamp: "2026-01-01T00:00:00Z" } },
      isHead: false, references: [],
    });
    history.getCommitGraph
      .mockResolvedValueOnce({ nodes: [node("1", "Newest")], nextCursor: "opaque:cursor" })
      .mockResolvedValueOnce({ nodes: [node("2", "Older")], nextCursor: null });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));
    fireEvent.click(await screen.findByRole("button", { name: "Load more" }));

    expect(await screen.findByText("Older")).toBeInTheDocument();
    expect(screen.getByText("Newest")).toBeInTheDocument();
    expect(history.getCommitGraph).toHaveBeenNthCalledWith(1);
    expect(history.getCommitGraph).toHaveBeenNthCalledWith(2, "opaque:cursor");
  });

  it("keeps loaded commits when load more fails and retries the same cursor", async () => {
    const firstNode = { commit: { oid: "1".repeat(40), parents: [], summary: "Kept commit", message: "Kept commit\n", author: { name: "Ada", email: "a@b.c", timestamp: "2026-01-01T00:00:00Z" }, committer: { name: "Ada", email: "a@b.c", timestamp: "2026-01-01T00:00:00Z" } }, isHead: true, references: [] };
    history.getCommitGraph
      .mockResolvedValueOnce({ nodes: [firstNode], nextCursor: "next" })
      .mockRejectedValueOnce({ code: "history.invalid_cursor", message: "History page failed", retryable: true })
      .mockResolvedValueOnce({ nodes: [], nextCursor: null });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));
    fireEvent.click(await screen.findByRole("button", { name: "Load more" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("History page failed. Loaded commits were kept.");
    expect(screen.getByText("Kept commit")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Retry load more" }));
    expect(await screen.findByText("Kept commit")).toBeInTheDocument();
    expect(history.getCommitGraph).toHaveBeenLastCalledWith("next");
  });

  it("loads history for bare repositories without requesting working tree status", async () => {
    repository.openRepository.mockResolvedValue({ ...descriptor, kind: "bare", worktreeRoot: null });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));

    expect(await screen.findByText("No commits yet")).toBeInTheDocument();
    expect(history.getCommitGraph).toHaveBeenCalledOnce();
    expect(status.getWorkingTreeStatus).not.toHaveBeenCalled();
  });

  it("keeps staged and working tree changes distinct and preserves rename paths", async () => {
    status.getWorkingTreeStatus.mockResolvedValue({
      branch: { head: "feature/status", oid: "b".repeat(40), upstream: null, ahead: 0, behind: 0 },
      entries: [
        { path: "src/new.ts", originalPath: "src/old.ts", kind: "renameOrCopy", indexStatus: "renamed", worktreeStatus: "modified" },
        { path: "notes.txt", originalPath: null, kind: "untracked", indexStatus: "unmodified", worktreeStatus: "untracked" },
        { path: "conflict.txt", originalPath: null, kind: "unmerged", indexStatus: "unmerged", worktreeStatus: "unmerged" },
      ],
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));

    expect(await screen.findByRole("heading", { name: "feature/status" })).toBeInTheDocument();
    expect(screen.getByText("from src/old.ts")).toBeInTheDocument();
    expect(screen.getByText("Staged · Renamed")).toBeInTheDocument();
    expect(screen.getByText("Working · Modified")).toBeInTheDocument();
    expect(screen.getByText("Working · Untracked")).toBeInTheDocument();
    expect(screen.getAllByText(/Conflict/)).toHaveLength(2);
  });

  it("requests the selected scope and renders structured lines as text", async () => {
    status.getWorkingTreeStatus.mockResolvedValue({
      branch: { head: "main", oid: "a".repeat(40), upstream: null, ahead: 0, behind: 0 },
      entries: [{ path: "src/app.ts", originalPath: null, kind: "ordinary", indexStatus: "added", worktreeStatus: "modified" }],
    });
    diff.getFileDiff.mockResolvedValue({
      oldPath: "src/app.ts", newPath: "src/app.ts", isBinary: false,
      hunks: [{ oldStart: 4, oldLines: 1, newStart: 4, newLines: 1, header: "function run()", lines: [
        { kind: "deletion", content: "return '<unsafe>';", oldLine: 4, newLine: null },
        { kind: "addition", content: "return '& safe';", oldLine: null, newLine: 4 },
      ] }],
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));
    fireEvent.click(await screen.findByRole("button", { name: "View working diff for src/app.ts" }));

    expect(diff.getFileDiff).toHaveBeenCalledWith("src/app.ts", "workingTree");
    const hunk = await screen.findByRole("region", { name: "Diff hunk 1" });
    expect(within(hunk).getByText("@@ -4,1 +4,1 @@ function run()")).toBeInTheDocument();
    expect(within(hunk).getByText("return '<unsafe>';" )).toBeInTheDocument();
    expect(within(hunk).getByText("return '& safe';")).toBeInTheDocument();
  });

  it("does not offer a synthetic diff for an untracked file", async () => {
    status.getWorkingTreeStatus.mockResolvedValue({
      branch: { head: "main", oid: "a".repeat(40), upstream: null, ahead: 0, behind: 0 },
      entries: [{ path: "new.txt", originalPath: null, kind: "untracked", indexStatus: "unmodified", worktreeStatus: "untracked" }],
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));

    expect(await screen.findByText("Working · Untracked")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /diff for new\.txt/ })).not.toBeInTheDocument();
  });

  it("presents binary and empty scope results without content", async () => {
    status.getWorkingTreeStatus.mockResolvedValue({
      branch: { head: "main", oid: "a".repeat(40), upstream: null, ahead: 0, behind: 0 },
      entries: [{ path: "image.png", originalPath: null, kind: "ordinary", indexStatus: "modified", worktreeStatus: "unmodified" }],
    });
    diff.getFileDiff.mockResolvedValue({ oldPath: "image.png", newPath: "image.png", isBinary: true, hunks: [] });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));
    fireEvent.click(await screen.findByRole("button", { name: "View staged diff for image.png" }));

    expect(await screen.findByText("Binary file changed. Content is not returned by Core.")).toBeInTheDocument();
  });

  it("keeps status available on diff error and retries the same selection", async () => {
    status.getWorkingTreeStatus.mockResolvedValue({
      branch: { head: "main", oid: "a".repeat(40), upstream: null, ahead: 0, behind: 0 },
      entries: [{ path: "src/app.ts", originalPath: null, kind: "ordinary", indexStatus: "unmodified", worktreeStatus: "modified" }],
    });
    diff.getFileDiff
      .mockRejectedValueOnce({ code: "git.command_failed", message: "Diff failed", retryable: true })
      .mockResolvedValueOnce({ oldPath: "src/app.ts", newPath: "src/app.ts", isBinary: false, hunks: [] });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));
    fireEvent.click(await screen.findByRole("button", { name: "View working diff for src/app.ts" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("Diff failed. Working tree status is still available.");
    expect(screen.getByText("Working · Modified")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Retry diff" }));
    expect(await screen.findByText("No changes in this scope.")).toBeInTheDocument();
    expect(diff.getFileDiff).toHaveBeenLastCalledWith("src/app.ts", "workingTree");
  });

  it("closes a previous diff when status is refreshed", async () => {
    status.getWorkingTreeStatus.mockResolvedValue({
      branch: { head: "main", oid: "a".repeat(40), upstream: null, ahead: 0, behind: 0 },
      entries: [{ path: "src/app.ts", originalPath: null, kind: "ordinary", indexStatus: "modified", worktreeStatus: "unmodified" }],
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));
    fireEvent.click(await screen.findByRole("button", { name: "View staged diff for src/app.ts" }));
    expect(await screen.findByRole("heading", { name: "src/app.ts" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Refresh" }));

    expect(screen.queryByRole("heading", { name: "src/app.ts" })).not.toBeInTheDocument();
  });

  it("does not request working tree status for a bare repository", async () => {
    repository.openRepository.mockResolvedValue({ ...descriptor, kind: "bare", worktreeRoot: null });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));

    expect(await screen.findByText("Bare repositories do not have a working tree.")).toBeInTheDocument();
    expect(status.getWorkingTreeStatus).not.toHaveBeenCalled();
  });

  it("distinguishes an unborn named branch from detached HEAD", async () => {
    status.getWorkingTreeStatus.mockResolvedValue({
      branch: { head: "main", oid: null, upstream: null, ahead: 0, behind: 0 },
      entries: [],
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));

    expect(await screen.findByRole("heading", { name: "main · Unborn" })).toBeInTheDocument();
  });

  it("keeps the repository open when status fails and retries explicitly", async () => {
    status.getWorkingTreeStatus
      .mockRejectedValueOnce({ code: "git.command_failed", message: "Git could not read status", retryable: true })
      .mockResolvedValueOnce({ branch: { head: null, oid: "c".repeat(40), upstream: null, ahead: 0, behind: 0 }, entries: [] });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("Git could not read status. The repository remains open.");
    expect(screen.getByRole("heading", { name: "Worktree" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Refresh" }));
    expect(await screen.findByRole("heading", { name: "Detached HEAD" })).toBeInTheDocument();
  });

  it("reopens the same Core repository idempotently without another picker", async () => {
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));
    fireEvent.click(await screen.findByRole("button", { name: "Reopen repository" }));

    expect(await screen.findByRole("button", { name: "Reopen repository" })).toBeEnabled();
    expect(repository.selectRepositoryDirectory).toHaveBeenCalledTimes(1);
    expect(repository.openRepository).toHaveBeenLastCalledWith("/work/project");
  });

  it("treats picker cancellation as an idle state", async () => {
    repository.selectRepositoryDirectory.mockResolvedValue(null);
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));

    expect(await screen.findByRole("button", { name: "Choose repository" })).toBeEnabled();
    expect(repository.openRepository).not.toHaveBeenCalled();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("disables duplicate actions while opening", async () => {
    repository.selectRepositoryDirectory.mockReturnValue(new Promise(() => undefined));
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));

    expect(screen.getByRole("button", { name: "Opening…" })).toBeDisabled();
  });

  it("shows a stable Core domain error and allows another selection", async () => {
    repository.openRepository.mockRejectedValue({ code: "repository.not_found", message: "Repository was not found", retryable: true });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("Repository was not found. No repository data was changed.");
    expect(screen.getByRole("button", { name: "Choose another folder" })).toBeEnabled();
  });

  it("requires an explicit successful Core start before repository selection", async () => {
    core.getCoreStatus.mockResolvedValue({ connected: false, protocolVersion: null, capabilities: null });
    render(<App />);

    expect(await screen.findByRole("button", { name: "Start Core" })).toBeEnabled();
    expect(screen.queryByRole("button", { name: "Choose repository" })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Start Core" }));
    expect(await screen.findByRole("button", { name: "Choose repository" })).toBeEnabled();
  });

  it("shows a sanitized retry state when Core startup fails", async () => {
    core.getCoreStatus.mockResolvedValue({ connected: false, protocolVersion: null, capabilities: null });
    core.startCore.mockRejectedValue({ code: "desktop.core_unavailable", message: "GitNova Core executable is unavailable", retryable: true });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Start Core" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("GitNova Core executable is unavailable. No repository data was changed.");
    expect(screen.getByRole("button", { name: "Retry Core" })).toBeEnabled();
  });
});
