import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { RepositorySyncControls } from "./RepositorySyncControls";

const sync = vi.hoisted(() => ({ fetchRepository: vi.fn(), pullRepository: vi.fn(), pushRepository: vi.fn() }));
vi.mock("./sync", () => sync);

const status = { head: "main", oid: "a".repeat(40), upstream: "origin/main", ahead: 1, behind: 2 };
const snapshot = { status: { branch: status, entries: [] }, references: { head: { oid: status.oid, symbolicRef: "refs/heads/main" }, references: [] } };

describe("repository sync controls", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    sync.fetchRepository.mockResolvedValue({ operation: "fetch", remote: "origin", branch: "main", remoteBranch: "main", snapshot });
    sync.pullRepository.mockResolvedValue({ operation: "pull", remote: "origin", branch: "main", remoteBranch: "main", snapshot });
    sync.pushRepository.mockResolvedValue({ operation: "push", remote: "origin", branch: "main", remoteBranch: "main", snapshot });
  });

  it("fetches directly but confirms exact branch and HEAD before pull", async () => {
    const onApplied = vi.fn();
    render(<RepositorySyncControls branch={status} onApplied={onApplied} />);
    fireEvent.click(screen.getByRole("button", { name: "Fetch" }));
    expect(await screen.findByText("Fetch origin/main complete")).toBeInTheDocument();
    expect(sync.fetchRepository).toHaveBeenCalledTimes(1);
    expect(onApplied).toHaveBeenCalledWith(snapshot);

    fireEvent.click(screen.getByRole("button", { name: "Pull" }));
    const confirmation = screen.getByRole("group", { name: "Confirm pull" });
    expect(confirmation).toHaveTextContent(`main at ${"a".repeat(8)}`);
    expect(confirmation).toHaveTextContent("Only a fast-forward is allowed");
    fireEvent.click(screen.getByRole("button", { name: "Confirm pull" }));
    expect(await screen.findByText("Pull origin/main complete")).toBeInTheDocument();
    expect(sync.pullRepository).toHaveBeenCalledWith("main", "a".repeat(40));
  });

  it("confirms a non-force push and disables pull without upstream", async () => {
    render(<RepositorySyncControls branch={{ ...status, upstream: null }} onApplied={vi.fn()} />);
    expect(screen.getByRole("button", { name: "Pull" })).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "Push" }));
    expect(screen.getByRole("group", { name: "Confirm push" })).toHaveTextContent("will not force, delete, or push another ref");
    fireEvent.click(screen.getByRole("button", { name: "Confirm push" }));
    expect(await screen.findByText("Push origin/main complete")).toBeInTheDocument();
    expect(sync.pushRepository).toHaveBeenCalledWith("main", "a".repeat(40));
  });
});
