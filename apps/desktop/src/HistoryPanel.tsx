import type { CommitGraphNode } from "@gitnova/protocol";
import type { DesktopError } from "./core";

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
  return Number.isNaN(timestamp.valueOf()) ? value : timestamp.toLocaleString();
}

export function HistoryPanel({ state, commitLoading, onRetry, onLoadMore, onSelectCommit }: { state: HistoryState; commitLoading: boolean; onRetry: () => void; onLoadMore: () => void; onSelectCommit: (commit: CommitGraphNode["commit"]) => void }) {
  return (
    <section className="history-panel" aria-labelledby="history-title" aria-busy={state.kind === "loading" || (state.kind === "ready" && state.more.kind === "loading")}>
      <header>
        <p className="eyebrow">Commit history</p>
        <h2 id="history-title">Repository timeline</h2>
      </header>
      {state.kind === "loading" && <p className="empty-state" role="status">Reading commit graph from GitNova Core…</p>}
      {state.kind === "error" && (
        <div className="history-error">
          <p role="alert">{state.error.message}. The repository remains open.</p>
          <button type="button" onClick={onRetry}>Retry history</button>
        </div>
      )}
      {state.kind === "ready" && state.nodes.length === 0 && <p className="empty-state">No commits yet</p>}
      {state.kind === "ready" && state.nodes.length > 0 && (
        <ol className="commit-list" aria-label="Commit history">
          {state.nodes.map((node) => (
            <li key={node.commit.oid}>
              <span className={`commit-dot${node.isHead ? " commit-dot--head" : ""}`} aria-hidden="true" />
              <div className="commit-main">
                <div className="commit-summary">
                  <strong>{node.commit.summary || "(no commit message)"}</strong>
                  <code>{shortOid(node.commit.oid)}</code>
                </div>
                <p>{node.commit.author.name} · {formatTimestamp(node.commit.author.timestamp)}{node.commit.parents.length > 1 ? ` · Merge (${node.commit.parents.length} parents)` : ""}</p>
                {(node.isHead || node.references.length > 0) && (
                  <span className="commit-decorations">
                    {node.isHead && <span className="decoration decoration--head">HEAD</span>}
                    {node.references.map((reference) => <span className={`decoration decoration--${reference.kind}`} key={reference.fullName}>{reference.name}</span>)}
                  </span>
                )}
                <button type="button" className="commit-view" disabled={commitLoading} onClick={() => onSelectCommit(node.commit)}>View commit {shortOid(node.commit.oid)}</button>
              </div>
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
