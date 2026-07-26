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
    expect(rows.map((row) => row.hasIncoming)).toEqual([false, true, true]);
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

  it("uses a stable lane palette and curves only cross-lane parent edges", () => {
    const row = projectGraphRows([node("m", ["a", "b"])])[0];
    const { container } = render(<CommitGraph row={row} isHead={false} />);
    const firstParent = container.querySelector('[data-edge="parent"][data-lane="0"]');
    const secondParent = container.querySelector('[data-edge="parent"][data-lane="1"]');

    expect(firstParent).toHaveAttribute("data-curved", "false");
    expect(firstParent).toHaveAttribute("d", "M 9 14 L 9 28");
    expect(firstParent).toHaveStyle("--graph-color: #e47d14");
    expect(secondParent).toHaveAttribute("data-curved", "true");
    expect(secondParent).toHaveAttribute("d", "M 9 14 C 18 14 27 18 27 28");
    expect(secondParent).toHaveStyle("--graph-color: #1595a3");
  });

  it("keeps a branch color on its first-parent curve when it rejoins another lane", () => {
    const rows = projectGraphRows([node("m", ["a", "b"]), node("a", ["r"]), node("b", ["r"])]);
    const { container } = render(<CommitGraph row={rows[2]} isHead={false} />);
    const joiningEdge = container.querySelector('[data-edge="parent"]');

    expect(rows[2].lane).toBe(1);
    expect(container.querySelector('[data-edge="incoming"]')).toHaveAttribute("d", "M 27 0 L 27 14");
    expect(joiningEdge).toHaveAttribute("data-lane", "1");
    expect(joiningEdge).toHaveAttribute("d", "M 27 14 C 27 21 9 21 9 28");
  });
});
