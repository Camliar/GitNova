import type { AiProviderKind } from "@gitnova/protocol";

export type AiAssistSettings = {
  providerKind: AiProviderKind;
  model: string;
  baseUrl: string;
  excludedText: string;
};

export const defaultAiAssistSettings: AiAssistSettings = {
  providerKind: "ollama",
  model: "",
  baseUrl: "http://127.0.0.1:11434",
  excludedText: "",
};

export function AiSettingsPanel({ settings, onChange }: { settings: AiAssistSettings; onChange: (settings: AiAssistSettings) => void }) {
  const update = <Key extends keyof AiAssistSettings>(key: Key, value: AiAssistSettings[Key]) => onChange({ ...settings, [key]: value });
  return <section className="settings-panel" aria-labelledby="ai-settings-title">
    <header><p className="eyebrow">Commit assistance</p><h2 id="ai-settings-title">AI provider</h2></header>
    <p className="settings-intro">These settings control the explicit AI action shown in the commit composer. GitNova always previews staged disclosure before generation, and AI never commits automatically.</p>
    <div className="settings-form">
      <label>Provider<select value={settings.providerKind} onChange={(event) => update("providerKind", event.target.value as AiProviderKind)}><option value="ollama">Ollama (local)</option><option value="openAi">OpenAI</option></select></label>
      <label>Model<input value={settings.model} onChange={(event) => update("model", event.target.value)} placeholder={settings.providerKind === "ollama" ? "qwen3, deepseek-r1, llama…" : "gpt-5.2, gpt-4.1…"} /></label>
      {settings.providerKind === "ollama" && <label>Ollama loopback URL<input value={settings.baseUrl} onChange={(event) => update("baseUrl", event.target.value)} /></label>}
      <label className="settings-form__wide">Excluded repository paths <span>one exact path per line</span><textarea value={settings.excludedText} onChange={(event) => update("excludedText", event.target.value)} placeholder={"config/private.json\nsecrets/test.key"} /></label>
    </div>
    <div className="settings-security"><strong>Credential boundary</strong><p>{settings.providerKind === "openAi" ? <><code>OPENAI_API_KEY</code> is read only by Core in the repository environment.</> : "Ollama must remain on a loopback address."} Desktop does not receive or store API keys.</p></div>
  </section>;
}
