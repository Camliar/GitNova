import { useEffect, useRef, useState } from "react";
import type { BranchStatus, RepositoryMutationSnapshot, RepositorySyncOperation } from "@gitnova/protocol";
import { asDesktopError, type DesktopError } from "./core";
import { fetchRepository, pullRepository, pushRepository } from "./sync";

type ConfirmedAction = Exclude<RepositorySyncOperation, "fetch">;
type SyncState =
  | { kind: "idle" }
  | { kind: "confirm"; action: ConfirmedAction; branch: string; oid: string }
  | { kind: "loading"; action: RepositorySyncOperation }
  | { kind: "error"; action: RepositorySyncOperation; error: DesktopError }
  | { kind: "success"; message: string };

export function RepositorySyncControls({ branch, onApplied }: { branch: BranchStatus; onApplied: (snapshot: RepositoryMutationSnapshot) => void }) {
  const [state, setState] = useState<SyncState>({ kind: "idle" });
  const active = useRef(true);
  const request = useRef(0);
  useEffect(() => () => { active.current = false; request.current += 1; }, []);
  const attached = branch.head !== null && branch.oid !== null;
  const busy = state.kind === "loading";

  function review(action: ConfirmedAction) {
    if (!branch.head || !branch.oid) return;
    setState({ kind: "confirm", action, branch: branch.head, oid: branch.oid });
  }

  async function execute(action: RepositorySyncOperation, confirmed?: { branch: string; oid: string }) {
    const serial = ++request.current;
    setState({ kind: "loading", action });
    try {
      const result = action === "fetch"
        ? await fetchRepository()
        : action === "pull"
          ? await pullRepository(confirmed?.branch ?? "", confirmed?.oid ?? "")
          : await pushRepository(confirmed?.branch ?? "", confirmed?.oid ?? "");
      if (!active.current || serial !== request.current) return;
      onApplied(result.snapshot);
      setState({ kind: "success", message: `${label(action)} ${result.remote}/${result.remoteBranch} complete` });
    } catch (error) {
      if (active.current && serial === request.current) setState({ kind: "error", action, error: asDesktopError(error) });
    }
  }

  return <div className="repository-sync" aria-label="Repository sync actions">
    <button type="button" disabled={busy || !attached} onClick={() => void execute("fetch")}>Fetch</button>
    <button type="button" disabled={busy || !attached || !branch.upstream} onClick={() => review("pull")} title={branch.upstream ? "Pull the tracked branch" : "Pull requires an upstream branch"}>Pull</button>
    <button type="button" disabled={busy || !attached} onClick={() => review("push")}>Push</button>
    {state.kind === "confirm" && <div className="sync-confirmation" role="group" aria-label={`Confirm ${state.action}`}>
      <strong>{label(state.action)} {state.branch} at {state.oid.slice(0, 8)}?</strong>
      <span>{state.action === "pull" ? "Only a fast-forward is allowed. GitNova will not merge, rebase, stash, reset, or discard changes." : "Only the current branch is pushed. GitNova will not force, delete, or push another ref."}</span>
      <div><button type="button" onClick={() => setState({ kind: "idle" })}>Cancel</button><button type="button" onClick={() => void execute(state.action, state)}>Confirm {state.action}</button></div>
    </div>}
    {state.kind === "loading" && <span className="sync-feedback" role="status">{label(state.action)}…</span>}
    {state.kind === "error" && <span className="sync-feedback sync-feedback--error" role="alert">{state.error.message} <button type="button" onClick={() => state.action === "fetch" ? void execute("fetch") : review(state.action)}>Retry</button></span>}
    {state.kind === "success" && <span className="sync-feedback" role="status">{state.message}</span>}
  </div>;
}

function label(action: RepositorySyncOperation) {
  return action.slice(0, 1).toUpperCase() + action.slice(1);
}
