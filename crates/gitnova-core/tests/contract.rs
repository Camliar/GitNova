use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn frame(value: &Value) -> Vec<u8> {
    let body = serde_json::to_vec(value).unwrap();
    let mut framed = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    framed.extend(body);
    framed
}

fn run(messages: &[Value]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_gitnova-core"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let stdin = child.stdin.as_mut().unwrap();
        for message in messages {
            stdin.write_all(&frame(message)).unwrap();
        }
    }
    drop(child.stdin.take());
    child.wait_with_output().unwrap()
}

fn responses(bytes: &[u8]) -> Vec<Value> {
    let mut remaining = bytes;
    let mut values = Vec::new();
    while !remaining.is_empty() {
        let header_end = remaining
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .expect("stdout contains only framed messages");
        let header = std::str::from_utf8(&remaining[..header_end]).unwrap();
        let length: usize = header
            .strip_prefix("Content-Length: ")
            .expect("canonical Content-Length header")
            .parse()
            .unwrap();
        let body_start = header_end + 4;
        let body_end = body_start + length;
        values.push(serde_json::from_slice(&remaining[body_start..body_end]).unwrap());
        remaining = &remaining[body_end..];
    }
    values
}

fn initialize(id: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "gitnova/initialize",
        "params": {
            "clientInfo": {"name": "contract-test", "version": "1.0.0"},
            "protocolVersion": "1.0",
            "capabilities": {"cancellation": true}
        }
    })
}

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new(label: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("gitnova-{label}-{}-{unique}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn git(arguments: &[&str], directory: &Path) {
    let output = git_output(arguments, directory);
    assert!(
        output.status.success(),
        "git command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_output(arguments: &[&str], directory: &Path) -> Output {
    Command::new("git")
        .args(arguments)
        .current_dir(directory)
        .env("LC_ALL", "C")
        .output()
        .unwrap()
}

fn repository_request(id: i64, method: &str, path: &Path) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": {"path": path.to_str().unwrap()}
    })
}

#[test]
fn completes_lifecycle_and_keeps_stdout_protocol_clean() {
    let output = run(&[
        initialize(json!("init-1")),
        json!({"jsonrpc":"2.0","id":2,"method":"gitnova/shutdown"}),
        json!({"jsonrpc":"2.0","method":"exit"}),
    ]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let responses = responses(&output.stdout);
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0]["id"], "init-1");
    assert_eq!(responses[0]["result"]["protocolVersion"], "1.3");
    assert_eq!(responses[0]["result"]["capabilities"]["cancellation"], true);
    assert_eq!(
        responses[0]["result"]["capabilities"]["workingTreeStatus"],
        true
    );
    assert_eq!(
        responses[0]["result"]["capabilities"]["structuredFileDiff"],
        true
    );
    assert_eq!(responses[1]["result"], Value::Null);
}

#[test]
fn rejects_requests_before_initialization() {
    let output = run(&[json!({
        "jsonrpc":"2.0","id":9,"method":"gitnova/unknown","params":{}
    })]);
    assert!(output.status.success());
    let responses = responses(&output.stdout);
    assert_eq!(responses[0]["error"]["code"], -32002);
    assert_eq!(
        responses[0]["error"]["data"]["stableCode"],
        "core.not_initialized"
    );
}

#[test]
fn cancellation_notification_is_applied_to_matching_request_id() {
    let output = run(&[
        initialize(json!(1)),
        json!({"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":"work-1"}}),
        json!({"jsonrpc":"2.0","id":"work-1","method":"gitnova/unknown","params":{}}),
    ]);
    let responses = responses(&output.stdout);
    assert_eq!(responses[1]["error"]["code"], -32800);
    assert_eq!(
        responses[1]["error"]["data"]["stableCode"],
        "request.cancelled"
    );
}

