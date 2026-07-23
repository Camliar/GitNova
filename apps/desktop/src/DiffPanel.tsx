import type { DiffScope, FileDiff } from "@gitnova/protocol";
import type { DesktopError } from "./core";
import { FileDiffView } from "./FileDiffView";

export interface DiffSelection { path: string; scope: DiffScope }
export type DiffState =
  | { kind: "idle" }
  | { kind: "loading"; selection: DiffSelection }
  | { kind: "ready"; selection: DiffSelection; diff: FileDiff }
  | { kind: "error"; selection: DiffSelection; error: DesktopError };

export function DiffPanel({ state, onRetry, onClose }: { state: Exclude<DiffState, { kind: "idle" }>; onRetry: () => void; onClose: () => void }) {
  const diff = state.kind === "ready" ? state.diff : null;
  return (
    <section className="diff-panel" aria-labelledby="diff-title" aria-busy={state.kind === "loading"}>
      <header className="diff-panel__header">
        <div>
          <p className="eyebrow">{state.selection.scope === "staged" ? "Staged diff" : "Working tree diff"}</p>
          <h2 id="diff-title">{state.selection.path}</h2>
          {diff && diff.oldPath !== diff.newPath && <p className="branch-meta">{diff.oldPath} → {diff.newPath}</p>}
        </div>
        <button type="button" className="button-secondary" onClick={onClose}>Close diff</button>
      </header>
      {state.kind === "loading" && <p className="empty-state" role="status">Reading structured diff from GitNova Core…</p>}
      {state.kind === "error" && (
        <div className="diff-error">
          <p role="alert">{state.error.message}. Working tree status is still available.</p>
          <button type="button" onClick={onRetry}>Retry diff</button>
        </div>
      )}
      {diff && <FileDiffView diff={diff} emptyMessage="No changes in this scope." />}
    </section>
  );
}
