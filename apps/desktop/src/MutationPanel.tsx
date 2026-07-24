import { useEffect, useRef, useState } from "react";
import type { RepositoryMutationSnapshot, RepositoryReferences, WorkingTreeStatus } from "@gitnova/protocol";
import { asDesktopError, type DesktopError } from "./core";
import { commitStaged, createLocalBranch, getRepositoryReferences, switchLocalBranch } from "./mutations";

type Pending = { kind: "commit"; message: string } | { kind: "create"; name: string } | { kind: "switch"; name: string };
type Operation = { kind: "idle" } | { kind: "confirm"; pending: Pending } | { kind: "loading"; pending: Pending } | { kind: "error"; pending: Pending; error: DesktopError } | { kind: "success"; message: string };

export function MutationPanel({ status, onApplied }: { status: WorkingTreeStatus; onApplied: (snapshot: RepositoryMutationSnapshot) => void }) {
  const [message, setMessage] = useState("");
  const [branchName, setBranchName] = useState("");
  const [switchName, setSwitchName] = useState("");
  const [references, setReferences] = useState<{ kind: "loading" } | { kind: "ready"; value: RepositoryReferences } | { kind: "error"; error: DesktopError }>({ kind: "loading" });
  const [operation, setOperation] = useState<Operation>({ kind: "idle" });
  const serial = useRef(0);
  const active = useRef(true);
  useEffect(() => { const current = ++serial.current; void getRepositoryReferences().then((value) => { if (active.current && current === serial.current) setReferences({ kind: "ready", value }); }).catch((error) => { if (active.current && current === serial.current) setReferences({ kind: "error", error: asDesktopError(error) }); }); return () => { active.current = false; serial.current += 1; }; }, []);

  const stagedCount = status.entries.filter((entry) => entry.indexStatus !== "unmodified").length;
  const localBranches = references.kind === "ready" ? references.value.references.filter((reference) => reference.kind === "localBranch") : [];
  const busy = operation.kind === "loading";

  function review(pending: Pending) {
    setOperation({ kind: "confirm", pending });
  }

  async function execute(pending: Pending) {
    setOperation({ kind: "loading", pending });
    try {
      if (pending.kind === "commit") {
        const result = await commitStaged(pending.message);
        if (!active.current) return;
        setMessage("");
        setReferences({ kind: "ready", value: result.snapshot.references });
        onApplied(result.snapshot);
        setOperation({ kind: "success", message: `Created commit ${result.commit.oid.slice(0, 8)}` });
      } else {
        const snapshot = pending.kind === "create" ? await createLocalBranch(pending.name) : await switchLocalBranch(pending.name);
        if (!active.current) return;
        if (pending.kind === "create") setBranchName("");
        setReferences({ kind: "ready", value: snapshot.references });
        onApplied(snapshot);
        setOperation({ kind: "success", message: pending.kind === "create" ? `Created branch ${pending.name}` : `Switched to ${pending.name}` });
      }
    } catch (error) {
      if (active.current) setOperation({ kind: "error", pending, error: asDesktopError(error) });
    }
  }

  return <section className="mutation-panel" aria-labelledby="mutation-title" aria-busy={busy}>
    <header><p className="eyebrow">Explicit Git actions</p><h2 id="mutation-title">Commit & branches</h2></header>
    <div className="mutation-grid">
      <form onSubmit={(event) => { event.preventDefault(); if (message.trim()) review({ kind: "commit", message }); }}>
        <h3>Commit staged changes</h3><p>{stagedCount} staged path{stagedCount === 1 ? "" : "s"}. Unstaged and untracked files will not be added.</p>
        <label htmlFor="commit-message">Commit message</label><textarea id="commit-message" value={message} onChange={(event) => setMessage(event.target.value)} disabled={busy} />
        <button type="submit" disabled={busy || !message.trim()}>Review commit</button>
      </form>
      <form onSubmit={(event) => { event.preventDefault(); if (branchName) review({ kind: "create", name: branchName }); }}>
        <h3>Create local branch</h3><p>Creates at current HEAD without switching.</p>
        <label htmlFor="new-branch">Branch name</label><input id="new-branch" value={branchName} onChange={(event) => setBranchName(event.target.value)} disabled={busy} />
        <button type="submit" disabled={busy || !branchName}>Review branch creation</button>
      </form>
      <form onSubmit={(event) => { event.preventDefault(); if (switchName) review({ kind: "switch", name: switchName }); }}>
        <h3>Switch local branch</h3><p>Core will not stash, force, or discard working changes.</p>
        <label htmlFor="switch-branch">Local branch</label><select id="switch-branch" value={switchName} onChange={(event) => setSwitchName(event.target.value)} disabled={busy || references.kind !== "ready"}><option value="">Choose branch</option>{localBranches.map((branch) => <option key={branch.fullName} value={branch.name}>{branch.name}</option>)}</select>
        <button type="submit" disabled={busy || !switchName}>Review branch switch</button>
      </form>
    </div>
    {references.kind === "loading" && <p className="empty-state" role="status">Reading local branches from Core…</p>}
    {references.kind === "error" && <p className="mutation-error" role="alert">{references.error.message}. Branch choices are unavailable.</p>}
    {(operation.kind === "confirm" || operation.kind === "loading") && <div className="mutation-confirm" role="group" aria-label="Confirm Git action"><p>{pendingDescription(operation.pending)}</p><span><button type="button" disabled={busy} onClick={() => setOperation({ kind: "idle" })}>Cancel</button><button type="button" disabled={busy} onClick={() => void execute(operation.pending)}>{busy ? "Applying…" : "Confirm action"}</button></span></div>}
    {operation.kind === "error" && <div className="mutation-confirm mutation-confirm--error"><p role="alert">{operation.error.message}. No success was recorded.</p><span><button type="button" onClick={() => setOperation({ kind: "idle" })}>Cancel</button><button type="button" onClick={() => void execute(operation.pending)}>Retry action</button></span></div>}
    {operation.kind === "success" && <p className="mutation-success" role="status">{operation.message}</p>}
  </section>;
}

function pendingDescription(pending: Pending) {
  if (pending.kind === "commit") return `Confirm committing the current staged index with message: “${pending.message}”`;
  if (pending.kind === "create") return `Confirm creating local branch “${pending.name}” at current HEAD without switching.`;
  return `Confirm switching to local branch “${pending.name}”. Working changes will not be stashed or discarded.`;
}
