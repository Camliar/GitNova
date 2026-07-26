import { useEffect, useRef, useState } from "react";
import type { GitHubCommitFileDiff, GitHubPullRequestCommit, GitHubPullRequestCommitFiles, GitHubSquashTrace } from "@gitnova/protocol";
import { asDesktopError, type DesktopError } from "./core";
import { FileDiffView } from "./FileDiffView";
import { getGitHubPullRequestCommitFileDiff, getGitHubPullRequestCommitFiles } from "./github";

type FilesState = { kind: "idle" } | { kind: "loading"; commit: GitHubPullRequestCommit } | { kind: "ready"; value: GitHubPullRequestCommitFiles } | { kind: "error"; commit: GitHubPullRequestCommit; error: DesktopError };
type FileState = { kind: "idle" } | { kind: "loading"; path: string } | { kind: "ready"; path: string; value: GitHubCommitFileDiff } | { kind: "error"; path: string; error: DesktopError };

const relationshipLabel = { notMerged: "Not merged", originalCommit: "Original retained", mergeCommit: "Merge commit", squashCandidate: "Squash candidate", unresolved: "Unresolved" } as const;

export function HistorySquashTrace({ trace }: { trace: GitHubSquashTrace }) {
  const [files, setFiles] = useState<FilesState>({ kind: "idle" });
  const [file, setFile] = useState<FileState>({ kind: "idle" });
  const [tab, setTab] = useState<"commit" | "changes">("commit");
  const filesSerial = useRef(0);
  const fileSerial = useRef(0);
  useEffect(() => () => { filesSerial.current += 1; fileSerial.current += 1; }, []);
  const selectedCommit = files.kind === "idle" ? null : files.kind === "ready" ? files.value.commit : files.commit;

  async function selectCommit(commit: GitHubPullRequestCommit) {
    const current = ++filesSerial.current;
    fileSerial.current += 1;
    setFile({ kind: "idle" });
    setTab("commit");
    setFiles({ kind: "loading", commit });
    try {
      const value = await getGitHubPullRequestCommitFiles(trace.pullRequest.number, commit.oid, trace.pullRequest.nameWithOwner);
      if (current === filesSerial.current) setFiles({ kind: "ready", value });
    } catch (error) {
      if (current === filesSerial.current) setFiles({ kind: "error", commit, error: asDesktopError(error) });
    }
  }

  async function selectFile(path: string) {
    if (files.kind !== "ready") return;
    const current = ++fileSerial.current;
    setFile({ kind: "loading", path });
    try {
      const value = await getGitHubPullRequestCommitFileDiff(trace.pullRequest.number, files.value.commit.oid, path, trace.pullRequest.nameWithOwner);
      if (current === fileSerial.current) setFile({ kind: "ready", path, value });
    } catch (error) {
      if (current === fileSerial.current) setFile({ kind: "error", path, error: asDesktopError(error) });
    }
  }

  return <section className="history-squash" aria-label="Squash Trace original commits">
    <header className="history-squash__relationship">
      <span>PR #{trace.pullRequest.number}</span>
      <strong>{trace.pullRequest.title}</strong>
      <b>{relationshipLabel[trace.relationship.classification]} · {trace.relationship.confidence}</b>
      <code>{trace.pullRequest.commits.length} originals → {trace.relationship.mergeCommitOid?.slice(0, 8) ?? "unknown"}</code>
    </header>
    <div className="history-squash__layout">
      <aside className="original-commits" aria-label="Original commits">
        <header><strong>Original commits</strong><span>{trace.pullRequest.commits.length}</span></header>
        {trace.pullRequest.commits.length === 0 ? <p className="empty-state">No original commits returned.</p> : <ol>{trace.pullRequest.commits.map((commit, index) => <li key={commit.oid}><button type="button" className={selectedCommit?.oid === commit.oid ? "is-selected" : ""} onClick={() => void selectCommit(commit)}><span>{index + 1}</span><strong>{commit.summary || "(no commit message)"}</strong><code>{commit.oid.slice(0, 8)}</code></button></li>)}</ol>}
      </aside>
      <section className="original-detail">
        {!selectedCommit && <div className="pane-placeholder"><strong>Select an original commit</strong><span>Its metadata and changed files will appear here.</span></div>}
        {selectedCommit && <>
          <div className="original-detail__tabs" role="tablist" aria-label="Original commit detail views"><button type="button" role="tab" aria-selected={tab === "commit"} className={tab === "commit" ? "is-active" : ""} onClick={() => setTab("commit")}>Commit</button><button type="button" role="tab" aria-selected={tab === "changes"} className={tab === "changes" ? "is-active" : ""} onClick={() => setTab("changes")}>Changes{files.kind === "ready" ? ` · ${files.value.files.length}` : ""}</button></div>
          {tab === "commit" && <div className="original-metadata"><h3>{selectedCommit.summary}</h3><dl><div><dt>OID</dt><dd>{selectedCommit.oid}</dd></div><div><dt>Author</dt><dd>{selectedCommit.author.name} &lt;{selectedCommit.author.email}&gt; · {selectedCommit.author.timestamp}</dd></div><div><dt>Parents</dt><dd>{selectedCommit.parents.join(" · ") || "None"}</dd></div></dl><pre>{selectedCommit.message}</pre></div>}
          {tab === "changes" && files.kind === "loading" && <p className="empty-state" role="status">Loading original commit files…</p>}
          {tab === "changes" && files.kind === "error" && <div className="diff-error"><p role="alert">{files.error.message}. Original commit list remains available.</p><button type="button" onClick={() => void selectCommit(files.commit)}>Retry original commit</button></div>}
          {tab === "changes" && files.kind === "ready" && <div className="original-changes"><aside className="commit-file-browser" aria-label="Original commit changed files"><header><strong>Changed files</strong><span>{files.value.files.length}</span></header><ul className="commit-files">{files.value.files.map((changed) => <li key={changed.newPath}><button type="button" className={(file.kind !== "idle" && file.path === changed.newPath) ? "is-selected" : ""} onClick={() => void selectFile(changed.newPath)}><b className={`commit-file-status commit-file-status--${changed.status}`}>{changed.status.slice(0, 1).toUpperCase()}</b><span><strong>{changed.newPath}</strong>{changed.oldPath !== changed.newPath && <small>from {changed.oldPath}</small>}</span></button></li>)}</ul></aside><section className="commit-file-content">{file.kind === "idle" && <div className="pane-placeholder"><strong>Select a changed file</strong><span>Patch content stays in Core until selected.</span></div>}{file.kind === "loading" && <p className="empty-state" role="status">Loading {file.path}…</p>}{file.kind === "error" && <div className="diff-error"><p role="alert">{file.error.message}. Changed-file list remains available.</p><button type="button" onClick={() => void selectFile(file.path)}>Retry file diff</button></div>}{file.kind === "ready" && <section className="commit-file-diff" aria-label={`Original commit diff for ${file.value.newPath}`}><h3>{file.value.oldPath === file.value.newPath ? file.value.newPath : `${file.value.oldPath} → ${file.value.newPath}`}</h3><p>Provider: {file.value.status} · +{file.value.additions} −{file.value.deletions}</p>{file.value.patchState === "unavailable" ? <p className="empty-state">GitHub did not provide a patch for this file.</p> : <FileDiffView diff={{ oldPath: file.value.oldPath, newPath: file.value.newPath, isBinary: false, hunks: file.value.hunks }} />}</section>}</section></div>}
        </>}
      </section>
    </div>
  </section>;
}