#[test]
fn exit_without_shutdown_is_unsuccessful() {
    let output = run(&[json!({"jsonrpc":"2.0","method":"exit"})]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn discovers_normal_repository_from_nested_file_and_opens_idempotently() {
    let directory = TestDirectory::new("normal");
    git(&["init", "repo"], &directory.0);
    let repository = directory.0.join("repo");
    let nested = repository.join("nested");
    fs::create_dir(&nested).unwrap();
    let file = nested.join("file.txt");
    fs::write(&file, "content").unwrap();

    let output = run(&[
        initialize(json!(1)),
        repository_request(2, "repository/discover", &file),
        repository_request(3, "repository/open", &nested),
        repository_request(4, "repository/open", &repository),
    ]);
    assert!(output.status.success());
    let responses = responses(&output.stdout);
    assert_eq!(responses[1]["result"]["kind"], "worktree");
    assert_eq!(
        responses[1]["result"]["worktreeRoot"],
        repository.canonicalize().unwrap().to_str().unwrap()
    );
    assert_eq!(responses[2]["result"], responses[3]["result"]);
}

#[test]
fn distinguishes_linked_worktree_and_bare_repository() {
    let directory = TestDirectory::new("kinds");
    git(&["init", "main"], &directory.0);
    let main = directory.0.join("main");
    fs::write(main.join("README.md"), "test").unwrap();
    git(&["add", "README.md"], &main);
    git(
        &[
            "-c",
            "user.name=GitNova Test",
            "-c",
            "user.email=test@gitnova.invalid",
            "commit",
            "-m",
            "initial",
        ],
        &main,
    );
    git(&["worktree", "add", "../linked", "-b", "linked"], &main);
    git(&["init", "--bare", "bare.git"], &directory.0);

    let output = run(&[
        initialize(json!(1)),
        repository_request(2, "repository/discover", &directory.0.join("linked")),
        repository_request(3, "repository/discover", &directory.0.join("bare.git")),
    ]);
    let responses = responses(&output.stdout);
    assert_eq!(responses[1]["result"]["kind"], "linkedWorktree");
    assert_ne!(
        responses[1]["result"]["gitDirectory"],
        responses[1]["result"]["commonGitDirectory"]
    );
    assert_eq!(responses[2]["result"]["kind"], "bare");
    assert_eq!(responses[2]["result"]["worktreeRoot"], Value::Null);
}

#[test]
fn rejects_opening_a_different_repository_in_one_session() {
    let directory = TestDirectory::new("different");
    git(&["init", "first"], &directory.0);
    git(&["init", "second"], &directory.0);
    let output = run(&[
        initialize(json!(1)),
        repository_request(2, "repository/open", &directory.0.join("first")),
        repository_request(3, "repository/open", &directory.0.join("second")),
    ]);
    let responses = responses(&output.stdout);
    assert_eq!(
        responses[2]["error"]["data"]["stableCode"],
        "repository.different_repository_open"
    );
}

#[test]
fn reports_invalid_path_and_non_repository_separately() {
    let directory = TestDirectory::new("errors");
    let missing = directory.0.join("missing");
    let output = run(&[
        initialize(json!(1)),
        repository_request(2, "repository/discover", &missing),
        repository_request(3, "repository/discover", &directory.0),
    ]);
    let responses = responses(&output.stdout);
    assert_eq!(responses[1]["error"]["data"]["stableCode"], "path.invalid");
    assert_eq!(
        responses[2]["error"]["data"]["stableCode"],
        "repository.not_found"
    );
}

#[test]
fn reports_staged_unstaged_untracked_and_renamed_entries() {
    let directory = TestDirectory::new("status-entries");
    git(&["init", "repo"], &directory.0);
    let repository = directory.0.join("repo");
    fs::write(repository.join("modified.txt"), "base").unwrap();
    fs::write(repository.join("original.txt"), "base").unwrap();
    git(&["add", "."], &repository);
    git(
        &[
            "-c",
            "user.name=GitNova Test",
            "-c",
            "user.email=test@gitnova.invalid",
            "commit",
            "-m",
            "initial",
        ],
        &repository,
    );
    fs::write(repository.join("modified.txt"), "changed").unwrap();
    fs::write(repository.join("staged.txt"), "staged").unwrap();
    git(&["add", "staged.txt"], &repository);
    git(&["mv", "original.txt", "renamed.txt"], &repository);
    fs::write(repository.join("untracked.txt"), "untracked").unwrap();

    let output = run(&[
        initialize(json!(1)),
        repository_request(2, "repository/open", &repository),
        json!({"jsonrpc":"2.0","id":3,"method":"repository/status"}),
    ]);
    let responses = responses(&output.stdout);
    let entries = responses[2]["result"]["entries"].as_array().unwrap();
    let by_path = |path: &str| entries.iter().find(|entry| entry["path"] == path).unwrap();
    assert_eq!(by_path("modified.txt")["worktreeStatus"], "modified");
    assert_eq!(by_path("staged.txt")["indexStatus"], "added");
    assert_eq!(by_path("renamed.txt")["indexStatus"], "renamed");
    assert_eq!(by_path("renamed.txt")["originalPath"], "original.txt");
    assert_eq!(by_path("untracked.txt")["worktreeStatus"], "untracked");
}

#[test]
fn reports_conflict() {
    let directory = TestDirectory::new("status-conflict");
    git(&["init", "repo"], &directory.0);
    let repository = directory.0.join("repo");
    git(&["config", "user.name", "GitNova Test"], &repository);
    git(
        &["config", "user.email", "test@gitnova.invalid"],
        &repository,
    );
    fs::write(repository.join("conflict.txt"), "base\n").unwrap();
    git(&["add", "."], &repository);
    git(&["commit", "-m", "initial"], &repository);
    git(&["branch", "-M", "main"], &repository);
    git(&["checkout", "-b", "other"], &repository);
    fs::write(repository.join("conflict.txt"), "other\n").unwrap();
    git(
        &[
            "commit",
            "-am",
            "other",
            "--author=GitNova Test <test@gitnova.invalid>",
        ],
        &repository,
    );
    git(&["checkout", "main"], &repository);
    fs::write(repository.join("conflict.txt"), "master\n").unwrap();
    git(
        &[
            "commit",
            "-am",
            "master",
            "--author=GitNova Test <test@gitnova.invalid>",
        ],
        &repository,
    );
    let merge = git_output(&["merge", "other"], &repository);
    assert!(!merge.status.success());

    let output = run(&[
        initialize(json!(1)),
        repository_request(2, "repository/open", &repository),
        json!({"jsonrpc":"2.0","id":3,"method":"repository/status","params":{}}),
    ]);
    let responses = responses(&output.stdout);
    assert_eq!(responses[2]["result"]["entries"][0]["kind"], "unmerged");
}

#[test]
fn reports_detached_head() {
    let directory = TestDirectory::new("status-detached");
    git(&["init", "repo"], &directory.0);
    let repository = directory.0.join("repo");
    fs::write(repository.join("file.txt"), "base").unwrap();
    git(&["add", "."], &repository);
    git(
        &[
            "-c",
            "user.name=GitNova Test",
            "-c",
            "user.email=test@gitnova.invalid",
            "commit",
            "-m",
            "initial",
        ],
        &repository,
    );
    git(&["checkout", "--detach"], &repository);
    let output = run(&[
        initialize(json!(1)),
        repository_request(2, "repository/open", &repository),
        json!({"jsonrpc":"2.0","id":3,"method":"repository/status"}),
    ]);
    let responses = responses(&output.stdout);
    assert_eq!(responses[2]["result"]["branch"]["head"], Value::Null);
    assert!(responses[2]["result"]["branch"]["oid"].is_string());
}

#[test]
fn reports_upstream_ahead_and_behind() {
    let directory = TestDirectory::new("status-upstream");
    git(&["init", "--bare", "remote.git"], &directory.0);
    let remote = directory.0.join("remote.git");
    git(&["clone", remote.to_str().unwrap(), "work"], &directory.0);
    let work = directory.0.join("work");
    git(&["config", "user.name", "GitNova Test"], &work);
    git(&["config", "user.email", "test@gitnova.invalid"], &work);
    fs::write(work.join("base.txt"), "base").unwrap();
    git(&["add", "."], &work);
    git(&["commit", "-m", "initial"], &work);
    git(&["branch", "-M", "main"], &work);
    git(&["push", "-u", "origin", "main"], &work);
    git(&["symbolic-ref", "HEAD", "refs/heads/main"], &remote);
    git(&["clone", remote.to_str().unwrap(), "other"], &directory.0);
    let other = directory.0.join("other");
    git(&["config", "user.name", "GitNova Test"], &other);
    git(&["config", "user.email", "test@gitnova.invalid"], &other);
    fs::write(other.join("remote.txt"), "remote").unwrap();
    git(&["add", "."], &other);
    git(&["commit", "-m", "remote"], &other);
    git(&["push"], &other);
    fs::write(work.join("local.txt"), "local").unwrap();
    git(&["add", "."], &work);
    git(&["commit", "-m", "local"], &work);
    git(&["fetch", "origin"], &work);

    let output = run(&[
        initialize(json!(1)),
        repository_request(2, "repository/open", &work),
        json!({"jsonrpc":"2.0","id":3,"method":"repository/status"}),
    ]);
    let responses = responses(&output.stdout);
    let branch = &responses[2]["result"]["branch"];
    assert_eq!(branch["head"], "main");
    assert_eq!(branch["upstream"], "origin/main");
    assert_eq!(branch["ahead"], 1);
    assert_eq!(branch["behind"], 1);
}

#[test]
fn status_requires_open_non_bare_repository() {
    let directory = TestDirectory::new("status-errors");
    git(&["init", "--bare", "bare.git"], &directory.0);
    let no_open = run(&[
        initialize(json!(1)),
        json!({"jsonrpc":"2.0","id":2,"method":"repository/status"}),
    ]);
    let no_open_responses = responses(&no_open.stdout);
    assert_eq!(
        no_open_responses[1]["error"]["data"]["stableCode"],
        "repository.not_open"
    );

    let bare = run(&[
        initialize(json!(1)),
        repository_request(2, "repository/open", &directory.0.join("bare.git")),
        json!({"jsonrpc":"2.0","id":3,"method":"repository/status"}),
    ]);
    let bare_responses = responses(&bare.stdout);
    assert_eq!(
        bare_responses[2]["error"]["data"]["stableCode"],
        "repository.worktree_required"
    );
}

#[test]
fn returns_distinct_staged_and_working_tree_hunks() {
    let directory = TestDirectory::new("diff-scopes");
    git(&["init", "repo"], &directory.0);
    let repository = directory.0.join("repo");
    fs::write(repository.join("file.txt"), "one\nold\nthree\n").unwrap();
    git(&["add", "."], &repository);
    git(
        &[
            "-c",
            "user.name=GitNova Test",
            "-c",
            "user.email=test@gitnova.invalid",
            "commit",
            "-m",
            "initial",
        ],
        &repository,
    );
    fs::write(repository.join("file.txt"), "one\nstaged\nthree\n").unwrap();
    git(&["add", "file.txt"], &repository);
    fs::write(repository.join("file.txt"), "one\nworking\nthree").unwrap();

    let output = run(&[
        initialize(json!(1)),
        repository_request(2, "repository/open", &repository),
        json!({"jsonrpc":"2.0","id":3,"method":"repository/diff","params":{"path":"file.txt","scope":"staged","contextLines":0}}),
        json!({"jsonrpc":"2.0","id":4,"method":"repository/diff","params":{"path":"file.txt","scope":"workingTree","contextLines":0}}),
    ]);
    let responses = responses(&output.stdout);
    let staged_lines = responses[2]["result"]["hunks"][0]["lines"]
        .as_array()
        .unwrap();
    assert!(
        staged_lines
            .iter()
            .any(|line| line["content"] == "old" && line["kind"] == "deletion")
    );
    assert!(
        staged_lines
            .iter()
            .any(|line| line["content"] == "staged" && line["kind"] == "addition")
    );
    let working_lines = responses[3]["result"]["hunks"][0]["lines"]
        .as_array()
        .unwrap();
    assert!(
        working_lines
            .iter()
            .any(|line| line["content"] == "staged" && line["kind"] == "deletion")
    );
    assert!(
        working_lines
            .iter()
            .any(|line| line["content"] == "working" && line["kind"] == "addition")
    );
}

#[test]
fn reports_binary_empty_and_invalid_diff_requests() {
    let directory = TestDirectory::new("diff-boundaries");
    git(&["init", "repo"], &directory.0);
    let repository = directory.0.join("repo");
    fs::write(repository.join("image.bin"), [0_u8, 1, 2, 3]).unwrap();
    fs::write(repository.join("clean.txt"), "clean\n").unwrap();
    git(&["add", "."], &repository);
    git(
        &[
            "-c",
            "user.name=GitNova Test",
            "-c",
            "user.email=test@gitnova.invalid",
            "commit",
            "-m",
            "initial",
        ],
        &repository,
    );
    fs::write(repository.join("image.bin"), [0_u8, 9, 8, 7]).unwrap();
    let output = run(&[
        initialize(json!(1)),
        repository_request(2, "repository/open", &repository),
        json!({"jsonrpc":"2.0","id":3,"method":"repository/diff","params":{"path":"image.bin","scope":"workingTree"}}),
        json!({"jsonrpc":"2.0","id":4,"method":"repository/diff","params":{"path":"clean.txt","scope":"workingTree"}}),
        json!({"jsonrpc":"2.0","id":5,"method":"repository/diff","params":{"path":"../outside","scope":"workingTree"}}),
        json!({"jsonrpc":"2.0","id":6,"method":"repository/diff","params":{"path":"clean.txt","scope":"workingTree","contextLines":21}}),
    ]);
    let responses = responses(&output.stdout);
    assert_eq!(responses[2]["result"]["isBinary"], true);
    assert_eq!(responses[2]["result"]["hunks"], json!([]));
    assert_eq!(responses[3]["result"]["hunks"], json!([]));
    assert_eq!(
        responses[4]["error"]["data"]["stableCode"],
        "path.invalid_repository_relative"
    );
    assert_eq!(
        responses[5]["error"]["data"]["stableCode"],
        "protocol.invalid_params"
    );
}

#[test]
fn diff_requires_open_non_bare_repository() {
    let directory = TestDirectory::new("diff-errors");
    git(&["init", "--bare", "bare.git"], &directory.0);
    let no_open = run(&[
        initialize(json!(1)),
        json!({"jsonrpc":"2.0","id":2,"method":"repository/diff","params":{"path":"file.txt","scope":"workingTree"}}),
    ]);
    assert_eq!(
        responses(&no_open.stdout)[1]["error"]["data"]["stableCode"],
        "repository.not_open"
    );
    let bare = run(&[
        initialize(json!(1)),
        repository_request(2, "repository/open", &directory.0.join("bare.git")),
        json!({"jsonrpc":"2.0","id":3,"method":"repository/diff","params":{"path":"file.txt","scope":"workingTree"}}),
    ]);
    assert_eq!(
        responses(&bare.stdout)[2]["error"]["data"]["stableCode"],
        "repository.worktree_required"
    );
}
