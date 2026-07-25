function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function renderTrace(trace, diff) {
  const pr = trace.pullRequest;
  const relationship = trace.relationship;
  const commits = pr.commits.map((commit) => `<li><code>${escapeHtml(commit.oid.slice(0, 8))}</code> ${escapeHtml(commit.summary)}</li>`).join("");
  const files = diff ? diff.files.map((file) => {
    const hunks = file.hunks.map((hunk) => {
      const lines = hunk.lines.map((line) => `${line.kind === "addition" ? "+" : line.kind === "deletion" ? "-" : " "}${line.content}`).join("\n");
      return `<pre>${escapeHtml(hunk.header)}\n${escapeHtml(lines)}</pre>`;
    }).join("");
    return `<section><h3>${escapeHtml(file.newPath)}</h3><p>+${file.additions} −${file.deletions} · ${escapeHtml(file.patchState)}</p>${hunks || "<p>Patch unavailable.</p>"}</section>`;
  }).join("") : "<p>Select an original commit to inspect its remote diff.</p>";
  return `<!doctype html><html><head><meta charset="utf-8"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'"><style>body{font:13px var(--vscode-font-family);padding:20px;color:var(--vscode-foreground)}code,pre{font-family:var(--vscode-editor-font-family)}pre{padding:12px;overflow:auto;background:var(--vscode-textCodeBlock-background)}li{margin:6px 0}.meta{color:var(--vscode-descriptionForeground)}</style></head><body><h1>PR #${pr.number}: ${escapeHtml(pr.title)}</h1><p class="meta">${escapeHtml(pr.nameWithOwner)} · ${escapeHtml(pr.state)}</p><h2>Squash relationship</h2><p>${escapeHtml(relationship.classification)} · confidence ${escapeHtml(relationship.confidence)}</p><p>Final commit: <code>${escapeHtml(relationship.mergeCommitOid || "unavailable")}</code></p><h2>Original commits</h2><ol>${commits}</ol><h2>Selected commit diff</h2>${files}</body></html>`;
}

module.exports = { escapeHtml, renderTrace };
