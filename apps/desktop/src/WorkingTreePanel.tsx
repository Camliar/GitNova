import type { DiffScope, FileStatus, StatusEntry, WorkingTreeStatus } from "@gitnova/protocol";
import type { DesktopError } from "./core";

export type WorkingTreeState =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "ready"; status: WorkingTreeStatus }
  | { kind: "error"; error: DesktopError };

const statusLabel: Record<FileStatus, string> = {
  unmodified: "Unmodified",
  modified: "Modified",
  added: "Added",
  deleted: "Deleted",
  renamed: "Renamed",
  copied: "Copied",
  unmerged: "Conflict",
  untracked: "Untracked",
  typeChanged: "Type changed",
  unknown: "Unknown",
};

function ChangeBadges({ entry, disabled, onDiff }: { entry: StatusEntry; disabled: boolean; onDiff: (path: string, scope: DiffScope) => void }) {
  const stagedDiff = entry.indexStatus !== "unmodified" && entry.indexStatus !== "untracked";
  const workingDiff = entry.worktreeStatus !== "unmodified" && entry.worktreeStatus !== "untracked";
  return (
    <span className="change-badges">
      {entry.indexStatus !== "unmodified" && (
        <span className="change-badge-group">
          <span className={`change-badge change-badge--${entry.indexStatus}`}>Staged · {statusLabel[entry.indexStatus]}</span>
          {stagedDiff && <button type="button" disabled={disabled} aria-label={`View staged diff for ${entry.path}`} onClick={() => onDiff(entry.path, "staged")}>View</button>}
        </span>
      )}
      {entry.worktreeStatus !== "unmodified" && (
        <span className="change-badge-group">
          <span className={`change-badge change-badge--${entry.worktreeStatus}`}>Working · {statusLabel[entry.worktreeStatus]}</span>
          {workingDiff && <button type="button" disabled={disabled} aria-label={`View working diff for ${entry.path}`} onClick={() => onDiff(entry.path, "workingTree")}>View</button>}
        </span>
      )}
    </span>
  );
}

export function WorkingTreePanel({ state, diffLoading, onRefresh, onDiff }: { state: WorkingTreeState; diffLoading: boolean; onRefresh: () => void; onDiff: (path: string, scope: DiffScope) => void }) {
  const status = state.kind === "ready" ? state.status : null;
  const branchTitle = status
    ? status.branch.oid === null
      ? `${status.branch.head ?? "Branch"} · Unborn`
      : status.branch.head ?? "Detached HEAD"
    : "Working tree status";
  return (
    <section className="working-tree" aria-labelledby="working-tree-title" aria-busy={state.kind === "loading"}>
      <div className="working-tree__header">
        <div>
          <p className="eyebrow">Working tree</p>
          <h2 id="working-tree-title">{branchTitle}</h2>
          {status?.branch.upstream && (
            <p className="branch-meta">
              {status.branch.upstream} · {status.branch.ahead} ahead · {status.branch.behind} behind
            </p>
          )}
        </div>
        <button type="button" className="button-secondary" disabled={state.kind === "loading"} onClick={onRefresh}>
          {state.kind === "loading" ? "Refreshing…" : "Refresh"}
        </button>
      </div>
      {state.kind === "error" && <p className="status-error" role="alert">{state.error.message}. The repository remains open.</p>}
      {state.kind === "loading" && <p className="empty-state" role="status">Reading status from GitNova Core…</p>}
      {status && status.entries.length === 0 && <p className="empty-state">Working tree clean</p>}
      {status && status.entries.length > 0 && (
        <ol className="change-list" aria-label="Working tree changes">
          {status.entries.map((entry, index) => (
            <li key={`${entry.path}:${index}`}>
              <div className="change-path">
                <strong>{entry.path}</strong>
                {entry.originalPath && <span>from {entry.originalPath}</span>}
              </div>
              <ChangeBadges entry={entry} disabled={diffLoading} onDiff={onDiff} />
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}
