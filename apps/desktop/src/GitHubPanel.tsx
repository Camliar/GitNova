import { useEffect, useRef, useState } from "react";
import type { GitHubPullRequest, GitHubPullRequestCommitDiff, GitHubRepository, GitHubSquashTrace, SquashTraceEvidence } from "@gitnova/protocol";
import { asDesktopError, type DesktopError } from "./core";
import { getGitHubPullRequest, getGitHubPullRequestCommitDiff, getGitHubRepository, getGitHubSquashTrace } from "./github";
import { FileDiffView } from "./FileDiffView";

type RepositoryState = { kind: "idle" } | { kind: "loading" } | { kind: "ready"; repository: GitHubRepository } | { kind: "error"; error: DesktopError };
type PullRequestState = { kind: "idle" } | { kind: "loading"; number: number } | { kind: "ready"; pullRequest: GitHubPullRequest } | { kind: "error"; number: number; error: DesktopError };

export function GitHubPanel() {
  const [repository, setRepository] = useState<RepositoryState>({ kind: "idle" });
  const [pullRequest, setPullRequest] = useState<PullRequestState>({ kind: "idle" });
  const [number, setNumber] = useState("");
  const active = useRef(true);
  const request = useRef(0);
  useEffect(() => () => { active.current = false; request.current += 1; }, []);

  async function connect() {
    const current = ++request.current;
    setRepository({ kind: "loading" });
    setPullRequest({ kind: "idle" });
    try {
      const result = await getGitHubRepository();
      if (active.current && current === request.current) setRepository({ kind: "ready", repository: result });
    } catch (error) {
      if (active.current && current === request.current) setRepository({ kind: "error", error: asDesktopError(error) });
    }
  }

  async function loadPullRequest(requestedNumber?: number) {
    if (repository.kind !== "ready") return;
    const parsed = requestedNumber ?? Number(number);
    if (!Number.isSafeInteger(parsed) || parsed <= 0) {
      setPullRequest({ kind: "error", number: parsed, error: { code: "desktop.invalid_pr_number", message: "Enter a positive pull request number", retryable: false } });
      return;
    }
    const current = ++request.current;
    setPullRequest({ kind: "loading", number: parsed });
    try {
      const result = await getGitHubPullRequest(parsed, repository.repository.nameWithOwner);
      if (active.current && current === request.current) setPullRequest({ kind: "ready", pullRequest: result });
    } catch (error) {
      if (active.current && current === request.current) setPullRequest({ kind: "error", number: parsed, error: asDesktopError(error) });
    }
  }

  return (
    <section className="github-panel" aria-labelledby="github-title" aria-busy={repository.kind === "loading" || pullRequest.kind === "loading"}>
      <header><div><p className="eyebrow">GitHub provider</p><h2 id="github-title">Pull request history</h2></div><span className={`network-state network-state--${repository.kind === "idle" ? "off" : "triggered"}`}>Network: {repository.kind === "idle" ? "off" : "user-triggered"}</span></header>
      {repository.kind === "idle" && <><p className="empty-state">No network request has been made. Connect sends only this repository's GitHub identity through Core using existing gh configuration; it does not send source files or diffs.</p><button type="button" className="github-connect" onClick={() => void connect()}>Connect GitHub</button></>}
      {repository.kind === "loading" && <p className="empty-state" role="status">Requesting GitHub repository metadata…</p>}
      {repository.kind === "error" && <div className="github-error"><p role="alert">{repository.error.message}. No credentials were requested by Desktop.</p><button type="button" onClick={() => void connect()}>Retry GitHub</button></div>}
      {repository.kind === "ready" && (
        <>
          <dl className="github-repository">
            <div><dt>Repository</dt><dd>{repository.repository.nameWithOwner}</dd></div>
            <div><dt>Default branch</dt><dd>{repository.repository.defaultBranch}</dd></div>
            <div><dt>Visibility</dt><dd>{repository.repository.isPrivate ? "Private" : "Public"}</dd></div>
            <div><dt>URL</dt><dd>{repository.repository.url}</dd></div>
          </dl>
          <form className="pr-form" onSubmit={(event) => { event.preventDefault(); void loadPullRequest(); }}>
            <label htmlFor="pr-number">Pull request number</label>
            <span><input id="pr-number" inputMode="numeric" value={number} onChange={(event) => setNumber(event.target.value)} disabled={pullRequest.kind === "loading"} /><button type="submit" disabled={pullRequest.kind === "loading"}>{pullRequest.kind === "loading" ? "Loading…" : "Open PR"}</button></span>
            <small>Open PR sends repository identity and PR number to GitHub, requesting PR metadata and ordered original commits.</small>
          </form>
        </>
      )}
      {pullRequest.kind === "error" && <div className="github-error"><p role="alert">{pullRequest.error.message}.</p>{pullRequest.error.retryable && <button type="button" onClick={() => void loadPullRequest(pullRequest.number)}>Retry PR</button>}</div>}
      {pullRequest.kind === "ready" && <PullRequestView key={`${pullRequest.pullRequest.nameWithOwner}#${pullRequest.pullRequest.number}`} pullRequest={pullRequest.pullRequest} />}
    </section>
  );
}

function PullRequestView({ pullRequest }: { pullRequest: GitHubPullRequest }) {
  const [remoteDiff, setRemoteDiff] = useState<{ kind: "idle" } | { kind: "loading"; oid: string } | { kind: "ready"; value: GitHubPullRequestCommitDiff } | { kind: "error"; oid: string; error: DesktopError }>({ kind: "idle" });
  const [trace, setTrace] = useState<{ kind: "idle" } | { kind: "loading" } | { kind: "ready"; value: GitHubSquashTrace } | { kind: "error"; error: DesktopError }>({ kind: "idle" });
  const serial = useRef(0);
  const traceSerial = useRef(0);
  useEffect(() => () => { serial.current += 1; traceSerial.current += 1; }, []);
  async function load(oid: string) { const current = ++serial.current; setRemoteDiff({ kind: "loading", oid }); try { const value = await getGitHubPullRequestCommitDiff(pullRequest.number, oid, pullRequest.nameWithOwner); if (current === serial.current) setRemoteDiff({ kind: "ready", value }); } catch (error) { if (current === serial.current) setRemoteDiff({ kind: "error", oid, error: asDesktopError(error) }); } }
  async function loadTrace() { const current = ++traceSerial.current; setTrace({ kind: "loading" }); try { const value = await getGitHubSquashTrace(pullRequest.number, pullRequest.nameWithOwner); if (current === traceSerial.current) setTrace({ kind: "ready", value }); } catch (error) { if (current === traceSerial.current) setTrace({ kind: "error", error: asDesktopError(error) }); } }
  return <section className="pr-detail" aria-labelledby="pr-title">
    <header><span className={`pr-state pr-state--${pullRequest.state}`}>{pullRequest.state}{pullRequest.isDraft ? " · draft" : ""}</span><h3 id="pr-title">#{pullRequest.number} {pullRequest.title}</h3><p>{pullRequest.authorLogin ? `@${pullRequest.authorLogin}` : "Unknown GitHub author"} · {pullRequest.base.name} ← {pullRequest.head.name}</p></header>
    {pullRequest.body && <pre>{pullRequest.body}</pre>}
    <dl className="github-repository"><div><dt>Updated</dt><dd>{pullRequest.updatedAt}</dd></div><div><dt>Merged</dt><dd>{pullRequest.mergedAt ?? "Not merged"}</dd></div><div><dt>Merge commit</dt><dd>{pullRequest.mergeCommitOid ?? "Not available"}</dd></div><div><dt>URL</dt><dd>{pullRequest.url}</dd></div></dl>
    <section className="squash-trace" aria-labelledby="squash-trace-title">
      <header><div><p className="eyebrow">Core relationship analysis</p><h4 id="squash-trace-title">Squash Trace</h4></div><button type="button" disabled={trace.kind === "loading"} onClick={() => void loadTrace()}>{trace.kind === "loading" ? "Analyzing…" : trace.kind === "ready" ? "Refresh trace" : "Analyze Squash Trace"}</button></header>
      {trace.kind === "idle" && <p className="empty-state">Analysis is off until requested. Core will refresh this PR and inspect the merge commit locally.</p>}
      {trace.kind === "loading" && <p className="empty-state" role="status">Analyzing PR and local merge relationship…</p>}
      {trace.kind === "error" && <div className="github-error"><p role="alert">{trace.error.message}. PR details remain available.</p><button type="button" onClick={() => void loadTrace()}>Retry Squash Trace</button></div>}
      {trace.kind === "ready" && <SquashTraceView trace={trace.value} />}
    </section>
    <h4>Original commits · {pullRequest.commits.length}</h4>
    {pullRequest.commits.length === 0 ? <p className="empty-state">No original commits returned.</p> : <ol className="pr-commits">{pullRequest.commits.map((commit) => <li key={commit.oid}><strong>{commit.summary || "(no commit message)"}</strong><code>{commit.oid.slice(0, 8)}</code><p>{commit.author.name} &lt;{commit.author.email}&gt;{commit.author.login ? ` · @${commit.author.login}` : ""} · {commit.parents.length} parent{commit.parents.length === 1 ? "" : "s"}</p><button type="button" disabled={remoteDiff.kind === "loading"} onClick={() => void load(commit.oid)}>View remote diff {commit.oid.slice(0, 8)}</button><small>Requests this listed commit's file metadata and available patch from GitHub.</small></li>)}</ol>}
    {remoteDiff.kind === "loading" && <p className="empty-state" role="status">Loading original commit diff…</p>}
    {remoteDiff.kind === "error" && <div className="github-error"><p role="alert">{remoteDiff.error.message}. PR details remain available.</p><button type="button" onClick={() => void load(remoteDiff.oid)}>Retry remote diff</button></div>}
    {remoteDiff.kind === "ready" && <section className="remote-diff" aria-label="Original commit remote diff"><h4>{remoteDiff.value.commit.summary}</h4>{remoteDiff.value.files.length === 0 ? <p className="empty-state">No changed files.</p> : remoteDiff.value.files.map((file, index) => <section key={`${file.newPath}:${index}`}><h5>{file.oldPath === file.newPath ? file.newPath : `${file.oldPath} → ${file.newPath}`}</h5><p>Provider status: {file.status} · +{file.additions} −{file.deletions} · {file.changes} changes</p>{file.patchState === "unavailable" ? <p className="empty-state">GitHub did not provide a patch for this file. Binary content is not assumed.</p> : <FileDiffView diff={{ oldPath: file.oldPath, newPath: file.newPath, isBinary: false, hunks: file.hunks }} />}</section>)}</section>}
  </section>;
}

const traceLabels = {
  notMerged: { title: "Not merged", detail: "This pull request is not merged, so there is no final merge relationship yet." },
  originalCommit: { title: "Original commit retained", detail: "The provider merge OID matches one of the PR's original commits." },
  mergeCommit: { title: "Merge commit", detail: "The local final commit has multiple parents, consistent with a merge commit." },
  squashCandidate: { title: "Squash candidate", detail: "The final commit is distinct and has at most one parent. This is a candidate relationship, not a verified merge strategy." },
  unresolved: { title: "Relationship unresolved", detail: "The available provider and local evidence is insufficient to classify this relationship." },
} as const;

const evidenceLabels: Record<SquashTraceEvidence, string> = {
  providerNotMerged: "GitHub reports the PR is not merged",
  providerMergeOidMissing: "GitHub did not provide a merge commit OID",
  mergeOidMatchesOriginalCommit: "Merge OID matches an original PR commit",
  mergeOidDistinctFromOriginalCommits: "Merge OID differs from every original PR commit",
  localCommitAvailable: "Final commit is available in the local repository",
  localCommitMissing: "Final commit is not available in the local repository",
  localCommitHasAtMostOneParent: "Final commit has at most one local parent",
  localCommitHasMultipleParents: "Final commit has multiple local parents",
  providerMergeStrategyUnavailable: "GitHub merge-strategy metadata is unavailable",
};

function SquashTraceView({ trace }: { trace: GitHubSquashTrace }) {
  const relationship = trace.relationship;
  const label = traceLabels[relationship.classification];
  return <div className="trace-result">
    <div className="trace-summary"><span className={`trace-confidence trace-confidence--${relationship.confidence}`}>{relationship.confidence} confidence</span><h5>{label.title}</h5><p>{label.detail}</p></div>
    <div className="trace-flow"><div><span>PR originals</span><strong>{trace.pullRequest.commits.length} commit{trace.pullRequest.commits.length === 1 ? "" : "s"}</strong></div><span aria-hidden="true">→</span><div><span>Final commit</span><strong>{relationship.mergeCommitOid?.slice(0, 12) ?? "Unavailable"}</strong></div></div>
    <dl className="github-repository"><div><dt>Local availability</dt><dd>{relationship.localAvailability}</dd></div><div><dt>Local parents</dt><dd>{relationship.localParentOids.length === 0 ? "None available" : relationship.localParentOids.map((oid) => oid.slice(0, 12)).join(", ")}</dd></div></dl>
    <div><h5>Evidence from Core</h5><ul className="trace-evidence">{relationship.evidence.map((item) => <li key={item}>{evidenceLabels[item]}</li>)}</ul></div>
  </div>;
}
