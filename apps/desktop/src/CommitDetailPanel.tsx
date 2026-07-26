import type { CommitFiles, CommitSummary, FileDiff } from "@gitnova/protocol";
import type { DesktopError } from "./core";
import { FileDiffView } from "./FileDiffView";

export interface CommitSelection { commit: CommitSummary; parentOid?: string }
export type CommitDetailState =
  | { kind: "idle" }
  | { kind: "choosingParent"; commit: CommitSummary }
  | { kind: "loading"; selection: CommitSelection }
  | { kind: "ready"; selection: CommitSelection; files: CommitFiles; legacyDiffs?: FileDiff[] }
  | { kind: "error"; selection: CommitSelection; error: DesktopError };

export type CommitFileDiffState =
  | { kind: "idle" }
  | { kind: "loading"; path: string }
  | { kind: "ready"; path: string; diff: FileDiff }
  | { kind: "error"; path: string; error: DesktopError };

const shortOid = (oid: string) => oid.slice(0, 8);

export function CommitDetailPanel({ state, fileDiff, mode, onChooseParent, onSelectFile, onRetry, onRetryFile, onClose }: {
  state: Exclude<CommitDetailState, { kind: "idle" }>;
  fileDiff: CommitFileDiffState;
  mode: "commit" | "changes";
  onChooseParent: (parentOid: string) => void;
  onSelectFile: (path: string) => void;
  onRetry: () => void;
  onRetryFile: () => void;
  onClose: () => void;
}) {
  const commit = state.kind === "ready" ? state.files.commit : state.kind === "choosingParent" ? state.commit : state.selection.commit;
  const selectedPath = fileDiff.kind === "idle" ? null : fileDiff.path;
  return (
    <section className="commit-detail" aria-labelledby="commit-detail-title" aria-busy={state.kind === "loading" || fileDiff.kind === "loading"}>
      <header className="commit-detail__header">
        <div><h2 id="commit-detail-title">{commit.summary || "(no commit message)"}</h2></div>
        <button type="button" className="button-secondary" onClick={onClose}>Close commit</button>
      </header>
      {mode === "commit" && <>
        <dl className="commit-metadata">
          <div><dt>Commit</dt><dd><code>{commit.oid}</code></dd></div>
          <div><dt>Author</dt><dd>{commit.author.name} &lt;{commit.author.email}&gt; · {commit.author.timestamp}</dd></div>
          <div><dt>Committer</dt><dd>{commit.committer.name} &lt;{commit.committer.email}&gt; · {commit.committer.timestamp}</dd></div>
          <div><dt>Parents</dt><dd>{commit.parents.length ? commit.parents.join(" · ") : "Root commit (empty tree)"}</dd></div>
          {state.kind === "ready" && <div><dt>Compared with</dt><dd>{state.files.parentOid ?? "Empty tree"}</dd></div>}
        </dl>
        <div className="commit-message"><h3>Message</h3><pre>{commit.message}</pre></div>
      </>}
      {state.kind === "choosingParent" && (
        <div className="parent-choice">
          <p>This merge has multiple parents. Choose the parent edge to compare.</p>
          <span>{commit.parents.map((parent) => <button type="button" key={parent} onClick={() => onChooseParent(parent)}>Compare parent {shortOid(parent)}</button>)}</span>
        </div>
      )}
      {state.kind === "loading" && <p className="empty-state" role="status">Reading changed files from GitNova Core…</p>}
      {state.kind === "error" && <div className="diff-error"><p role="alert">{state.error.message}. Commit history is still available.</p><button type="button" onClick={onRetry}>Retry changed files</button></div>}
      {mode === "changes" && state.kind === "ready" && state.files.files.length === 0 && <p className="empty-state">Empty commit: its tree is identical to the selected comparison baseline.</p>}
      {mode === "changes" && state.kind === "ready" && state.files.files.length > 0 && (
        <div className="commit-changes-layout">
          <aside className="commit-file-browser" aria-label="Changed files">
            <header><strong>Changed files</strong><span>{state.files.files.length}</span></header>
            <ul className="commit-files">
              {state.files.files.map((changedFile) => (
                <li key={`${changedFile.oldPath}:${changedFile.newPath}`}>
                  <button type="button" className={changedFile.newPath === selectedPath ? "is-selected" : ""} onClick={() => onSelectFile(changedFile.newPath)}>
                    <b className={`commit-file-status commit-file-status--${changedFile.status}`}>{changedFile.status.slice(0, 1).toUpperCase()}</b>
                    <span><strong>{changedFile.newPath}</strong>{changedFile.oldPath !== changedFile.newPath && <small>from {changedFile.oldPath}</small>}</span>
                  </button>
                </li>
              ))}
            </ul>
          </aside>
          <section className="commit-file-content">
            {fileDiff.kind === "idle" && <div className="pane-placeholder"><strong>Select a changed file</strong><span>Its line-level diff will be loaded on demand.</span></div>}
            {fileDiff.kind === "loading" && <p className="empty-state" role="status">Reading {fileDiff.path}…</p>}
            {fileDiff.kind === "error" && <div className="diff-error"><p role="alert">{fileDiff.error.message}. The changed-file list is still available.</p><button type="button" onClick={onRetryFile}>Retry file diff</button></div>}
            {fileDiff.kind === "ready" && <section className="commit-file-diff" aria-label={`Commit diff for ${fileDiff.diff.newPath}`}>
              <h3>{fileDiff.diff.oldPath === fileDiff.diff.newPath ? fileDiff.diff.newPath : `${fileDiff.diff.oldPath} → ${fileDiff.diff.newPath}`}</h3>
              <FileDiffView diff={fileDiff.diff} />
            </section>}
          </section>
        </div>
      )}
    </section>
  );
}
