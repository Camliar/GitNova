import type { RepositoryReference } from "@gitnova/protocol";
import type { ReferencesState } from "./BranchSwitcher";

function RefGroup({ title, references, currentBranch, canSwitch, onSwitch }: { title: string; references: RepositoryReference[]; currentBranch: string | null; canSwitch: boolean; onSwitch: (name: string) => void }) {
  if (references.length === 0) return null;
  return <details className="ref-group" open>
    <summary>{title}<span>{references.length}</span></summary>
    <ul>
      {references.map((reference) => {
        const current = reference.kind === "localBranch" && reference.name === currentBranch;
        return <li key={reference.fullName}>
          {reference.kind === "localBranch" && canSwitch && !current
            ? <button type="button" title={`Switch to ${reference.name}`} onClick={() => onSwitch(reference.name)}><span aria-hidden="true">⎇</span><span>{reference.name}</span>{reference.upstream && <small>{reference.upstream}</small>}</button>
            : <span className={current ? "is-current" : ""}><span aria-hidden="true">{current ? "✓" : reference.kind === "tag" ? "◇" : "⎇"}</span><span>{reference.name}</span>{reference.upstream && <small>{reference.upstream}</small>}</span>}
        </li>;
      })}
    </ul>
  </details>;
}

export function RepositoryRefTree({ state, currentBranch, canSwitch, onSwitch }: { state: ReferencesState; currentBranch: string | null; canSwitch: boolean; onSwitch: (name: string) => void }) {
  if (state.kind === "idle") return null;
  if (state.kind === "loading") return <p className="ref-tree-state">Reading references…</p>;
  if (state.kind === "error") return <p className="ref-tree-state ref-tree-state--error" role="alert">References unavailable</p>;
  const local = state.value.references.filter((reference) => reference.kind === "localBranch");
  const remote = state.value.references.filter((reference) => reference.kind === "remoteBranch");
  const tags = state.value.references.filter((reference) => reference.kind === "tag");
  return <div className="repository-ref-tree" aria-label="Repository references">
    <RefGroup title="Branches" references={local} currentBranch={currentBranch} canSwitch={canSwitch} onSwitch={onSwitch} />
    <RefGroup title="Remotes" references={remote} currentBranch={currentBranch} canSwitch={false} onSwitch={onSwitch} />
    <RefGroup title="Tags" references={tags} currentBranch={currentBranch} canSwitch={false} onSwitch={onSwitch} />
  </div>;
}
