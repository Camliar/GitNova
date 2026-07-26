import type { RepositoryReferences } from "@gitnova/protocol";
import type { DesktopError } from "./core";

export type ReferencesState =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "ready"; value: RepositoryReferences }
  | { kind: "error"; error: DesktopError };

export function BranchSwitcher({ currentBranch, references, canSwitch, switching, onSelect }: { currentBranch: string | null; references: ReferencesState; canSwitch: boolean; switching: boolean; onSelect: (name: string) => void }) {
  const all = references.kind === "ready" ? references.value.references : [];
  const branches = all.filter((reference) => reference.kind === "localBranch");
  const remoteBranches = all.filter((reference) => reference.kind === "remoteBranch");

  return <div className="branch-switcher">
    <label>
      <span className="sr-only">Current branch</span>
      <select
        aria-label="Current branch"
        value={currentBranch ?? ""}
        disabled={switching || references.kind === "loading"}
        onChange={(event) => { if (canSwitch && event.target.value && event.target.value !== currentBranch) onSelect(event.target.value); }}
      >
        {!currentBranch && <option value="">Detached HEAD</option>}
        {currentBranch && !branches.some((branch) => branch.name === currentBranch) && <option value={currentBranch}>{currentBranch}</option>}
        <optgroup label="Local branches">
          {branches.map((branch) => <option key={branch.fullName} value={branch.name} disabled={!canSwitch && branch.name !== currentBranch}>{branch.name}{branch.upstream ? ` → ${branch.upstream}` : ""}</option>)}
        </optgroup>
        {remoteBranches.length > 0 && <optgroup label="Remote branches">
          {remoteBranches.map((branch) => <option key={branch.fullName} value={`remote:${branch.fullName}`} disabled>{branch.name}</option>)}
        </optgroup>}
      </select>
    </label>
    {references.kind === "error" && <span className="branch-switcher__error" role="alert" title={references.error.message}>Branches unavailable</span>}
  </div>;
}
