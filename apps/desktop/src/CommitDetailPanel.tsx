import { useState } from "react";
import type { CommitDiff, CommitSummary } from "@gitnova/protocol";
import type { DesktopError } from "./core";
import { FileDiffView } from "./FileDiffView";

export interface CommitSelection { commit: CommitSummary; parentOid?: string }
export type CommitDetailState =
  | { kind: "idle" }
  | { kind: "choosingParent"; commit: CommitSummary }
  | { kind: "loading"; selection: CommitSelection }
  | { kind: "ready"; selection: CommitSelection; diff: CommitDiff }
  | { kind: "error"; selection: CommitSelection; error: DesktopError };

const shortOid = (oid: string) => oid.slice(0, 8);

export function CommitDetailPanel({ state, onChooseParent, onRetry, onClose }: {
  state: Exclude<CommitDetailState, { kind: "idle" }>;
  onChooseParent: (parentOid: string) => void;
  onRetry: () => void;
  onClose: () => void;
}) {
  const [selectedFile, setSelectedFile] = useState(0);
  const commit = state.kind === "ready" ? state.diff.commit : state.kind === "choosingParent" ? state.commit : state.selection.commit;
  const file = state.kind === "ready" ? state.diff.files[selectedFile] : undefined;
  return (
    <section className="commit-detail" aria-labelledby="commit-detail-title" aria-busy={state.kind === "loading"}>
      <header className="commit-detail__header">
        <div><p className="eyebrow">Commit detail</p><h2 id="commit-detail-title">{commit.summary || "(no commit message)"}</h2></div>
        <button type="button" className="button-secondary" onClick={onClose}>Close commit</button>
      </header>
      <dl className="commit-metadata">
        <div><dt>Commit</dt><dd><code>{commit.oid}</code></dd></div>
        <div><dt>Author</dt><dd>{commit.author.name} &lt;{commit.author.email}&gt; · {commit.author.timestamp}</dd></div>
        <div><dt>Committer</dt><dd>{commit.committer.name} &lt;{commit.committer.email}&gt; · {commit.committer.timestamp}</dd></div>
        <div><dt>Parents</dt><dd>{commit.parents.length ? commit.parents.join(" · ") : "Root commit (empty tree)"}</dd></div>
        {state.kind === "ready" && <div><dt>Compared with</dt><dd>{state.diff.parentOid ?? "Empty tree"}</dd></div>}
      </dl>
      <div className="commit-message"><h3>Message</h3><pre>{commit.message}</pre></div>
      {state.kind === "choosingParent" && (
        <div className="parent-choice">
          <p>This merge has multiple parents. Choose the parent edge to compare.</p>
          <span>{commit.parents.map((parent) => <button type="button" key={parent} onClick={() => onChooseParent(parent)}>Compare parent {shortOid(parent)}</button>)}</span>
        </div>
      )}
      {state.kind === "loading" && <p className="empty-state" role="status">Reading commit diff from GitNova Core…</p>}
      {state.kind === "error" && <div className="diff-error"><p role="alert">{state.error.message}. Commit history is still available.</p><button type="button" onClick={onRetry}>Retry commit diff</button></div>}
      {state.kind === "ready" && state.diff.files.length === 0 && <p className="empty-state">No changed files in this comparison.</p>}
      {state.kind === "ready" && state.diff.files.length > 0 && (
        <>
          <ul className="commit-files" aria-label="Changed files">
            {state.diff.files.map((changedFile, index) => (
              <li key={`${changedFile.oldPath}:${changedFile.newPath}:${index}`}>
                <button type="button" className={index === selectedFile ? "is-selected" : ""} onClick={() => setSelectedFile(index)}>
                  <strong>{changedFile.newPath}</strong>
                  {changedFile.oldPath !== changedFile.newPath && <span>from {changedFile.oldPath}</span>}
                </button>
              </li>
            ))}
          </ul>
          {file && <section className="commit-file-diff" aria-label={`Commit diff for ${file.newPath}`}>
            <h3>{file.oldPath === file.newPath ? file.newPath : `${file.oldPath} → ${file.newPath}`}</h3>
            <FileDiffView diff={file} />
          </section>}
        </>
      )}
    </section>
  );
}
