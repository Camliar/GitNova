import type { CSSProperties } from "react";
import type { CommitGraphNode } from "@gitnova/protocol";
import type { DesktopError } from "./core";
import { CommitGraph, projectGraphRows } from "./CommitGraph";

export type HistoryState =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "error"; error: DesktopError }
  | {
      kind: "ready";
      nodes: CommitGraphNode[];
      nextCursor: string | null;
      more: { kind: "idle" } | { kind: "loading" } | { kind: "error"; error: DesktopError };
    };

function shortOid(oid: string) {
  return oid.slice(0, 8);
}

function formatTimestamp(value: string) {
  const timestamp = new Date(value);
  if (Number.isNaN(timestamp.valueOf())) return value;
  const now = new Date();
  const start = new Date(now.getFullYear(), now.getMonth(), now.getDate()).valueOf();
  const day = new Date(timestamp.getFullYear(), timestamp.getMonth(), timestamp.getDate()).valueOf();
  const time = timestamp.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  if (day === start) return `Today ${time}`;
  if (day === start - 86_400_000) return `Yesterday ${time}`;
  return timestamp.toLocaleDateString([], { month: "short", day: "numeric", year: timestamp.getFullYear() === now.getFullYear() ? undefined : "numeric" });
}

export function HistoryPanel({ state, commitLoading, onRetry, onLoadMore, onSelectCommit }: { state: HistoryState; commitLoading: boolean; onRetry: () => void; onLoadMore: () => void; onSelectCommit: (commit: CommitGraphNode["commit"]) => void }) {
  const graphRows = state.kind === "ready" ? projectGraphRows(state.nodes) : [];
  const graphStyle = { "--graph-column": `${Math.max(1, ...graphRows.map((row) => row.laneCount)) * 18}px` } as CSSProperties;
  return (
    <section className="history-panel" aria-label="Repository timeline" aria-busy={state.kind === "loading" || (state.kind === "ready" && state.more.kind === "loading")}>
      {state.kind === "loading" && <p className="empty-state" role="status">Reading commit graph from GitNova Core…</p>}
      {state.kind === "error" && (
        <div className="history-error">
          <p role="alert">{state.error.message}. The repository remains open.</p>
          <button type="button" onClick={onRetry}>Retry history</button>
        </div>
      )}
      {state.kind === "ready" && state.nodes.length === 0 && <p className="empty-state">No commits yet</p>}
      {state.kind === "ready" && state.nodes.length > 0 && (
        <ol className="commit-list" aria-label="Commit history" style={graphStyle}>
          {state.nodes.map((node, index) => (
            <li key={node.commit.oid}>
              <CommitGraph row={graphRows[index]} isHead={node.isHead} />
              <button type="button" className="commit-main commit-row" aria-label={`View commit ${shortOid(node.commit.oid)}`} disabled={commitLoading} onClick={() => onSelectCommit(node.commit)}>
                <div className="commit-summary">
                  <strong>{node.commit.summary || "(no commit message)"}</strong>
                  {(node.isHead || node.references.length > 0) && (
                    <span className="commit-decorations">
                      {node.isHead && <span className="decoration decoration--head">HEAD</span>}
                      {node.references.map((reference) => <span className={`decoration decoration--${reference.kind}`} key={reference.fullName}>{reference.name}</span>)}
                    </span>
                  )}
                </div>
                <span className="commit-author">{node.commit.author.name}{node.commit.parents.length > 1 ? ` · Merge (${node.commit.parents.length} parents)` : ""}</span>
                <code className="commit-oid">{shortOid(node.commit.oid)}</code>
                <time dateTime={node.commit.author.timestamp}>{formatTimestamp(node.commit.author.timestamp)}</time>
              </button>
            </li>
          ))}
        </ol>
      )}
      {state.kind === "ready" && state.more.kind === "error" && <p className="history-more-error" role="alert">{state.more.error.message}. Loaded commits were kept.</p>}
      {state.kind === "ready" && state.nextCursor && (
        <button type="button" className="history-more" disabled={state.more.kind === "loading"} onClick={onLoadMore}>
          {state.more.kind === "loading" ? "Loading more…" : state.more.kind === "error" ? "Retry load more" : "Load more"}
        </button>
      )}
    </section>
  );
}
