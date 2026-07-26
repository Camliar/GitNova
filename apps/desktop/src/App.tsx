import { useEffect, useRef, useState } from "react";
import type { CommitSummary, DiffScope, RepositoryDescriptor, RepositoryMutationSnapshot } from "@gitnova/protocol";
import markUrl from "../../../assets/icons/gitnova-mark.svg";
import { asDesktopError, configureCore, getCoreStatus, startCore, type CoreEnvironment, type CoreLaunchTarget, type DesktopError } from "./core";
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
import { MutationPanel } from "./MutationPanel";
import { AiAssistPanel } from "./AiAssistPanel";
import { AiSettingsPanel, defaultAiAssistSettings } from "./AiSettingsPanel";

type Connection =
  | { kind: "checking" }
  | { kind: "stopped" }
  | { kind: "connected"; version: string; mutations: boolean; aiAssist: boolean }
  | { kind: "error"; error: DesktopError };

type RepositoryState =
  | { kind: "idle" }
  | { kind: "selecting" }
  | { kind: "open"; repository: RepositoryDescriptor }
  | { kind: "error"; error: DesktopError };

type WorkspaceView = "changes" | "history" | "pullRequests" | "settings";

const repositoryKindLabel: Record<RepositoryDescriptor["kind"], string> = {
  worktree: "Worktree",
  linkedWorktree: "Linked worktree",
  bare: "Bare repository",
};

const workspaceBookmarkKey = "gitnova.workspace.v1";
type WorkspaceBookmark = { version: 1; target: CoreLaunchTarget; path: string };

function loadWorkspaceBookmark(): WorkspaceBookmark | null {
  try {
    const value = JSON.parse(localStorage.getItem(workspaceBookmarkKey) ?? "null") as Partial<WorkspaceBookmark> | null;
    if (value?.version !== 1 || typeof value.path !== "string" || !value.path || !value.target || typeof value.target.kind !== "string") return null;
    if (!(["local", "wsl", "ssh", "devContainer"] as string[]).includes(value.target.kind)) return null;
    return value as WorkspaceBookmark;
  } catch {
    return null;
  }
}

function saveWorkspaceBookmark(bookmark: WorkspaceBookmark) {
  try {
    localStorage.setItem(workspaceBookmarkKey, JSON.stringify(bookmark));
  } catch {
    // Host preference persistence must never block opening a repository.
  }
}

function targetDetail(target: CoreLaunchTarget) {
  if (target.kind === "wsl") return target.distribution;
  if (target.kind === "ssh") return target.destination;
  if (target.kind === "devContainer") return target.workspaceFolder;
  return "";
}

