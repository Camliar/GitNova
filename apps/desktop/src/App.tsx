import { useEffect, useRef, useState } from "react";
import type { CommitSummary, DiffScope, RepositoryDescriptor } from "@gitnova/protocol";
import markUrl from "../../../assets/icons/gitnova-mark.svg";
import { asDesktopError, getCoreStatus, startCore, type DesktopError } from "./core";
import { openRepository, selectRepositoryDirectory } from "./repository";
import { getWorkingTreeStatus } from "./status";
import { WorkingTreePanel, type WorkingTreeState } from "./WorkingTreePanel";
import { getFileDiff } from "./diff";
import { DiffPanel, type DiffSelection, type DiffState } from "./DiffPanel";
import { getCommitGraph } from "./history";
import { HistoryPanel, type HistoryState } from "./HistoryPanel";
import { getCommitDiff } from "./commitDiff";
import { CommitDetailPanel, type CommitDetailState, type CommitSelection } from "./CommitDetailPanel";
import { GitHubPanel } from "./GitHubPanel";

type Connection =
  | { kind: "checking" }
  | { kind: "stopped" }
  | { kind: "connected"; version: string }
  | { kind: "error"; error: DesktopError };

type RepositoryState =
  | { kind: "idle" }
  | { kind: "selecting" }
  | { kind: "open"; repository: RepositoryDescriptor }
  | { kind: "error"; error: DesktopError };

const repositoryKindLabel: Record<RepositoryDescriptor["kind"], string> = {
  worktree: "Worktree",
  linkedWorktree: "Linked worktree",
  bare: "Bare repository",
};

