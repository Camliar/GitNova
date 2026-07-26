import { useEffect, useRef, useState } from "react";
import type { CommitSummary, DiffScope, RepositoryDescriptor, RepositoryMutationSnapshot } from "@gitnova/protocol";
import markUrl from "../../../assets/icons/gitnova-mark.svg";
import { asDesktopError, configureCore, getCoreStatus, shutdownCore, startCore, type CoreEnvironment, type CoreLaunchTarget, type CoreStatus, type DesktopError } from "./core";
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
import { BranchSwitcher, type ReferencesState } from "./BranchSwitcher";
import { RepositoryRefTree } from "./RepositoryRefTree";
import { getRepositoryReferences, switchLocalBranch } from "./mutations";

type Connection =
  | { kind: "checking" }
  | { kind: "stopped" }
  | { kind: "connected"; version: string; mutations: boolean; references: boolean; aiAssist: boolean }
  | { kind: "error"; error: DesktopError };

type RepositoryState =
  | { kind: "idle" }
  | { kind: "selecting" }
  | { kind: "open"; repository: RepositoryDescriptor }
  | { kind: "error"; error: DesktopError };

type WorkspaceView = "changes" | "history" | "pullRequests" | "settings";
type BranchOperation = { kind: "idle" } | { kind: "confirm"; name: string } | { kind: "loading"; name: string } | { kind: "error"; name: string; error: DesktopError };

const repositoryKindLabel: Record<RepositoryDescriptor["kind"], string> = {
  worktree: "Worktree",
  linkedWorktree: "Linked worktree",
  bare: "Bare repository",
};

const legacyWorkspaceBookmarkKey = "gitnova.workspace.v1";
const workspaceStateKey = "gitnova.workspace.v2";
type RepositoryBookmark = { target: CoreLaunchTarget; path: string };
type PersistedWorkspace = { version: 2; active: RepositoryBookmark | null; repositories: RepositoryBookmark[] };

function isLaunchTarget(value: unknown): value is CoreLaunchTarget {
  if (!value || typeof value !== "object" || !("kind" in value)) return false;
  if (value.kind === "local") return true;
  if (value.kind === "wsl") return "distribution" in value && typeof value.distribution === "string";
  if (value.kind === "ssh") return "destination" in value && typeof value.destination === "string";
  return value.kind === "devContainer" && "workspaceFolder" in value && typeof value.workspaceFolder === "string";
}

function isRepositoryBookmark(value: unknown): value is RepositoryBookmark {
  return !!value && typeof value === "object" && "path" in value && typeof value.path === "string" && value.path.length > 0 && "target" in value && isLaunchTarget(value.target);
}

function bookmarkKey(bookmark: RepositoryBookmark) {
  return `${JSON.stringify(bookmark.target)}\n${bookmark.path}`;
}

function targetKey(target: CoreLaunchTarget) {
  return JSON.stringify(target);
}

function loadWorkspaceState(): PersistedWorkspace {
  try {
    const value = JSON.parse(localStorage.getItem(workspaceStateKey) ?? "null") as Partial<PersistedWorkspace> | null;
    if (value?.version === 2) {
      let repositories = Array.isArray(value.repositories) ? value.repositories.filter(isRepositoryBookmark).slice(0, 12) : [];
      const active = isRepositoryBookmark(value.active) ? value.active : repositories[0] ?? null;
      if (active && !repositories.some((entry) => bookmarkKey(entry) === bookmarkKey(active))) repositories = [active, ...repositories].slice(0, 12);
      return { version: 2, active, repositories };
    }
  } catch {
    // Fall through to the legacy bookmark.
  }
  try {
    const legacy = JSON.parse(localStorage.getItem(legacyWorkspaceBookmarkKey) ?? "null") as { version?: unknown; target?: unknown; path?: unknown } | null;
    if (legacy?.version === 1 && isRepositoryBookmark(legacy)) {
      const bookmark = { target: legacy.target, path: legacy.path };
      return { version: 2, active: bookmark, repositories: [bookmark] };
    }
  } catch {
    // Corrupt host preferences are ignored.
  }
  return { version: 2, active: null, repositories: [] };
}

function saveWorkspaceState(workspace: PersistedWorkspace) {
  try {
    localStorage.setItem(workspaceStateKey, JSON.stringify(workspace));
    localStorage.removeItem(legacyWorkspaceBookmarkKey);
  } catch {
    // Host preference persistence must never block opening a repository.
  }
}

