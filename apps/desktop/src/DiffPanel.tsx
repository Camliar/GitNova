import type { DiffScope, FileDiff } from "@gitnova/protocol";
import type { DesktopError } from "./core";

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
      {diff?.isBinary && <p className="empty-state">Binary file changed. Content is not returned by Core.</p>}
      {diff && !diff.isBinary && diff.hunks.length === 0 && <p className="empty-state">No changes in this scope.</p>}
      {diff && !diff.isBinary && diff.hunks.map((hunk, hunkIndex) => (
        <section className="diff-hunk" key={`${hunk.oldStart}:${hunk.newStart}:${hunkIndex}`} aria-label={`Diff hunk ${hunkIndex + 1}`}>
          <h3>{`@@ -${hunk.oldStart},${hunk.oldLines} +${hunk.newStart},${hunk.newLines} @@${hunk.header ? ` ${hunk.header}` : ""}`}</h3>
          <ol>
            {hunk.lines.map((line, lineIndex) => (
              <li className={`diff-line diff-line--${line.kind}`} key={`${lineIndex}:${line.oldLine}:${line.newLine}`}>
                <span aria-label="Old line">{line.oldLine ?? ""}</span>
                <span aria-label="New line">{line.newLine ?? ""}</span>
                <span className="diff-line__prefix" aria-hidden="true">{line.kind === "addition" ? "+" : line.kind === "deletion" ? "−" : " "}</span>
                <code>{line.content}</code>
              </li>
            ))}
          </ol>
        </section>
      ))}
    </section>
  );
}
