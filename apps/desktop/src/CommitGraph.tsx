import type { CSSProperties } from "react";
import type { CommitGraphNode } from "@gitnova/protocol";

export interface VisualGraphRow {
  oid: string;
  lane: number;
  laneCount: number;
  continuingLanes: number[];
  parentLanes: number[];
}

function firstOpenLane(lanes: Array<string | null>, preferred: number) {
  if (lanes[preferred] == null) return preferred;
  const open = lanes.findIndex((value, index) => index > preferred && value == null);
  return open >= 0 ? open : lanes.length;
}

export function projectGraphRows(nodes: CommitGraphNode[]): VisualGraphRow[] {
  const lanes: Array<string | null> = [];
  return nodes.map(({ commit }) => {
    let lane = lanes.indexOf(commit.oid);
    if (lane < 0) {
      lane = lanes.findIndex((value) => value == null);
      if (lane < 0) lane = lanes.length;
      lanes[lane] = commit.oid;
    }

    const continuingLanes = lanes.flatMap((value, index) => value != null && index !== lane ? [index] : []);
    lanes[lane] = null;
    const parentLanes = commit.parents.map((parent, index) => {
      const existing = lanes.indexOf(parent);
      if (existing >= 0) return existing;
      const target = index === 0 && lanes[lane] == null ? lane : firstOpenLane(lanes, lane + 1);
      lanes[target] = parent;
      return target;
    });
    while (lanes.length > 1 && lanes.at(-1) == null) lanes.pop();
    return { oid: commit.oid, lane, laneCount: Math.max(lanes.length, lane + 1, ...parentLanes.map((value) => value + 1)), continuingLanes, parentLanes };
  });
}

const LANE_WIDTH = 18;
const NODE_Y = 18;

function laneX(lane: number) {
  return lane * LANE_WIDTH + 9;
}

export function CommitGraph({ row, isHead }: { row: VisualGraphRow; isHead: boolean }) {
  const width = row.laneCount * LANE_WIDTH;
  const style = { "--graph-width": `${width}px` } as CSSProperties;
  return <span className="commit-graph" style={style}>
    <svg width={width} height="100%" role="img" aria-label={`Commit graph lane ${row.lane + 1}; ${row.parentLanes.length} parent${row.parentLanes.length === 1 ? "" : "s"}`}>
      {row.continuingLanes.map((lane) => <line className="graph-line graph-line--continuing" key={`continuing:${lane}`} x1={laneX(lane)} y1="0" x2={laneX(lane)} y2="100%" />)}
      {row.parentLanes.map((parentLane, index) => <line className={`graph-line graph-line--parent graph-line--parent-${index}`} key={`${row.oid}:${parentLane}:${index}`} x1={laneX(row.lane)} y1={NODE_Y} x2={laneX(parentLane)} y2="100%" />)}
      <circle className={`graph-node${isHead ? " graph-node--head" : ""}`} cx={laneX(row.lane)} cy={NODE_Y} r="5" />
    </svg>
  </span>;
}
