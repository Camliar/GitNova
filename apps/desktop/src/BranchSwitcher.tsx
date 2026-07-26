import { useEffect, useState } from "react";
import type { RepositoryMutationSnapshot, RepositoryReferences } from "@gitnova/protocol";
import { asDesktopError, type DesktopError } from "./core";
import { getRepositoryReferences, switchLocalBranch } from "./mutations";

type ReferencesState =
  | { kind: "loading" }
  | { kind: "ready"; value: RepositoryReferences }
  | { kind: "error"; error: DesktopError };

export function BranchSwitcher({ currentBranch, disabled = false, onApplied }: { currentBranch: string | null; disabled?: boolean; onApplied: (snapshot: RepositoryMutationSnapshot) => void }) {
  const [references, setReferences] = useState<ReferencesState>({ kind: "loading" });
  const [pending, setPending] = useState<string | null>(null);
  const [switching, setSwitching] = useState(false);
  const [error, setError] = useState<DesktopError | null>(null);

  useEffect(() => {
    let active = true;
    setReferences({ kind: "loading" });
    void getRepositoryReferences()
      .then((value) => { if (active) setReferences({ kind: "ready", value }); })
      .catch((reason) => { if (active) setReferences({ kind: "error", error: asDesktopError(reason) }); });
    return () => { active = false; };
  }, [currentBranch]);

  const branches = references.kind === "ready" ? references.value.references.filter((reference) => reference.kind === "localBranch") : [];
  const remoteBranches = references.kind === "ready" ? references.value.references.filter((reference) => reference.kind === "remoteBranch") : [];

  async function confirmSwitch() {
    if (!pending) return;
    setSwitching(true);
    setError(null);
    try {
      const snapshot = await switchLocalBranch(pending);
      setReferences({ kind: "ready", value: snapshot.references });
      setPending(null);
      onApplied(snapshot);
    } catch (reason) {
      setError(asDesktopError(reason));
    } finally {
      setSwitching(false);
    }
  }

  return <div className="branch-switcher">
    <label>
      <span className="sr-only">Current branch</span>
      <select
        aria-label="Current branch"
        value={currentBranch ?? ""}
        disabled={disabled || switching || references.kind !== "ready"}
        onChange={(event) => { if (event.target.value && event.target.value !== currentBranch) setPending(event.target.value); }}
      >
        {!currentBranch && <option value="">Detached HEAD</option>}
        {currentBranch && !branches.some((branch) => branch.name === currentBranch) && <option value={currentBranch}>{currentBranch}</option>}
        <optgroup label="Local branches">
          {branches.map((branch) => <option key={branch.fullName} value={branch.name}>{branch.name}{branch.upstream ? ` → ${branch.upstream}` : ""}</option>)}
        </optgroup>
        {remoteBranches.length > 0 && <optgroup label="Remote branches">
          {remoteBranches.map((branch) => <option key={branch.fullName} value={`remote:${branch.fullName}`} disabled>{branch.name}</option>)}
        </optgroup>}
      </select>
    </label>
    {references.kind === "error" && <span className="branch-switcher__error" role="alert" title={references.error.message}>Branches unavailable</span>}
    {pending && <div className="branch-switcher__confirm" role="group" aria-label="Confirm branch switch">
      <span>Switch to <strong>{pending}</strong>? Working changes will be kept.</span>
      {error && <span role="alert">{error.message}</span>}
      <div><button type="button" disabled={switching} onClick={() => { setPending(null); setError(null); }}>Cancel</button><button type="button" disabled={switching} onClick={() => void confirmSwitch()}>{switching ? "Switching…" : "Switch branch"}</button></div>
    </div>}
  </div>;
}
