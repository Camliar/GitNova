import { useEffect, useState } from "react";
import markUrl from "../../../assets/icons/gitnova-mark.svg";
import { asDesktopError, getCoreStatus, startCore, type DesktopError } from "./core";

const foundationItems = [
  { label: "Desktop shell", detail: "Ready", state: "ready" },
  { label: "Repository", detail: "Not opened", state: "idle" },
] as const;

export function App() {
  const [connection, setConnection] = useState<
    | { kind: "checking" }
    | { kind: "stopped" }
    | { kind: "connected"; version: string }
    | { kind: "error"; error: DesktopError }
  >({ kind: "checking" });

  useEffect(() => {
    let active = true;
    void getCoreStatus()
      .then((status) => {
        if (!active) return;
        setConnection(
          status.connected
            ? { kind: "connected", version: status.protocolVersion ?? "unknown" }
            : { kind: "stopped" },
        );
      })
      .catch((error: unknown) => {
        if (active) setConnection({ kind: "error", error: asDesktopError(error) });
      });
    return () => {
      active = false;
    };
  }, []);

  async function connectCore() {
    setConnection({ kind: "checking" });
    try {
      const status = await startCore();
      setConnection({ kind: "connected", version: status.protocolVersion ?? "unknown" });
    } catch (error) {
      setConnection({ kind: "error", error: asDesktopError(error) });
    }
  }

  const coreDetail =
    connection.kind === "connected"
      ? `Connected · v${connection.version}`
      : connection.kind === "checking"
        ? "Checking…"
        : connection.kind === "error"
          ? "Unavailable"
          : "Not running";

  return (
    <div className="app-shell">
      <header className="app-header">
        <a className="brand" href="#main-content" aria-label="GitNova home">
          <img src={markUrl} alt="" width="36" height="36" />
          <span>GitNova</span>
        </a>
        <span className="local-badge">
          <span aria-hidden="true" className="local-badge__dot" />
          Local-first desktop
        </span>
      </header>

      <main id="main-content" className="workspace" tabIndex={-1}>
        <section className="hero" aria-labelledby="welcome-title">
          <p className="eyebrow">Desktop foundation</p>
          <h1 id="welcome-title">Understand the history behind the merge.</h1>
          <p className="hero__copy">
            GitNova will connect pull requests, their original commits, and the final merge without
            moving repository data to a central service.
          </p>
          <div className="next-step" role="status" aria-live="polite">
            <span className="next-step__icon" aria-hidden="true">01</span>
            <span>
              <strong>Core:</strong>{" "}
              {connection.kind === "connected"
                ? "the independent local process is ready."
                : "start the independent local process to continue."}
            </span>
          </div>
        </section>

        <aside className="foundation-card" aria-labelledby="foundation-title">
          <div>
            <p className="eyebrow">System status</p>
            <h2 id="foundation-title">Foundation</h2>
          </div>
          <ul>
            <li>
              <span
                className={`status-mark status-mark--${connection.kind === "connected" ? "ready" : connection.kind === "checking" ? "pending" : "idle"}`}
                aria-hidden="true"
              />
              <span>Core connection</span>
              <strong>{coreDetail}</strong>
            </li>
            {foundationItems.map((item) => (
              <li key={item.label}>
                <span className={`status-mark status-mark--${item.state}`} aria-hidden="true" />
                <span>{item.label}</span>
                <strong>{item.detail}</strong>
              </li>
            ))}
          </ul>
          {(connection.kind === "stopped" || connection.kind === "error") && (
            <div className="connection-action">
              {connection.kind === "error" && (
                <p role="alert">{connection.error.message}. No repository data was changed.</p>
              )}
              <button type="button" onClick={() => void connectCore()}>
                {connection.kind === "error" ? "Retry Core" : "Start Core"}
              </button>
            </div>
          )}
          <p className="privacy-note">
            No repository content or credentials leave the repository environment.
          </p>
        </aside>
      </main>
    </div>
  );
}
