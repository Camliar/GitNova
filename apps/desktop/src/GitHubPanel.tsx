import { useEffect, useRef, useState } from "react";
import type { GitHubPullRequest, GitHubRepository } from "@gitnova/protocol";
import { asDesktopError, type DesktopError } from "./core";
import { getGitHubPullRequest, getGitHubRepository } from "./github";

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
      <header><p className="eyebrow">GitHub provider</p><h2 id="github-title">Pull request history</h2></header>
      {repository.kind === "idle" && <><p className="empty-state">GitHub access is off until you explicitly connect. Core uses the existing gh configuration in this repository environment.</p><button type="button" className="github-connect" onClick={() => void connect()}>Connect GitHub</button></>}
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
          </form>
        </>
      )}
      {pullRequest.kind === "error" && <div className="github-error"><p role="alert">{pullRequest.error.message}.</p>{pullRequest.error.retryable && <button type="button" onClick={() => void loadPullRequest(pullRequest.number)}>Retry PR</button>}</div>}
      {pullRequest.kind === "ready" && <PullRequestView pullRequest={pullRequest.pullRequest} />}
    </section>
  );
}

function PullRequestView({ pullRequest }: { pullRequest: GitHubPullRequest }) {
  return <section className="pr-detail" aria-labelledby="pr-title">
    <header><span className={`pr-state pr-state--${pullRequest.state}`}>{pullRequest.state}{pullRequest.isDraft ? " · draft" : ""}</span><h3 id="pr-title">#{pullRequest.number} {pullRequest.title}</h3><p>{pullRequest.authorLogin ? `@${pullRequest.authorLogin}` : "Unknown GitHub author"} · {pullRequest.base.name} ← {pullRequest.head.name}</p></header>
    {pullRequest.body && <pre>{pullRequest.body}</pre>}
    <dl className="github-repository"><div><dt>Updated</dt><dd>{pullRequest.updatedAt}</dd></div><div><dt>Merged</dt><dd>{pullRequest.mergedAt ?? "Not merged"}</dd></div><div><dt>Merge commit</dt><dd>{pullRequest.mergeCommitOid ?? "Not available"}</dd></div><div><dt>URL</dt><dd>{pullRequest.url}</dd></div></dl>
    <h4>Original commits · {pullRequest.commits.length}</h4>
    {pullRequest.commits.length === 0 ? <p className="empty-state">No original commits returned.</p> : <ol className="pr-commits">{pullRequest.commits.map((commit) => <li key={commit.oid}><strong>{commit.summary || "(no commit message)"}</strong><code>{commit.oid.slice(0, 8)}</code><p>{commit.author.name} &lt;{commit.author.email}&gt;{commit.author.login ? ` · @${commit.author.login}` : ""} · {commit.parents.length} parent{commit.parents.length === 1 ? "" : "s"}</p></li>)}</ol>}
  </section>;
}
