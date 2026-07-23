import markUrl from "../../../assets/icons/gitnova-mark.svg";

const foundationItems = [
  { label: "Desktop shell", detail: "Ready", state: "ready" },
  { label: "Core connection", detail: "Next task", state: "pending" },
  { label: "Repository", detail: "Not opened", state: "idle" },
] as const;

export function App() {
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
          <div className="next-step" role="status">
            <span className="next-step__icon" aria-hidden="true">01</span>
            <span>
              <strong>Next:</strong> connect this Host to the independent GitNova Core process.
            </span>
          </div>
        </section>

        <aside className="foundation-card" aria-labelledby="foundation-title">
          <div>
            <p className="eyebrow">System status</p>
            <h2 id="foundation-title">Foundation</h2>
          </div>
          <ul>
            {foundationItems.map((item) => (
              <li key={item.label}>
                <span className={`status-mark status-mark--${item.state}`} aria-hidden="true" />
                <span>{item.label}</span>
                <strong>{item.detail}</strong>
              </li>
            ))}
          </ul>
          <p className="privacy-note">
            No repository content or credentials leave the repository environment.
          </p>
        </aside>
      </main>
    </div>
  );
}
