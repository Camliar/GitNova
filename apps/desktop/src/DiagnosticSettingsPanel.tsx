import { useEffect, useState } from "react";
import { getDiagnosticInfo, type DiagnosticInfo } from "./diagnostics";

type DiagnosticState =
  | { kind: "loading" }
  | { kind: "ready"; info: DiagnosticInfo }
  | { kind: "unavailable" };

export function DiagnosticSettingsPanel() {
  const [state, setState] = useState<DiagnosticState>({ kind: "loading" });
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    let active = true;
    void getDiagnosticInfo()
      .then((info) => {
        if (active) setState(info ? { kind: "ready", info } : { kind: "unavailable" });
      })
      .catch(() => {
        if (active) setState({ kind: "unavailable" });
      });
    return () => { active = false; };
  }, []);

  const copyPath = async () => {
    if (state.kind !== "ready") return;
    try {
      await navigator.clipboard.writeText(state.info.path);
      setCopied(true);
    } catch {
      setCopied(false);
    }
  };

  return <section className="settings-panel" aria-labelledby="diagnostics-settings-title">
    <header><p className="eyebrow">Troubleshooting</p><h2 id="diagnostics-settings-title">Diagnostic log</h2></header>
    <p className="settings-intro">Local JSONL diagnostics record Core lifecycle, RPC method, duration and stable error codes. Repository paths, commit content, diffs, request payloads, Provider responses, stderr and credentials are never logged.</p>
    {state.kind === "loading" && <p className="diagnostics-status">Locating diagnostic log…</p>}
    {state.kind === "unavailable" && <p className="diagnostics-status">The diagnostic path is available in the installed Desktop app.</p>}
    {state.kind === "ready" && <div className="diagnostics-location">
      <div><strong>Active log</strong><code title={state.info.path}>{state.info.path}</code></div>
      <button type="button" onClick={() => void copyPath()}>{copied ? "Copied" : "Copy path"}</button>
      <p>Rotates at {Math.round(state.info.maxBytes / 1024 / 1024)} MiB · keeps {state.info.retainedFiles} files · never uploaded automatically</p>
    </div>}
  </section>;
}
