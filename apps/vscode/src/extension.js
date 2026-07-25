const vscode = require("vscode");
const { existsSync } = require("node:fs");
const { isAbsolute, join } = require("node:path");
const { CoreClient } = require("./coreClient");
const { renderTrace } = require("./render");

let client;
let repository;
let status;

function resolveCore(context) {
  const configured = vscode.workspace.getConfiguration("gitnova").get("core.path", "").trim();
  if (configured) {
    if (!isAbsolute(configured)) throw new Error("gitnova.core.path must be absolute");
    return configured;
  }
  const bundled = join(context.extensionPath, "bin", process.platform === "win32" ? "gitnova-core.exe" : "gitnova-core");
  return existsSync(bundled) ? bundled : process.platform === "win32" ? "gitnova-core.exe" : "gitnova-core";
}

async function ensureCore(context) {
  if (client?.running) return client;
  await client?.dispose();
  client = undefined;
  status.text = "$(sync~spin) GitNova connecting";
  const candidate = new CoreClient(resolveCore(context));
  try {
    await candidate.start();
    client = candidate;
    status.text = "$(check) GitNova Core";
    return client;
  } catch (error) {
    await candidate.dispose();
    status.text = "$(error) GitNova Core";
    throw error;
  }
}

async function openWorkspace(context) {
  const folder = vscode.workspace.workspaceFolders?.[0];
  if (!folder) throw new Error("Open a VS Code workspace folder first");
  const core = await ensureCore(context);
  repository = await core.request("repository/open", { path: folder.uri.fsPath });
  status.text = `$(repo) GitNova: ${repository.kind}`;
  return repository;
}

async function inspectPullRequest(context) {
  const core = await ensureCore(context);
  if (!repository) await openWorkspace(context);
  const input = await vscode.window.showInputBox({
    title: "GitNova Squash Trace",
    prompt: "PR data and the selected commit patch will be requested by Core through your configured GitHub Provider.",
    placeHolder: "Pull request number",
    validateInput: (value) => /^[1-9]\d*$/.test(value) ? undefined : "Enter a positive pull request number",
  });
  if (!input) return;
  const number = Number(input);
  const providerRepository = await core.request("github/repository", {});
  const trace = await core.request("github/squashTrace", { number, nameWithOwner: providerRepository.nameWithOwner });
  const selected = await vscode.window.showQuickPick(
    trace.pullRequest.commits.map((commit) => ({ label: commit.summary, description: commit.oid.slice(0, 12), commit })),
    { title: `PR #${number} original commits`, placeHolder: "Select a commit to request its remote patch, or press Escape for relationship only" },
  );
  const diff = selected ? await core.request("github/pullRequestCommitDiff", { number, oid: selected.commit.oid, nameWithOwner: providerRepository.nameWithOwner }) : undefined;
  const panel = vscode.window.createWebviewPanel("gitnova.squashTrace", `GitNova PR #${number}`, vscode.ViewColumn.Active, { enableScripts: false });
  panel.webview.html = renderTrace(trace, diff);
}

async function report(action) {
  try { await action(); }
  catch (error) { void vscode.window.showErrorMessage(error instanceof Error ? error.message : "GitNova operation failed"); }
}

function activate(context) {
  status = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 50);
  status.name = "GitNova Core";
  status.command = "gitnova.connect";
  status.text = "$(circle-outline) GitNova Core";
  status.show();
  context.subscriptions.push(
    status,
    vscode.commands.registerCommand("gitnova.connect", () => report(() => ensureCore(context))),
    vscode.commands.registerCommand("gitnova.openRepository", () => report(() => openWorkspace(context))),
    vscode.commands.registerCommand("gitnova.inspectPullRequest", () => report(() => inspectPullRequest(context))),
    { dispose: () => { void client?.dispose(); client = undefined; } },
  );
}

async function deactivate() {
  await client?.dispose();
  client = undefined;
}

module.exports = { activate, deactivate };
