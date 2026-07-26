import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { CommitDetailPanel } from "./CommitDetailPanel";

describe("CommitDetailPanel", () => {
  it("distinguishes an empty commit from a failed changed-file request", () => {
    const oid = "a".repeat(40);
    const parentOid = "b".repeat(40);
    const commit = {
      oid,
      parents: [parentOid],
      summary: "Empty backport",
      message: "Empty backport\n",
      author: { name: "Ada", email: "ada@example.com", timestamp: "2026-01-01T00:00:00Z" },
      committer: { name: "Ada", email: "ada@example.com", timestamp: "2026-01-01T00:00:00Z" },
    };
    render(<CommitDetailPanel state={{ kind: "ready", selection: { commit }, files: { commit, parentOid, files: [] } }} fileDiff={{ kind: "idle" }} mode="changes" onChooseParent={vi.fn()} onSelectFile={vi.fn()} onRetry={vi.fn()} onRetryFile={vi.fn()} onClose={vi.fn()} />);
    expect(screen.getByText("Empty commit: its tree is identical to the selected comparison baseline.")).toBeInTheDocument();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });
});
