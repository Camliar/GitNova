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

function ChangeBadges({ entry, disabled, selection, onDiff }: { entry: StatusEntry; disabled: boolean; selection: DiffSelection | null; onDiff: (path: string, scope: DiffScope) => void }) {
  const stagedDiff = entry.indexStatus !== "unmodified" && entry.indexStatus !== "untracked";
  const workingDiff = entry.worktreeStatus !== "unmodified" && entry.worktreeStatus !== "untracked";
  return (
    <span className="change-badges">
      {entry.indexStatus !== "unmodified" && (
        stagedDiff
          ? <button type="button" className={`change-badge change-badge--${entry.indexStatus}${selection?.path === entry.path && selection.scope === "staged" ? " is-selected" : ""}`} disabled={disabled} aria-label={`View staged diff for ${entry.path}`} onClick={() => onDiff(entry.path, "staged")}>Staged · {statusLabel[entry.indexStatus]}</button>
          : <span className={`change-badge change-badge--${entry.indexStatus}`}>Staged · {statusLabel[entry.indexStatus]}</span>
      )}
      {entry.worktreeStatus !== "unmodified" && (
        workingDiff
          ? <button type="button" className={`change-badge change-badge--${entry.worktreeStatus}${selection?.path === entry.path && selection.scope === "workingTree" ? " is-selected" : ""}`} disabled={disabled} aria-label={`View working diff for ${entry.path}`} onClick={() => onDiff(entry.path, "workingTree")}>Working · {statusLabel[entry.worktreeStatus]}</button>
          : <span className={`change-badge change-badge--${entry.worktreeStatus}`}>Working · {statusLabel[entry.worktreeStatus]}</span>
      )}
    </span>
  );
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
        <ol className="change-list" aria-label="Changed files">
          {status.entries.map((entry, index) => (
            <li key={`${entry.path}:${index}`} className={selection?.path === entry.path ? "is-selected" : ""}>
              <button type="button" className="change-path" disabled={diffLoading || (entry.worktreeStatus === "untracked" && (entry.indexStatus === "unmodified" || entry.indexStatus === "untracked"))} onClick={() => onDiff(entry.path, entry.worktreeStatus !== "unmodified" && entry.worktreeStatus !== "untracked" ? "workingTree" : "staged")}>
                <strong>{entry.path}</strong>
                {entry.originalPath && <span>from {entry.originalPath}</span>}
              </button>
              <ChangeBadges entry={entry} disabled={diffLoading} selection={selection} onDiff={onDiff} />
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}
