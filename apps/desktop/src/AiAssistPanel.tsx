import { useEffect, useRef, useState } from "react";
import type { AiCommitDraft, AiInputPreview, AiProviderConfig } from "@gitnova/protocol";
import { generateAiCommitDraft, previewAiInput } from "./ai";
import { asDesktopError, type DesktopError } from "./core";
import type { AiAssistSettings } from "./AiSettingsPanel";

type Operation = "idle" | "previewing" | "generating";

export function AiAssistPanel({ settings, onUseCommitMessage }: { settings: AiAssistSettings; onUseCommitMessage: (message: string) => void }) {
  const [preview, setPreview] = useState<AiInputPreview | null>(null);
  const [draft, setDraft] = useState<AiCommitDraft | null>(null);
  const [draftMessage, setDraftMessage] = useState("");
  const [externalConfirmed, setExternalConfirmed] = useState(false);
  const [operation, setOperation] = useState<Operation>("idle");
  const [error, setError] = useState<DesktopError | null>(null);
  const [handoff, setHandoff] = useState(false);
  const serial = useRef(0);

  useEffect(() => invalidate(), [settings.providerKind, settings.model, settings.baseUrl, settings.excludedText]);

  function invalidate() {
    serial.current += 1;
    setPreview(null);
    setDraft(null);
    setDraftMessage("");
    setExternalConfirmed(false);
    setError(null);
    setHandoff(false);
    setOperation("idle");
  }

  function exclusions() {
    return [...new Set(settings.excludedText.split(/\r?\n/).map((path) => path.trim()).filter(Boolean))];
  }

  function provider(): AiProviderConfig {
    const model = settings.model.trim();
    switch (settings.providerKind) {
      case "ollama":
        return { kind: "ollama", model, ...(settings.baseUrl.trim() ? { baseUrl: settings.baseUrl.trim() } : {}) };
      case "openAi":
        return { kind: "openAi", model };
      case "anthropic":
        return { kind: "anthropic", model };
      case "deepSeek":
        return { kind: "deepSeek", model };
      case "qwen":
        return { kind: "qwen", model };
      case "kimi":
        return { kind: "kimi", model };
    }
  }

  async function runPreview() {
    const request = ++serial.current;
    setPreview(null);
    setDraft(null);
    setDraftMessage("");
    setExternalConfirmed(false);
    setError(null);
    setHandoff(false);
    setOperation("previewing");
    try {
      const result = await previewAiInput(provider(), exclusions());
      if (request !== serial.current) return;
      setPreview(result);
      setOperation("idle");
    } catch (caught) {
      if (request !== serial.current) return;
      setError(asDesktopError(caught));
      setOperation("idle");
    }
  }

  async function runGenerate() {
    if (!preview) return;
    const request = ++serial.current;
    setError(null);
    setHandoff(false);
    setOperation("generating");
    try {
      const result = await generateAiCommitDraft(
        preview.previewId,
        provider(),
        exclusions(),
        externalConfirmed,
      );
      if (request !== serial.current) return;
      setDraft(result);
      setDraftMessage(result.commitMessage);
      setOperation("idle");
    } catch (caught) {
      if (request !== serial.current) return;
      const desktopError = asDesktopError(caught);
      setError(desktopError);
      if (desktopError.code === "ai.preview_stale") {
        setPreview(null);
        setExternalConfirmed(false);
      }
      setOperation("idle");
    }
  }

  const busy = operation !== "idle";
  const canPreview = !busy && settings.model.trim().length > 0 && (settings.providerKind !== "ollama" || settings.baseUrl.trim().length > 0);
  const canGenerate =
    !busy &&
    preview !== null &&
    (!preview.externalConfirmationRequired || externalConfirmed);

  return (
    <section className="ai-panel" aria-labelledby="ai-title" aria-busy={busy}>
      <header>
        <div><p className="eyebrow">Explicit AI action</p><h2 id="ai-title">AI commit draft</h2></div>
        <span className={`network-state ${settings.providerKind === "ollama" ? "network-state--off" : ""}`}>
          {settings.providerKind === "ollama" ? "Local provider" : "External provider"}
        </span>
      </header>
      <p className="ai-intro"><strong>{settings.model || "No model configured"}</strong> · Configure providers and exclusions in Settings. Preview exactly what Core would disclose before generating.</p>
      <button className="ai-primary" type="button" disabled={!canPreview} onClick={() => void runPreview()}>
        {operation === "previewing" ? "Previewing…" : "Preview input"}
      </button>

      {error && <div className="ai-error" role="alert"><p>{error.message}. No commit was created.</p>{error.code === "ai.preview_stale" && <button type="button" onClick={() => void runPreview()}>Preview current index</button>}</div>}

      {preview && <section className="ai-disclosure" aria-labelledby="ai-disclosure-title">
        <header><div><p className="eyebrow">Disclosure preview</p><h3 id="ai-disclosure-title">{preview.destination === "external" ? "Leaves this environment" : "Stays in this environment"}</h3></div><strong>{formatBytes(preview.promptBytes)} prompt</strong></header>
        <dl>
          <div><dt>Endpoint</dt><dd>{preview.endpoint}</dd></div>
          <div><dt>Model</dt><dd>{preview.model}</dd></div>
          <div><dt>Staged patch</dt><dd>{formatBytes(preview.stagedDiffBytes)}</dd></div>
          <div><dt>Index binding</dt><dd><code>{preview.indexFingerprint.slice(0, 12)}</code></dd></div>
        </dl>
        {preview.truncated && <p className="ai-warning">At least one patch was truncated at the disclosed limit.</p>}
        <ul className="ai-files" aria-label="AI disclosure files">{preview.files.map((file) => <li key={file.path}>
          <span><strong>{file.path}</strong><small>+{file.additions} −{file.deletions}</small></span>
          <span><b className={`ai-file-state ai-file-state--${file.state}`}>{file.state}</b><small>{formatBytes(file.patchBytes)}{file.reason ? ` · ${file.reason}` : ""}</small></span>
        </li>)}</ul>
        {preview.externalConfirmationRequired && <label className="ai-confirm"><input type="checkbox" checked={externalConfirmed} onChange={(event) => setExternalConfirmed(event.target.checked)} /> I confirm the listed staged patch will be sent directly to {preview.endpoint} for this generation.</label>}
        <button className="ai-primary" type="button" disabled={!canGenerate} onClick={() => void runGenerate()}>{operation === "generating" ? "Generating…" : "Generate draft"}</button>
      </section>}

      {draft && <section className="ai-result" aria-labelledby="ai-result-title">
        <div><p className="eyebrow">Editable result</p><h3 id="ai-result-title">Review the draft</h3></div>
        <label>Commit message
          <textarea value={draftMessage} onChange={(event) => { setDraftMessage(event.target.value); setHandoff(false); }} />
        </label>
        {draft.suggestions.length > 0 && <div><h4>Suggestions — never run automatically</h4><ul>{draft.suggestions.map((suggestion, index) => <li key={`${suggestion.kind}:${index}`}><strong>{suggestion.title}</strong><p>{suggestion.detail}</p>{suggestion.affectedPaths.length > 0 && <small>{suggestion.affectedPaths.join(", ")}</small>}</li>)}</ul></div>}
        {draft.warnings.length > 0 && <div><h4>Warnings</h4><ul>{draft.warnings.map((warning, index) => <li key={index}>{warning}</li>)}</ul></div>}
        <button className="ai-primary" type="button" disabled={!draftMessage.trim()} onClick={() => { onUseCommitMessage(draftMessage); setHandoff(true); }}>Use in commit</button>
        {handoff && <p className="ai-success" role="status">Draft copied to Commit & branches. Review it there, then confirm the Git action separately.</p>}
      </section>}
    </section>
  );
}

function formatBytes(value: number) {
  return value < 1024 ? `${value} B` : `${(value / 1024).toFixed(1)} KiB`;
}
