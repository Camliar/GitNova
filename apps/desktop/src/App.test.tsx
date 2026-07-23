import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";

const core = vi.hoisted(() => ({ getCoreStatus: vi.fn(), startCore: vi.fn() }));
const repository = vi.hoisted(() => ({ selectRepositoryDirectory: vi.fn(), openRepository: vi.fn() }));

vi.mock("./core", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./core")>()),
  getCoreStatus: core.getCoreStatus,
  startCore: core.startCore,
}));
vi.mock("./repository", () => repository);

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
  });

  it("opens only the explicitly selected directory and presents Core facts", async () => {
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Choose repository" }));

    expect(await screen.findByRole("heading", { name: "Worktree" })).toBeInTheDocument();
    expect(repository.openRepository).toHaveBeenCalledWith("/work/project");
    expect(screen.getByText("/work/project/.git")).toBeInTheDocument();
    expect(screen.getByText("git version 2.50.0")).toBeInTheDocument();
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
