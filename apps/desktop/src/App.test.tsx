import { fireEvent, render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";

const core = vi.hoisted(() => ({ getCoreStatus: vi.fn(), startCore: vi.fn() }));
const repository = vi.hoisted(() => ({ selectRepositoryDirectory: vi.fn(), openRepository: vi.fn() }));
const status = vi.hoisted(() => ({ getWorkingTreeStatus: vi.fn() }));
const diff = vi.hoisted(() => ({ getFileDiff: vi.fn() }));

vi.mock("./core", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./core")>()),
  getCoreStatus: core.getCoreStatus,
  startCore: core.startCore,
}));
vi.mock("./repository", () => repository);
vi.mock("./status", () => status);
vi.mock("./diff", () => diff);

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
