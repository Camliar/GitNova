import type { DiffScope, FileStatus, StatusEntry, WorkingTreeStatus } from "@gitnova/protocol";
import type { DesktopError } from "./core";
import type { DiffSelection } from "./DiffPanel";

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

function ChangeGroup({ title, entries, scope, disabled, selection, onDiff }: { title: string; entries: StatusEntry[]; scope: DiffScope; disabled: boolean; selection: DiffSelection | null; onDiff: (path: string, scope: DiffScope) => void }) {
  return <section className="change-group" aria-labelledby={`change-group-${scope}`}>
    <header><h2 id={`change-group-${scope}`}>{title}</h2><span>{entries.length}</span></header>
    {entries.length === 0 ? <p className="change-group__empty">No {title.toLowerCase()} paths</p> : <ol className="change-list">
      {entries.map((entry, index) => {
        const fileStatus = scope === "staged" ? entry.indexStatus : entry.worktreeStatus;
        const unavailable = scope === "workingTree" && fileStatus === "untracked";
        const selected = selection?.path === entry.path && selection.scope === scope;
        return <li key={`${scope}:${entry.path}:${index}`} className={selected ? "is-selected" : ""}>
          <button type="button" className="change-path" disabled={disabled || unavailable} aria-label={`${scope === "staged" ? "View staged" : "View working"} diff for ${entry.path}`} onClick={() => onDiff(entry.path, scope)}>
            <span className={`change-status change-status--${fileStatus}`} aria-hidden="true">{fileStatus === "added" || fileStatus === "untracked" ? "+" : fileStatus === "deleted" ? "−" : fileStatus === "renamed" ? "R" : "M"}</span>
            <span className="change-path__label"><strong>{entry.path}</strong>{entry.originalPath && <small>from {entry.originalPath}</small>}</span>
            <small>{statusLabel[fileStatus]}</small>
          </button>
        </li>;
      })}
    </ol>}
  </section>;
}

export function WorkingTreePanel({ state, diffLoading, selection, onDiff }: { state: WorkingTreeState; diffLoading: boolean; selection: DiffSelection | null; onDiff: (path: string, scope: DiffScope) => void }) {
  const status = state.kind === "ready" ? state.status : null;
  return (
    <section className="working-tree" aria-label="Working tree changes" aria-busy={state.kind === "loading"}>
      {status?.branch.upstream && <p className="working-tree__upstream">{status.branch.upstream} · {status.branch.ahead} ahead · {status.branch.behind} behind</p>}
      {state.kind === "error" && <p className="status-error" role="alert">{state.error.message}. The repository remains open.</p>}
      {state.kind === "loading" && <p className="empty-state" role="status">Reading status from GitNova Core…</p>}
      {status && status.entries.length === 0 && <p className="empty-state">Working tree clean</p>}
      {status && status.entries.length > 0 && (
        <div className="change-groups" aria-label="Changed files">
          <ChangeGroup title="Unstaged" entries={status.entries.filter((entry) => entry.worktreeStatus !== "unmodified")} scope="workingTree" disabled={diffLoading} selection={selection} onDiff={onDiff} />
          <ChangeGroup title="Staged" entries={status.entries.filter((entry) => entry.indexStatus !== "unmodified" && entry.indexStatus !== "untracked")} scope="staged" disabled={diffLoading} selection={selection} onDiff={onDiff} />
        </div>
      )}
    </section>
  );
}