export function App() {
  const [connection, setConnection] = useState<Connection>({ kind: "checking" });
  const [environment, setEnvironment] = useState<CoreEnvironment>("local");
  const [environmentDetail, setEnvironmentDetail] = useState("");
  const [remoteRepositoryPath, setRemoteRepositoryPath] = useState("");
  const [repository, setRepository] = useState<RepositoryState>({ kind: "idle" });
  const [workspaceView, setWorkspaceView] = useState<WorkspaceView>("changes");
  const [aiSettings, setAiSettings] = useState(defaultAiAssistSettings);
  const [workingTree, setWorkingTree] = useState<WorkingTreeState>({ kind: "idle" });
  const [fileDiff, setFileDiff] = useState<DiffState>({ kind: "idle" });
  const diffRequest = useRef(0);
  const [history, setHistory] = useState<HistoryState>({ kind: "idle" });
  const historyRequest = useRef(0);
  const [commitDetail, setCommitDetail] = useState<CommitDetailState>({ kind: "idle" });
  const commitRequest = useRef(0);
  const [aiCommitDraft, setAiCommitDraft] = useState<{ id: number; message: string } | null>(null);
  const aiDraftSequence = useRef(0);

  useEffect(() => {
    let active = true;
    void (async () => {
      try {
        const initialStatus = await getCoreStatus();
        if (!active) return;
        const bookmark = loadWorkspaceBookmark();
        setEnvironment(bookmark?.target.kind ?? initialStatus.environment ?? "local");
        if (bookmark) {
          setEnvironmentDetail(targetDetail(bookmark.target));
          if (bookmark.target.kind !== "local") setRemoteRepositoryPath(bookmark.path);
        }
        let status = initialStatus;
        if (bookmark && !status.connected) {
          await configureCore(bookmark.target);
          status = await startCore();
          if (!active) return;
        }
        setConnection(status.connected ? { kind: "connected", version: status.protocolVersion ?? "unknown", mutations: status.capabilities?.repositoryMutations === true, aiAssist: status.capabilities?.aiAssist === true } : { kind: "stopped" });
        if (!bookmark || !status.connected) return;
        setRepository({ kind: "selecting" });
        try {
          const opened = await openRepository(bookmark.path);
          if (!active) return;
          await activateRepository(opened);
        } catch (error) {
          if (active) setRepository({ kind: "error", error: asDesktopError(error) });
        }
      } catch (error) {
        if (active) setConnection({ kind: "error", error: asDesktopError(error) });
      }
    })();
    return () => {
      active = false;
    };
  }, []);

  function launchTarget(): CoreLaunchTarget {
    if (environment === "wsl") return { kind: "wsl", distribution: environmentDetail.trim() };
    if (environment === "ssh") return { kind: "ssh", destination: environmentDetail.trim() };
    if (environment === "devContainer") return { kind: "devContainer", workspaceFolder: environmentDetail.trim() };
    return { kind: "local" };
  }

  async function activateRepository(opened: RepositoryDescriptor) {
    setRepository({ kind: "open", repository: opened });
    setWorkspaceView(opened.kind === "bare" ? "history" : "changes");
    setAiCommitDraft(null);
    diffRequest.current += 1;
    setFileDiff({ kind: "idle" });
    if (opened.kind === "bare") setWorkingTree({ kind: "idle" });
    await Promise.all([opened.kind !== "bare" ? refreshWorkingTree() : Promise.resolve(), refreshHistory()]);
  }

  async function connectCore() {
    setConnection({ kind: "checking" });
    try {
      const target = launchTarget();
      await configureCore(target);
      const status = await startCore();
      setEnvironment(status.environment ?? environment);
      setConnection({ kind: "connected", version: status.protocolVersion ?? "unknown", mutations: status.capabilities?.repositoryMutations === true, aiAssist: status.capabilities?.aiAssist === true });
    } catch (error) {
      setConnection({ kind: "error", error: asDesktopError(error) });
    }
  }

  async function chooseRepository() {
    setRepository({ kind: "selecting" });
    try {
      const path = environment === "local" ? await selectRepositoryDirectory() : remoteRepositoryPath.trim();
      if (path === null) {
        setRepository({ kind: "idle" });
        return;
      }
      if (path.length === 0) {
        throw { code: "desktop.remote_path_required", message: "Enter the repository path in the Core environment", retryable: false } satisfies DesktopError;
      }
      const opened = await openRepository(path);
      saveWorkspaceBookmark({ version: 1, target: launchTarget(), path });
      await activateRepository(opened);
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
      saveWorkspaceBookmark({ version: 1, target: launchTarget(), path });
      await activateRepository(opened);
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

  function applyMutation(snapshot: RepositoryMutationSnapshot) {
    diffRequest.current += 1;
    setFileDiff({ kind: "idle" });
    commitRequest.current += 1;
    setCommitDetail({ kind: "idle" });
    setWorkingTree({ kind: "ready", status: snapshot.status });
    void refreshHistory();
  }

  const coreDetail =
    connection.kind === "connected"
      ? `Connected · ${environment} · v${connection.version}`
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

  const openedRepository = repository.kind === "open" ? repository.repository : null;
  const repositoryPath = openedRepository?.worktreeRoot ?? openedRepository?.gitDirectory ?? "";
  const repositoryName = repositoryPath.split(/[\\/]/).filter(Boolean).at(-1) ?? "Repository";
  const changeCount = workingTree.kind === "ready" ? workingTree.status.entries.length : 0;
  const branchName = workingTree.kind === "ready" ? workingTree.status.branch.head ?? "Detached HEAD" : "Reading branch…";

  return (
    <div className="app-shell">
      <header className="app-header app-toolbar">
        <a className="brand" href="#main-content" aria-label="GitNova home">
          <img src={markUrl} alt="" width="30" height="30" />
          <span>GitNova</span>
        </a>
        {openedRepository ? (
          <div className="toolbar-repository" aria-label="Current repository">
            <strong>{repositoryName}</strong>
            <span>{branchName}</span>
          </div>
        ) : <span className="toolbar-title">Local-first Git client</span>}
        <div className="toolbar-actions">
          {openedRepository && openedRepository.kind !== "bare" && (
            <button type="button" onClick={() => void refreshWorkingTree()}>Refresh repository</button>
          )}
          {openedRepository && <button type="button" onClick={() => void reopenRepository()}>Reopen repository</button>}
        </div>
      </header>

      {openedRepository ? (
        <main id="main-content" className="repository-workbench" tabIndex={-1}>
          <aside className="repository-sidebar" aria-label="Repository navigation">
            <div className="repository-sidebar__title">
              <img src={markUrl} alt="" width="28" height="28" />
              <span><strong>{repositoryName}</strong><small>{repositoryKindLabel[openedRepository.kind]}</small></span>
            </div>
            <nav>
              {openedRepository.kind !== "bare" && <button type="button" className={workspaceView === "changes" ? "is-active" : ""} onClick={() => setWorkspaceView("changes")}><span>Local Changes</span><strong>{changeCount}</strong></button>}
              <button type="button" className={workspaceView === "history" ? "is-active" : ""} onClick={() => setWorkspaceView("history")}><span>All Commits</span></button>
              <button type="button" className={workspaceView === "pullRequests" ? "is-active" : ""} onClick={() => setWorkspaceView("pullRequests")}><span>Pull Requests</span></button>
              <button type="button" className={workspaceView === "settings" ? "is-active" : ""} onClick={() => setWorkspaceView("settings")}><span>Settings</span></button>
            </nav>
            <dl className="repository-facts">
              <div><dt>Core</dt><dd>{coreDetail}</dd></div>
              <div><dt>System Git</dt><dd>{openedRepository.gitVersion}</dd></div>
              <div><dt>Path</dt><dd title={repositoryPath}>{repositoryPath}</dd></div>
            </dl>
            <p className="privacy-note">Repository data stays in the Core environment.</p>
          </aside>

          <section className="workbench-main">
            <header className="view-header">
              <div>
                <p>{workspaceView === "changes" ? "Working copy" : workspaceView === "history" ? "Repository history" : workspaceView === "pullRequests" ? "Provider data" : "Application preferences"}</p>
                <h1>{workspaceView === "changes" ? "Local Changes" : workspaceView === "history" ? "All Commits" : workspaceView === "pullRequests" ? "Pull Requests" : "Settings"}</h1>
              </div>
              <span className="branch-pill">{branchName}</span>
            </header>

            {workspaceView === "changes" && openedRepository.kind !== "bare" && (
              <div className="changes-workspace">
                <div className="changes-browser">
                  <WorkingTreePanel state={workingTree} diffLoading={fileDiff.kind === "loading"} onRefresh={() => void refreshWorkingTree()} onDiff={(path: string, scope: DiffScope) => void loadFileDiff({ path, scope })} />
                </div>
                <div className="changes-detail">
                  {fileDiff.kind === "idle" ? <div className="pane-placeholder"><strong>Select a changed file</strong><span>Choose View beside a staged or working change to inspect its line diff.</span></div> : <DiffPanel state={fileDiff} onRetry={() => void loadFileDiff(fileDiff.selection)} onClose={closeFileDiff} />}
                </div>
                {connection.kind === "connected" && connection.mutations && workingTree.kind === "ready" && (
                  <MutationPanel
                    key={`mutations:${openedRepository.gitDirectory}`}
                    status={workingTree.status}
                    suggestedCommit={aiCommitDraft}
                    onApplied={applyMutation}
                    aiAssist={connection.aiAssist ? <AiAssistPanel key={`ai:${openedRepository.gitDirectory}`} settings={aiSettings} onUseCommitMessage={(message) => setAiCommitDraft({ id: ++aiDraftSequence.current, message })} /> : null}
                  />
                )}
              </div>
            )}

            {workspaceView === "history" && (
              <div className="history-workspace">
                {openedRepository.kind === "bare" && <p className="bare-repository-note">Bare repositories do not have a working tree.</p>}
                <HistoryPanel state={history} commitLoading={commitDetail.kind === "loading"} onRetry={() => void refreshHistory()} onLoadMore={() => void loadMoreHistory()} onSelectCommit={selectCommit} />
                <div className="history-detail">
                  {commitDetail.kind === "idle" ? <div className="pane-placeholder"><strong>Select a commit</strong><span>Commit metadata, changed files and line-level diff will appear here.</span></div> : <CommitDetailPanel key={`${commitDetail.kind === "choosingParent" ? commitDetail.commit.oid : commitDetail.selection.commit.oid}:${commitDetail.kind === "choosingParent" ? "" : commitDetail.selection.parentOid ?? ""}`} state={commitDetail} onChooseParent={chooseCommitParent} onRetry={() => commitDetail.kind === "error" && void loadCommitDiff(commitDetail.selection)} onClose={closeCommitDetail} />}
                </div>
              </div>
            )}

            {workspaceView === "pullRequests" && <div className="provider-workspace"><GitHubPanel key={`github:${openedRepository.gitDirectory}`} /></div>}
            {workspaceView === "settings" && <div className="settings-workspace"><AiSettingsPanel settings={aiSettings} onChange={setAiSettings} /></div>}
          </section>
        </main>
      ) : (
        <main id="main-content" className="setup-workspace" tabIndex={-1}>
          <section className="setup-intro">
            <img src={markUrl} alt="" width="72" height="72" />
            <p className="eyebrow">GitNova Desktop</p>
            <h1>Open a repository.</h1>
            <p>Inspect local changes, commit history, pull request commits and Squash Trace without moving repository data to a central service.</p>
          </section>
          <aside className="foundation-card" aria-labelledby="foundation-title">
            <div><p className="eyebrow">Workspace setup</p><h2 id="foundation-title">Connect Core</h2></div>
            <ul>
              <li><span className={`status-mark status-mark--${connection.kind === "connected" ? "ready" : connection.kind === "checking" ? "pending" : "idle"}`} aria-hidden="true" /><span>Core connection</span><strong>{coreDetail}</strong></li>
              <li><span className={`status-mark status-mark--${repository.kind === "open" ? "ready" : repository.kind === "selecting" ? "pending" : "idle"}`} aria-hidden="true" /><span>Repository</span><strong>{repositoryDetail}</strong></li>
            </ul>
            {(connection.kind === "stopped" || connection.kind === "error") && <div className="connection-action">
              {connection.kind === "error" && <p role="alert">{connection.error.message}. No repository data was changed.</p>}
              <label className="environment-field">Core environment<select value={environment} onChange={(event) => { setEnvironment(event.target.value as CoreEnvironment); setEnvironmentDetail(""); }}><option value="local">This computer</option><option value="wsl">WSL distribution</option><option value="ssh">Remote SSH</option><option value="devContainer">Dev Container</option></select></label>
              {environment !== "local" && <label className="environment-field">{environment === "wsl" ? "Distribution name" : environment === "ssh" ? "SSH destination" : "Local workspace folder"}<input value={environmentDetail} onChange={(event) => setEnvironmentDetail(event.target.value)} placeholder={environment === "wsl" ? "Ubuntu-24.04" : environment === "ssh" ? "user@example.com" : "/absolute/path/to/workspace"} /></label>}
              {environment !== "local" && <p className="environment-note"><code>gitnova-core</code> must already be installed on PATH in that environment.</p>}
              <button type="button" onClick={() => void connectCore()}>{connection.kind === "error" ? "Retry Core" : "Start Core"}</button>
            </div>}
            {connection.kind === "connected" && <div className="connection-action">
              {repository.kind === "error" && <p role="alert">{repository.error.message}. No repository data was changed.</p>}
              {environment !== "local" && <label className="environment-field">Repository path in {environment}<input value={remoteRepositoryPath} onChange={(event) => setRemoteRepositoryPath(event.target.value)} placeholder="/workspaces/project" /></label>}
              <button type="button" disabled={repository.kind === "selecting"} onClick={() => void chooseRepository()}>{repository.kind === "selecting" ? "Opening…" : repository.kind === "error" ? environment === "local" ? "Choose another folder" : "Open another repository path" : environment === "local" ? "Choose repository" : "Open repository path"}</button>
            </div>}
            <p className="privacy-note">The selected path is sent only to GitNova Core in the same repository environment.</p>
          </aside>
        </main>
      )}
    </div>
  );
}
