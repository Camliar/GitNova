import type { CommitGraphNode } from "@gitnova/protocol";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { CommitGraph, projectGraphRows } from "./CommitGraph";

const identity = { name: "Ada", email: "ada@example.com", timestamp: "2026-01-01T00:00:00Z" };
function node(oid: string, parents: string[]): CommitGraphNode {
  return { commit: { oid, parents, summary: oid, message: oid, author: identity, committer: identity }, isHead: false, references: [] };
}

describe("Desktop visual commit graph", () => {
  it("keeps a linear first-parent chain in one lane", () => {
    const rows = projectGraphRows([node("a", ["b"]), node("b", ["c"]), node("c", [])]);
    expect(rows.map((row) => ({ lane: row.lane, parents: row.parentLanes }))).toEqual([
      { lane: 0, parents: [0] }, { lane: 0, parents: [0] }, { lane: 0, parents: [] },
    ]);
  });

  it("routes an ordered merge parent to a second lane and rejoins it", () => {
    const rows = projectGraphRows([node("m", ["a", "b"]), node("a", ["r"]), node("b", ["r"]), node("r", [])]);
    expect(rows[0].parentLanes).toEqual([0, 1]);
    expect(rows[1].continuingLanes).toEqual([1]);
    expect(rows[2].lane).toBe(1);
    expect(rows[2].parentLanes).toEqual([0]);
    expect(rows[3].lane).toBe(0);
  });

  it("retains an off-page parent lane for later pagination", () => {
    const firstPage = projectGraphRows([node("a", ["b"])]);
    const appended = projectGraphRows([node("a", ["b"]), node("b", [])]);
    expect(firstPage[0].parentLanes).toEqual([0]);
    expect(appended[1].lane).toBe(0);
  });

  it("exposes lane and parent count without relying on color", () => {
    const row = projectGraphRows([node("m", ["a", "b"])])[0];
    render(<CommitGraph row={row} isHead />);
    expect(screen.getByRole("img", { name: "Commit graph lane 1; 2 parents" })).toBeInTheDocument();
  });
});
