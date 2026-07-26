import { useEffect, useState, type MouseEvent } from "react";
import type { RepositoryReference, RepositoryReferences } from "@gitnova/protocol";
import type { DesktopError } from "./core";

export type ReferencesState =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "ready"; value: RepositoryReferences }
  | { kind: "error"; error: DesktopError };

type ContextMenu = { reference: RepositoryReference; x: number; y: number };
type RefActions = {
  currentBranch: string | null;
  canSwitch: boolean;
  canCheckoutRemote: boolean;
  onSwitch: (name: string) => void;
  onCheckoutRemote: (fullName: string, displayName: string) => void;
  onMenu: (event: MouseEvent, reference: RepositoryReference) => void;
};

function actionable(reference: RepositoryReference, actions: RefActions) {
  return (reference.kind === "localBranch" && actions.canSwitch)
    || (reference.kind === "remoteBranch" && reference.symbolicTarget === null && actions.canCheckoutRemote);
}

function RefGroup({ title, references, actions }: { title: string; references: RepositoryReference[]; actions: RefActions }) {
  if (references.length === 0) return null;
  return <details className="ref-group" open>
    <summary>{title}<span>{references.length}</span></summary>
    <ul>
      {references.map((reference) => {
        const current = reference.kind === "localBranch" && reference.name === actions.currentBranch;
        const hasActions = actionable(reference, actions);
        return <li key={reference.fullName}><div className="ref-row" onContextMenu={hasActions ? (event) => actions.onMenu(event, reference) : undefined}>
          {reference.kind === "localBranch" && actions.canSwitch && !current
            ? <button className="ref-row__primary" type="button" title={`Switch to ${reference.name}`} onClick={() => actions.onSwitch(reference.name)}><span aria-hidden="true">⎇</span><span>{reference.name}</span>{reference.upstream && <small>{reference.upstream}</small>}</button>
            : <span className={`ref-row__primary ${current ? "is-current" : ""}`} aria-label={current ? `Current branch ${reference.name}` : undefined}><span aria-hidden="true">{current ? "✓" : reference.kind === "tag" ? "◇" : "⎇"}</span><span>{reference.name}</span>{reference.upstream && <small>{reference.upstream}</small>}</span>}
          {hasActions && <button className="ref-row__actions" type="button" aria-label={`Actions for ${reference.name}`} aria-haspopup="menu" onClick={(event) => actions.onMenu(event, reference)}>•••</button>}
        </div></li>;
      })}
    </ul>
  </details>;
}

export function RepositoryRefTree({ state, currentBranch, canSwitch, canCheckoutRemote, onSwitch, onCheckoutRemote }: { state: ReferencesState; currentBranch: string | null; canSwitch: boolean; canCheckoutRemote: boolean; onSwitch: (name: string) => void; onCheckoutRemote: (fullName: string, displayName: string) => void }) {
  const [menu, setMenu] = useState<ContextMenu | null>(null);
  useEffect(() => {
    if (!menu) return;
    const close = () => setMenu(null);
    const keydown = (event: KeyboardEvent) => { if (event.key === "Escape") close(); };
    window.addEventListener("click", close);
    window.addEventListener("keydown", keydown);
    return () => { window.removeEventListener("click", close); window.removeEventListener("keydown", keydown); };
  }, [menu]);
  if (state.kind === "idle") return null;
  if (state.kind === "loading") return <p className="ref-tree-state">Reading references…</p>;
  if (state.kind === "error") return <p className="ref-tree-state ref-tree-state--error" role="alert">References unavailable</p>;
  const openMenu = (event: MouseEvent, reference: RepositoryReference) => {
    event.preventDefault();
    event.stopPropagation();
    const target = event.currentTarget.getBoundingClientRect();
    const requestedX = event.clientX || target.right;
    const requestedY = event.clientY || target.bottom;
    setMenu({ reference, x: Math.max(4, Math.min(requestedX, window.innerWidth - 230)), y: Math.max(4, Math.min(requestedY, window.innerHeight - 70)) });
  };
  const actions: RefActions = { currentBranch, canSwitch, canCheckoutRemote, onSwitch, onCheckoutRemote, onMenu: openMenu };
  const local = state.value.references.filter((reference) => reference.kind === "localBranch");
  const remote = state.value.references.filter((reference) => reference.kind === "remoteBranch");
  const tags = state.value.references.filter((reference) => reference.kind === "tag");
  return <div className="repository-ref-tree" aria-label="Repository references">
    <RefGroup title="Branches" references={local} actions={actions} />
    <RefGroup title="Remotes" references={remote} actions={actions} />
    <RefGroup title="Tags" references={tags} actions={actions} />
    {menu && <div className="ref-context-menu" role="menu" aria-label={`Branch actions for ${menu.reference.name}`} style={{ left: menu.x, top: menu.y }} onClick={(event) => event.stopPropagation()}>
      {menu.reference.kind === "localBranch" && menu.reference.name === currentBranch
        ? <button type="button" role="menuitem" disabled>Current branch</button>
        : menu.reference.kind === "localBranch"
          ? <button type="button" role="menuitem" onClick={() => { setMenu(null); onSwitch(menu.reference.name); }}>Checkout</button>
          : <button type="button" role="menuitem" onClick={() => { setMenu(null); onCheckoutRemote(menu.reference.fullName, menu.reference.name); }}>Checkout as local tracking branch</button>}
    </div>}
  </div>;
}
