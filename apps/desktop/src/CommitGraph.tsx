import type { CSSProperties } from "react";
import type { CommitGraphNode } from "@gitnova/protocol";

export interface VisualGraphRow {
  oid: string;
  lane: number;
  laneCount: number;
  hasIncoming: boolean;
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
    const hasIncoming = lane >= 0;
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
    return { oid: commit.oid, lane, laneCount: Math.max(lanes.length, lane + 1, ...parentLanes.map((value) => value + 1)), hasIncoming, continuingLanes, parentLanes };
  });
}

const LANE_WIDTH = 18;
const NODE_Y = 14;
const ROW_HEIGHT = 28;
const LANE_COLORS = ["#e47d14", "#1595a3", "#765fc4", "#c6507c", "#3f9255", "#b07a12", "#3d72b8", "#bd514a"] as const;

function laneX(lane: number) {
  return lane * LANE_WIDTH + 9;
}

function laneStyle(lane: number) {
  return { "--graph-color": LANE_COLORS[lane % LANE_COLORS.length] } as CSSProperties;
}

function continuingPath(lane: number) {
  const x = laneX(lane);
  return `M ${x} 0 L ${x} ${ROW_HEIGHT}`;
}

function parentPath(sourceLane: number, parentLane: number, parentIndex: number) {
  const sourceX = laneX(sourceLane);
  const parentX = laneX(parentLane);
  if (sourceLane === parentLane) return `M ${sourceX} ${NODE_Y} L ${parentX} ${ROW_HEIGHT}`;
  if (parentIndex === 0) return `M ${sourceX} ${NODE_Y} C ${sourceX} 21 ${parentX} 21 ${parentX} ${ROW_HEIGHT}`;
  const direction = Math.sign(parentX - sourceX);
  const turn = Math.min(10, Math.abs(parentX - sourceX) / 2);
  return `M ${sourceX} ${NODE_Y} C ${sourceX + direction * turn} ${NODE_Y} ${parentX} 18 ${parentX} ${ROW_HEIGHT}`;
}

export function CommitGraph({ row, isHead }: { row: VisualGraphRow; isHead: boolean }) {
  const width = row.laneCount * LANE_WIDTH;
  const style = { "--graph-width": `${width}px` } as CSSProperties;
  return <span className="commit-graph" style={style}>
    <svg width={width} height={ROW_HEIGHT} viewBox={`0 0 ${width} ${ROW_HEIGHT}`} role="img" aria-label={`Commit graph lane ${row.lane + 1}; ${row.parentLanes.length} parent${row.parentLanes.length === 1 ? "" : "s"}`}>
      {row.hasIncoming && <path className="graph-line graph-line--incoming" data-edge="incoming" data-lane={row.lane} d={`M ${laneX(row.lane)} 0 L ${laneX(row.lane)} ${NODE_Y}`} style={laneStyle(row.lane)} />}
      {row.continuingLanes.map((lane) => <path className="graph-line graph-line--continuing" data-edge="continuing" data-lane={lane} key={`continuing:${lane}`} d={continuingPath(lane)} style={laneStyle(lane)} />)}
      {row.parentLanes.map((parentLane, index) => {
        const edgeLane = index === 0 ? row.lane : parentLane;
        return <path className={`graph-line graph-line--parent graph-line--parent-${index}`} data-edge="parent" data-lane={edgeLane} data-curved={parentLane !== row.lane ? "true" : "false"} key={`${row.oid}:${parentLane}:${index}`} d={parentPath(row.lane, parentLane, index)} style={laneStyle(edgeLane)} />;
      })}
      <circle className={`graph-node${isHead ? " graph-node--head" : ""}`} data-lane={row.lane} cx={laneX(row.lane)} cy={NODE_Y} r="5" style={laneStyle(row.lane)} />
    </svg>
  </span>;
}