export function App() {
  const [connection, setConnection] = useState<Connection>({ kind: "checking" });
  const [repository, setRepository] = useState<RepositoryState>({ kind: "idle" });
  const [workingTree, setWorkingTree] = useState<WorkingTreeState>({ kind: "idle" });
  const [fileDiff, setFileDiff] = useState<DiffState>({ kind: "idle" });
  const diffRequest = useRef(0);
  const [history, setHistory] = useState<HistoryState>({ kind: "idle" });
  const historyRequest = useRef(0);
  const [commitDetail, setCommitDetail] = useState<CommitDetailState>({ kind: "idle" });
  const commitRequest = useRef(0);

  useEffect(() => {
    let active = true;
    void getCoreStatus()
      .then((status) => {
        if (!active) return;
        setConnection(
          status.connected
            ? { kind: "connected", version: status.protocolVersion ?? "unknown" }
            : { kind: "stopped" },
        );
      })
      .catch((error: unknown) => {
        if (active) setConnection({ kind: "error", error: asDesktopError(error) });
      });
    return () => {
      active = false;
    };
  }, []);

  async function connectCore() {
    setConnection({ kind: "checking" });
    try {
      const status = await startCore();
      setConnection({ kind: "connected", version: status.protocolVersion ?? "unknown" });
    } catch (error) {
      setConnection({ kind: "error", error: asDesktopError(error) });
    }
  }

  async function chooseRepository() {
    setRepository({ kind: "selecting" });
    try {
      const path = await selectRepositoryDirectory();
      if (path === null) {
        setRepository({ kind: "idle" });
        return;
      }
      const opened = await openRepository(path);
      setRepository({ kind: "open", repository: opened });
      diffRequest.current += 1;
      setFileDiff({ kind: "idle" });
      if (opened.kind === "bare") setWorkingTree({ kind: "idle" });
      await Promise.all([opened.kind !== "bare" ? refreshWorkingTree() : Promise.resolve(), refreshHistory()]);
    } catch (error) {
      setRepository({ kind: "error", error: asDesktopError(error) });
    }
  }

  async function reopenRepository() {
    if (repository.kind !== "open") return;
    const path = repository.repository.worktreeRoot ?? repository.repository.gitDirectory;
    historyRequest.current += 1;
    setHistory({ kind: "loading" });
    setRepository({ kind: "selecting" });
    try {
      const opened = await openRepository(path);
      setRepository({ kind: "open", repository: opened });
      diffRequest.current += 1;
      setFileDiff({ kind: "idle" });
      if (opened.kind === "bare") setWorkingTree({ kind: "idle" });
      await Promise.all([opened.kind !== "bare" ? refreshWorkingTree() : Promise.resolve(), refreshHistory()]);
    } catch (error) {
      setRepository({ kind: "error", error: asDesktopError(error) });
    }
  }

  async function refreshWorkingTree() {
    diffRequest.current += 1;
    setFileDiff({ kind: "idle" });
    setWorkingTree({ kind: "loading" });
    try {
      setWorkingTree({ kind: "ready", status: await getWorkingTreeStatus() });
    } catch (error) {
      setWorkingTree({ kind: "error", error: asDesktopError(error) });
    }
  }

  async function loadFileDiff(selection: DiffSelection) {
    const request = ++diffRequest.current;
    setFileDiff({ kind: "loading", selection });
    try {
      const diff = await getFileDiff(selection.path, selection.scope);
      if (request === diffRequest.current) setFileDiff({ kind: "ready", selection, diff });
    } catch (error) {
      if (request === diffRequest.current) setFileDiff({ kind: "error", selection, error: asDesktopError(error) });
    }
  }

  function closeFileDiff() {
    diffRequest.current += 1;
    setFileDiff({ kind: "idle" });
  }

  async function refreshHistory() {
    commitRequest.current += 1;
    setCommitDetail({ kind: "idle" });
    const request = ++historyRequest.current;
    setHistory({ kind: "loading" });
    try {
      const page = await getCommitGraph();
      if (request === historyRequest.current) {
        setHistory({ kind: "ready", nodes: page.nodes, nextCursor: page.nextCursor, more: { kind: "idle" } });
      }
    } catch (error) {
      if (request === historyRequest.current) setHistory({ kind: "error", error: asDesktopError(error) });
    }
  }

  function selectCommit(commit: CommitSummary) {
    commitRequest.current += 1;
    if (commit.parents.length > 1) {
      setCommitDetail({ kind: "choosingParent", commit });
    } else {
      void loadCommitDiff({ commit });
    }
  }

  async function loadCommitDiff(selection: CommitSelection) {
    const request = ++commitRequest.current;
    setCommitDetail({ kind: "loading", selection });
    try {
      const diff = await getCommitDiff(selection.commit.oid, selection.parentOid);
      if (request === commitRequest.current) setCommitDetail({ kind: "ready", selection, diff });
    } catch (error) {
      if (request === commitRequest.current) setCommitDetail({ kind: "error", selection, error: asDesktopError(error) });
    }
  }

  function chooseCommitParent(parentOid: string) {
    if (commitDetail.kind === "choosingParent" && commitDetail.commit.parents.includes(parentOid)) {
      void loadCommitDiff({ commit: commitDetail.commit, parentOid });
    }
  }

  function closeCommitDetail() {
    commitRequest.current += 1;
    setCommitDetail({ kind: "idle" });
  }

  async function loadMoreHistory() {
    if (history.kind !== "ready" || !history.nextCursor || history.more.kind === "loading") return;
    const request = historyRequest.current;
    const snapshot = history;
    const cursor = history.nextCursor;
    setHistory({ ...snapshot, more: { kind: "loading" } });
    try {
      const page = await getCommitGraph(cursor);
      if (request === historyRequest.current) {
        setHistory({ kind: "ready", nodes: [...snapshot.nodes, ...page.nodes], nextCursor: page.nextCursor, more: { kind: "idle" } });
      }
    } catch (error) {
      if (request === historyRequest.current) {
        setHistory({ ...snapshot, more: { kind: "error", error: asDesktopError(error) } });
      }
    }
  }

  const coreDetail =
    connection.kind === "connected"
      ? `Connected · v${connection.version}`
      : connection.kind === "checking"
        ? "Checking…"
        : connection.kind === "error"
          ? "Unavailable"
          : "Not running";
  const repositoryDetail =
    repository.kind === "open"
      ? repositoryKindLabel[repository.repository.kind]
      : repository.kind === "selecting"
        ? "Opening…"
        : repository.kind === "error"
          ? "Not opened"
          : "Not opened";

  return (
    <div className="app-shell">
      <header className="app-header">
        <a className="brand" href="#main-content" aria-label="GitNova home">
          <img src={markUrl} alt="" width="36" height="36" />
          <span>GitNova</span>
        </a>
        <span className="local-badge">
          <span aria-hidden="true" className="local-badge__dot" />
          Local-first desktop
        </span>
      </header>

      <main id="main-content" className="workspace" tabIndex={-1}>
        <section className="hero" aria-labelledby="welcome-title">
          <p className="eyebrow">Open a local repository</p>
          <h1 id="welcome-title">Understand the history behind the merge.</h1>
          <p className="hero__copy">
            Choose a repository in this environment. GitNova Core will identify it without scanning
            other folders or moving repository data to a central service.
          </p>
          {repository.kind === "open" ? (
            <section className="repository-card" aria-labelledby="repository-title">
              <div>
                <p className="eyebrow">Active repository</p>
                <h2 id="repository-title">{repositoryKindLabel[repository.repository.kind]}</h2>
              </div>
              <dl>
                {repository.repository.worktreeRoot && (
                  <div><dt>Worktree</dt><dd>{repository.repository.worktreeRoot}</dd></div>
                )}
                <div><dt>Git directory</dt><dd>{repository.repository.gitDirectory}</dd></div>
                <div><dt>System Git</dt><dd>{repository.repository.gitVersion}</dd></div>
              </dl>
            </section>
          ) : (
            <div className="next-step" role="status" aria-live="polite">
              <span className="next-step__icon" aria-hidden="true">01</span>
              <span>
                <strong>{connection.kind === "connected" ? "Choose a repository" : "Start Core"}</strong>{" "}
                to establish the local data path.
              </span>
            </div>
          )}
          {repository.kind === "open" && repository.repository.kind === "bare" && (
            <section className="working-tree" aria-labelledby="working-tree-title">
              <p className="eyebrow">Working tree</p>
              <h2 id="working-tree-title">Not available</h2>
              <p className="empty-state">Bare repositories do not have a working tree.</p>
            </section>
          )}
          {repository.kind === "open" && repository.repository.kind !== "bare" && (
            <WorkingTreePanel
              state={workingTree}
              diffLoading={fileDiff.kind === "loading"}
              onRefresh={() => void refreshWorkingTree()}
              onDiff={(path: string, scope: DiffScope) => void loadFileDiff({ path, scope })}
            />
          )}
          {fileDiff.kind !== "idle" && (
            <DiffPanel
              state={fileDiff}
              onRetry={() => void loadFileDiff(fileDiff.selection)}
              onClose={closeFileDiff}
            />
          )}
          {repository.kind === "open" && (
            <HistoryPanel
              state={history}
              commitLoading={commitDetail.kind === "loading"}
              onRetry={() => void refreshHistory()}
              onLoadMore={() => void loadMoreHistory()}
              onSelectCommit={selectCommit}
            />
          )}
          {commitDetail.kind !== "idle" && (
            <CommitDetailPanel
              key={`${commitDetail.kind === "choosingParent" ? commitDetail.commit.oid : commitDetail.selection.commit.oid}:${commitDetail.kind === "choosingParent" ? "" : commitDetail.selection.parentOid ?? ""}`}
              state={commitDetail}
              onChooseParent={chooseCommitParent}
              onRetry={() => commitDetail.kind === "error" && void loadCommitDiff(commitDetail.selection)}
              onClose={closeCommitDetail}
            />
          )}
          {repository.kind === "open" && <GitHubPanel key={repository.repository.gitDirectory} />}
        </section>

        <aside className="foundation-card" aria-labelledby="foundation-title">
          <div><p className="eyebrow">System status</p><h2 id="foundation-title">Workspace</h2></div>
          <ul>
            <li>
              <span className={`status-mark status-mark--${connection.kind === "connected" ? "ready" : connection.kind === "checking" ? "pending" : "idle"}`} aria-hidden="true" />
              <span>Core connection</span><strong>{coreDetail}</strong>
            </li>
            <li>
              <span className={`status-mark status-mark--${repository.kind === "open" ? "ready" : repository.kind === "selecting" ? "pending" : "idle"}`} aria-hidden="true" />
              <span>Repository</span><strong>{repositoryDetail}</strong>
            </li>
          </ul>
          {(connection.kind === "stopped" || connection.kind === "error") && (
            <div className="connection-action">
              {connection.kind === "error" && <p role="alert">{connection.error.message}. No repository data was changed.</p>}
              <button type="button" onClick={() => void connectCore()}>
                {connection.kind === "error" ? "Retry Core" : "Start Core"}
              </button>
            </div>
          )}
          {connection.kind === "connected" && repository.kind !== "open" && (
            <div className="connection-action">
              {repository.kind === "error" && <p role="alert">{repository.error.message}. No repository data was changed.</p>}
              <button type="button" disabled={repository.kind === "selecting"} onClick={() => void chooseRepository()}>
                {repository.kind === "selecting" ? "Opening…" : repository.kind === "error" ? "Choose another folder" : "Choose repository"}
              </button>
            </div>
          )}
          {connection.kind === "connected" && repository.kind === "open" && (
            <div className="connection-action">
              <button type="button" className="button-secondary" onClick={() => void reopenRepository()}>
                Reopen repository
              </button>
            </div>
          )}
          <p className="privacy-note">The selected path is sent only to GitNova Core in the same repository environment.</p>
        </aside>
      </main>
    </div>
  );
}