function repositoryLabel(path: string) {
  return path.split(/[\\/]/).filter(Boolean).at(-1) ?? path;
}

function targetDetail(target: CoreLaunchTarget) {
  if (target.kind === "wsl") return target.distribution;
  if (target.kind === "ssh") return target.destination;
  if (target.kind === "devContainer") return target.workspaceFolder;
  return "";
}

export function App() {
  const initialWorkspace = useRef(loadWorkspaceState());
  const [connection, setConnection] = useState<Connection>({ kind: "checking" });
  const referencesCapability = useRef(false);
  const [environment, setEnvironment] = useState<CoreEnvironment>("local");
  const [environmentDetail, setEnvironmentDetail] = useState("");
  const [remoteRepositoryPath, setRemoteRepositoryPath] = useState("");
  const [repository, setRepository] = useState<RepositoryState>({ kind: "idle" });
  const [recentRepositories, setRecentRepositories] = useState(initialWorkspace.current.repositories);
  const [workspaceError, setWorkspaceError] = useState<DesktopError | null>(null);
  const [workspaceView, setWorkspaceView] = useState<WorkspaceView>("changes");
  const [references, setReferences] = useState<ReferencesState>({ kind: "idle" });
  const referencesRequest = useRef(0);
  const [branchOperation, setBranchOperation] = useState<BranchOperation>({ kind: "idle" });
  const [aiSettings, setAiSettings] = useState(defaultAiAssistSettings);
  const [workingTree, setWorkingTree] = useState<WorkingTreeState>({ kind: "idle" });
  const workingTreeRequest = useRef(0);
  const [fileDiff, setFileDiff] = useState<DiffState>({ kind: "idle" });
  const diffRequest = useRef(0);
  const [history, setHistory] = useState<HistoryState>({ kind: "idle" });
  const historyRequest = useRef(0);
  const [commitDetail, setCommitDetail] = useState<CommitDetailState>({ kind: "idle" });
  const [historyDetailTab, setHistoryDetailTab] = useState<"commit" | "changes">("commit");
  const commitRequest = useRef(0);
  const [aiCommitDraft, setAiCommitDraft] = useState<{ id: number; message: string } | null>(null);
  const aiDraftSequence = useRef(0);

  useEffect(() => {
    let active = true;
    void (async () => {
      try {
        const initialStatus = await getCoreStatus();
        if (!active) return;
        const bookmark = initialWorkspace.current.active;
        setEnvironment(bookmark?.target.kind ?? initialStatus.environment ?? "local");
        if (bookmark) {
          setEnvironmentDetail(targetDetail(bookmark.target));
          if (bookmark.target.kind !== "local") setRemoteRepositoryPath(bookmark.path);
        }
        let status = initialStatus;
        if (bookmark && status.connected && status.environment !== bookmark.target.kind) {
          status = await shutdownCore();
        }
        if (bookmark && !status.connected) {
          await configureCore(bookmark.target);
          status = await startCore();
          if (!active) return;
        }
        if (status.connected) applyCoreStatus(status); else setConnection({ kind: "stopped" });
        if (!bookmark || !status.connected) return;
        setRepository({ kind: "selecting" });
        try {
          const opened = await openRepository(bookmark.path);
          if (!active) return;
          rememberRepository(bookmark);
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

  function applyCoreStatus(status: CoreStatus) {
    referencesCapability.current = status.capabilities?.repositoryReferences === true || status.capabilities?.repositoryMutations === true;
    if (!referencesCapability.current) {
      referencesRequest.current += 1;
      setReferences({ kind: "idle" });
    }
    setConnection({ kind: "connected", version: status.protocolVersion ?? "unknown", mutations: status.capabilities?.repositoryMutations === true, references: referencesCapability.current, aiAssist: status.capabilities?.aiAssist === true });
  }

  function rememberRepository(bookmark: RepositoryBookmark) {
    setRecentRepositories((current) => {
      const key = bookmarkKey(bookmark);
      const repositories = [bookmark, ...current.filter((entry) => bookmarkKey(entry) !== key)].slice(0, 12);
      saveWorkspaceState({ version: 2, active: bookmark, repositories });
      return repositories;
    });
  }

  function showTarget(target: CoreLaunchTarget, path = "") {
    setEnvironment(target.kind);
    setEnvironmentDetail(targetDetail(target));
    setRemoteRepositoryPath(target.kind === "local" ? "" : path);
  }

  async function ensureCoreTarget(target: CoreLaunchTarget) {
    const current = launchTarget();
    let status = await getCoreStatus();
    if (status.connected && targetKey(current) !== targetKey(target)) {
      status = await shutdownCore();
    }
    if (!status.connected) {
      await configureCore(target);
      status = await startCore();
    }
    showTarget(target);
    applyCoreStatus(status);
  }

  async function activateRepository(opened: RepositoryDescriptor) {
    setRepository({ kind: "open", repository: opened });
    setWorkspaceView(opened.kind === "bare" ? "history" : "changes");
    setAiCommitDraft(null);
    setBranchOperation({ kind: "idle" });
    diffRequest.current += 1;
    setFileDiff({ kind: "idle" });
    if (opened.kind === "bare") {
      workingTreeRequest.current += 1;
      setWorkingTree({ kind: "idle" });
    }
    await Promise.all([opened.kind !== "bare" ? refreshWorkingTree() : Promise.resolve(), refreshHistory(), referencesCapability.current ? refreshReferences() : Promise.resolve()]);
  }

  async function connectCore() {
    setConnection({ kind: "checking" });
    try {
      const target = launchTarget();
      await configureCore(target);
      const status = await startCore();
      setEnvironment(status.environment ?? environment);
      applyCoreStatus(status);
    } catch (error) {
      setConnection({ kind: "error", error: asDesktopError(error) });
    }
  }

  async function chooseRepository() {
    const previous = repository.kind === "open" ? repository : null;
    if (!previous) setRepository({ kind: "selecting" });
    setWorkspaceError(null);
    try {
      const path = environment === "local" ? await selectRepositoryDirectory() : remoteRepositoryPath.trim();
      if (path === null) {
        if (!previous) setRepository({ kind: "idle" });
        return;
      }
      if (path.length === 0) {
        throw { code: "desktop.remote_path_required", message: "Enter the repository path in the Core environment", retryable: false } satisfies DesktopError;
      }
      const opened = await openRepository(path);
      rememberRepository({ target: launchTarget(), path });
      await activateRepository(opened);
    } catch (error) {
      const desktopError = asDesktopError(error);
      if (previous) {
        setRepository(previous);
        setWorkspaceError(desktopError);
      } else setRepository({ kind: "error", error: desktopError });
    }
  }

  async function switchRepository(bookmark: RepositoryBookmark) {
    if (repository.kind === "open" && bookmark.path === (repository.repository.worktreeRoot ?? repository.repository.gitDirectory) && bookmarkKey({ target: launchTarget(), path: bookmark.path }) === bookmarkKey(bookmark)) return;
    const previous = repository.kind === "open" ? repository : null;
    const previousTarget = launchTarget();
    const previousPath = previous ? previous.repository.worktreeRoot ?? previous.repository.gitDirectory : null;
    setWorkspaceError(null);
    try {
      await ensureCoreTarget(bookmark.target);
      const opened = await openRepository(bookmark.path);
      showTarget(bookmark.target, bookmark.path);
      rememberRepository(bookmark);
      await activateRepository(opened);
    } catch (error) {
      const desktopError = asDesktopError(error);
      if (previous) {
        try {
          if (targetKey(previousTarget) !== targetKey(bookmark.target)) {
            await ensureCoreTarget(previousTarget);
            if (previousPath) await openRepository(previousPath);
          }
          setRepository(previous);
          setWorkspaceError(desktopError);
        } catch (rollbackError) {
          setRepository({ kind: "error", error: asDesktopError(rollbackError) });
        }
      } else setRepository({ kind: "error", error: desktopError });
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
      rememberRepository({ target: launchTarget(), path });
      await activateRepository(opened);
    } catch (error) {
      setRepository({ kind: "error", error: asDesktopError(error) });
    }
  }

  async function refreshReferences() {
    const request = ++referencesRequest.current;
    setReferences({ kind: "loading" });
    try {
      const value = await getRepositoryReferences();
      if (request === referencesRequest.current) setReferences({ kind: "ready", value });
    } catch (error) {
      if (request === referencesRequest.current) setReferences({ kind: "error", error: asDesktopError(error) });
    }
  }

  function reviewBranchSwitch(name: string) {
    setBranchOperation({ kind: "confirm", name });
  }

  async function confirmBranchSwitch(name: string) {
    setBranchOperation({ kind: "loading", name });
    try {
      const snapshot = await switchLocalBranch(name);
      setReferences({ kind: "ready", value: snapshot.references });
      applyMutation(snapshot);
      setBranchOperation({ kind: "idle" });
    } catch (error) {
      setBranchOperation({ kind: "error", name, error: asDesktopError(error) });
    }
  }

  async function refreshWorkingTree() {
    const request = ++workingTreeRequest.current;
    diffRequest.current += 1;
    setFileDiff({ kind: "idle" });
    setWorkingTree({ kind: "loading" });
    try {
      const status = await getWorkingTreeStatus();
      if (request === workingTreeRequest.current) setWorkingTree({ kind: "ready", status });
    } catch (error) {
      if (request === workingTreeRequest.current) setWorkingTree({ kind: "error", error: asDesktopError(error) });
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
    setHistoryDetailTab("commit");
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
    workingTreeRequest.current += 1;
    diffRequest.current += 1;
    setFileDiff({ kind: "idle" });
    commitRequest.current += 1;
    setCommitDetail({ kind: "idle" });
    setWorkingTree({ kind: "ready", status: snapshot.status });
    setReferences({ kind: "ready", value: snapshot.references });
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
  const changeCount = workingTree.kind === "ready" ? workingTree.status.entries.length : 0;
  const branchName = workingTree.kind === "ready" ? workingTree.status.branch.head ?? "Detached HEAD" : "Reading branch…";
  const selectedCommitOid = commitDetail.kind === "idle" ? null : commitDetail.kind === "choosingParent" ? commitDetail.commit.oid : commitDetail.selection.commit.oid;

  return (
    <div className="app-shell">
      <header className="app-header app-toolbar">
        <a className="brand" href="#main-content" aria-label="GitNova home">
          <img src={markUrl} alt="" width="30" height="30" />
          <span>GitNova</span>
        </a>
        {openedRepository ? (
          <div className="toolbar-context">
            <label className="toolbar-repository">
              <span className="sr-only">Current repository</span>
              <select aria-label="Current repository" value={bookmarkKey({ target: launchTarget(), path: repositoryPath })} onChange={(event) => {
                const selected = recentRepositories.find((entry) => bookmarkKey(entry) === event.target.value);
                if (selected) void switchRepository(selected);
              }}>
                {recentRepositories.map((entry) => <option key={bookmarkKey(entry)} value={bookmarkKey(entry)}>{repositoryLabel(entry.path)} — {entry.path}</option>)}
              </select>
            </label>
            {connection.kind === "connected" && (connection.references || connection.mutations) && openedRepository.kind !== "bare"
              ? <BranchSwitcher currentBranch={workingTree.kind === "ready" ? workingTree.status.branch.head : null} references={references} canSwitch={connection.mutations} switching={branchOperation.kind === "loading"} onSelect={reviewBranchSwitch} />
              : <span className="toolbar-branch-label">{branchName}</span>}
          </div>
        ) : <span className="toolbar-title">Local-first Git client</span>}
        <div className="toolbar-actions">
          {openedRepository && <button type="button" onClick={() => void chooseRepository()}>Add repository</button>}
          {openedRepository && openedRepository.kind !== "bare" && (
            <button type="button" onClick={() => void refreshWorkingTree()}>Refresh repository</button>
          )}
          {openedRepository && <button type="button" onClick={() => void reopenRepository()}>Reopen repository</button>}
        </div>
      </header>

      {(branchOperation.kind === "confirm" || branchOperation.kind === "loading" || branchOperation.kind === "error") && <div className="branch-confirmation" role="group" aria-label="Confirm branch switch">
        <span>Switch to <strong>{branchOperation.name}</strong>? Working changes will be kept; GitNova will not stash or discard them.</span>
        {branchOperation.kind === "error" && <span role="alert">{branchOperation.error.message}</span>}
        <div><button type="button" disabled={branchOperation.kind === "loading"} onClick={() => setBranchOperation({ kind: "idle" })}>Cancel</button><button type="button" disabled={branchOperation.kind === "loading"} onClick={() => void confirmBranchSwitch(branchOperation.name)}>{branchOperation.kind === "loading" ? "Switching…" : branchOperation.kind === "error" ? "Retry" : "Switch branch"}</button></div>
      </div>}

      {openedRepository ? (
        <main id="main-content" className="repository-workbench" tabIndex={-1}>
          <aside className="repository-sidebar" aria-label="Repository navigation">
            <nav>
              {openedRepository.kind !== "bare" && <button type="button" className={workspaceView === "changes" ? "is-active" : ""} onClick={() => setWorkspaceView("changes")}><span>Local Changes</span><strong>{changeCount}</strong></button>}
              <button type="button" className={workspaceView === "history" ? "is-active" : ""} onClick={() => setWorkspaceView("history")}><span>All Commits</span></button>
              <button type="button" className={workspaceView === "pullRequests" ? "is-active" : ""} onClick={() => setWorkspaceView("pullRequests")}><span>Pull Requests</span></button>
              <button type="button" className={workspaceView === "settings" ? "is-active" : ""} onClick={() => setWorkspaceView("settings")}><span>Settings</span></button>
            </nav>
            <RepositoryRefTree state={references} currentBranch={workingTree.kind === "ready" ? workingTree.status.branch.head : null} canSwitch={connection.kind === "connected" && connection.mutations} onSwitch={reviewBranchSwitch} />
            <dl className="repository-facts">
              <div><dt>Core</dt><dd>{coreDetail}</dd></div>
              <div><dt>System Git</dt><dd>{openedRepository.gitVersion}</dd></div>
              <div><dt>Path</dt><dd title={repositoryPath}>{repositoryPath}</dd></div>
            </dl>
            <p className="privacy-note">Repository data stays in the Core environment.</p>
          </aside>

          <section className="workbench-main">
            {workspaceError && <div className="workspace-error" role="alert"><span>{workspaceError.message}</span><button type="button" onClick={() => setWorkspaceError(null)}>Dismiss</button></div>}
            <header className="view-header">
              <h1>{workspaceView === "changes" ? "Local Changes" : workspaceView === "history" ? "All Commits" : workspaceView === "pullRequests" ? "Pull Requests" : "Settings"}</h1>
            </header>

            {workspaceView === "changes" && openedRepository.kind !== "bare" && (
              <div className="changes-workspace">
                <div className="changes-browser">
                  <WorkingTreePanel state={workingTree} diffLoading={fileDiff.kind === "loading"} selection={fileDiff.kind === "idle" ? null : fileDiff.selection} onDiff={(path: string, scope: DiffScope) => void loadFileDiff({ path, scope })} />
                </div>
                <div className="changes-detail">
                  {fileDiff.kind === "idle" ? <div className="pane-placeholder"><strong>Select a changed file</strong><span>Click a file name under Unstaged or Staged to inspect that diff.</span></div> : <DiffPanel state={fileDiff} onRetry={() => void loadFileDiff(fileDiff.selection)} onClose={closeFileDiff} />}
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
                <HistoryPanel state={history} selectedOid={selectedCommitOid} commitLoading={commitDetail.kind === "loading"} onRetry={() => void refreshHistory()} onLoadMore={() => void loadMoreHistory()} onSelectCommit={selectCommit} />
                <div className="history-detail">
                  <div className="history-detail-tabs" role="tablist" aria-label="Commit detail views">
                    <button type="button" role="tab" aria-selected={historyDetailTab === "commit"} className={historyDetailTab === "commit" ? "is-active" : ""} onClick={() => setHistoryDetailTab("commit")}>Commit</button>
                    <button type="button" role="tab" aria-selected={historyDetailTab === "changes"} className={historyDetailTab === "changes" ? "is-active" : ""} onClick={() => setHistoryDetailTab("changes")}>Changes</button>
                  </div>
                  <div className="history-detail-content">
                    {commitDetail.kind === "idle" ? <div className="pane-placeholder"><strong>Select a commit</strong><span>Commit metadata and line-level changes will appear here.</span></div> : <CommitDetailPanel key={`${commitDetail.kind === "choosingParent" ? commitDetail.commit.oid : commitDetail.selection.commit.oid}:${commitDetail.kind === "choosingParent" ? "" : commitDetail.selection.parentOid ?? ""}`} state={commitDetail} mode={historyDetailTab} onChooseParent={chooseCommitParent} onRetry={() => commitDetail.kind === "error" && void loadCommitDiff(commitDetail.selection)} onClose={closeCommitDetail} />}
                  </div>
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
