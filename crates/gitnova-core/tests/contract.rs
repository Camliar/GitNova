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
    assert_eq!(responses[0]["result"]["protocolVersion"], "1.8");
    assert_eq!(responses[0]["result"]["capabilities"]["cancellation"], true);
    assert_eq!(
        responses[0]["result"]["capabilities"]["workingTreeStatus"],
        true
    );
    assert_eq!(
        responses[0]["result"]["capabilities"]["structuredFileDiff"],
        true
    );
    assert_eq!(
        responses[0]["result"]["capabilities"]["paginatedCommitHistory"],
        true
    );
    assert_eq!(
        responses[0]["result"]["capabilities"]["structuredCommitDiff"],
        true
    );
    assert_eq!(
        responses[0]["result"]["capabilities"]["repositoryReferences"],
        true
    );
    assert_eq!(
        responses[0]["result"]["capabilities"]["commitGraphProjection"],
        true
    );
    assert_eq!(
        responses[0]["result"]["capabilities"]["githubRepository"],
        true
    );
    assert_eq!(responses[1]["result"], Value::Null);
}

fn commit_file(repository: &Path, number: usize, message: &str) {
    fs::write(repository.join("history.txt"), format!("{number}\n")).unwrap();
    git(&["add", "history.txt"], repository);
    git(
        &[
            "-c",
            "user.name=GitNova Author",
            "-c",
            "user.email=author@gitnova.invalid",
            "commit",
            "-m",
            message,
        ],
        repository,
    );
}

fn history_response(repository: &Path, params: Value) -> Value {
    let output = run(&[
        initialize(json!(1)),
        repository_request(2, "repository/open", repository),
        json!({"jsonrpc":"2.0","id":3,"method":"repository/history","params":params}),
    ]);
    assert!(output.status.success());
    responses(&output.stdout).remove(2)
}

fn head_oid(repository: &Path) -> String {
    String::from_utf8(git_output(&["rev-parse", "HEAD"], repository).stdout)
        .unwrap()
        .trim()
        .to_owned()
}

fn commit_diff_response(repository: &Path, oid: &str, extra: Value) -> Value {
    let mut params = serde_json::Map::new();
    params.insert("oid".into(), json!(oid));
    if let Some(extra) = extra.as_object() {
        params.extend(extra.clone());
    }
    let output = run(&[
        initialize(json!(1)),
        repository_request(2, "repository/open", repository),
        json!({"jsonrpc":"2.0","id":3,"method":"repository/commitDiff","params":params}),
    ]);
    assert!(output.status.success());
    responses(&output.stdout).remove(2)
}

fn references_response(repository: &Path) -> Value {
    let output = run(&[
        initialize(json!(1)),
        repository_request(2, "repository/open", repository),
        json!({"jsonrpc":"2.0","id":3,"method":"repository/references"}),
    ]);
    assert!(output.status.success());
    responses(&output.stdout).remove(2)
}

fn graph_response(repository: &Path, params: Value) -> Value {
    let output = run(&[
        initialize(json!(1)),
        repository_request(2, "repository/open", repository),
        json!({"jsonrpc":"2.0","id":3,"method":"repository/graph","params":params}),
    ]);
    assert!(output.status.success());
    responses(&output.stdout).remove(2)
}

#[test]
fn projects_paginated_commits_with_head_branch_and_tag_decorations() {
    let directory = TestDirectory::new("graph-projection");
    git(&["init", "repo"], &directory.0);
    let repository = directory.0.join("repo");
    for number in 1..=3 {
        commit_file(&repository, number, &format!("commit {number}"));
    }
    let head = head_oid(&repository);
    git(&["branch", "previous", "HEAD~1"], &repository);
    git(&["tag", "lightweight"], &repository);
    git(
        &[
            "-c",
            "user.name=GitNova Test",
            "-c",
            "user.email=test@gitnova.invalid",
            "tag",
            "-a",
            "annotated",
            "-m",
            "tag",
        ],
        &repository,
    );

    let first = graph_response(&repository, json!({"limit": 2}));
    let nodes = first["result"]["nodes"].as_array().unwrap();
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0]["commit"]["oid"], head);
    assert_eq!(nodes[0]["isHead"], true);
    let names: Vec<_> = nodes[0]["references"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"lightweight"));
    assert!(names.contains(&"annotated"));
    assert!(
        nodes[1]["references"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["name"] == "previous")
    );
    let cursor = first["result"]["nextCursor"].as_str().unwrap().to_owned();
    commit_file(&repository, 4, "after snapshot");
    let second = graph_response(&repository, json!({"limit": 2, "cursor": cursor}));
    assert_eq!(
        second["result"]["nodes"][0]["commit"]["summary"],
        "commit 1"
    );
    assert!(second["result"]["nextCursor"].is_null());
}

#[test]
fn graph_preserves_merge_topology_and_supports_detached_bare_and_empty() {
    let directory = TestDirectory::new("graph-kinds");
    git(&["init", "repo"], &directory.0);
    let repository = directory.0.join("repo");
    let empty = graph_response(&repository, Value::Null);
    assert!(empty["result"]["nodes"].as_array().unwrap().is_empty());
    commit_file(&repository, 1, "root");
    git(&["switch", "-c", "topic"], &repository);
    fs::write(repository.join("topic.txt"), "topic").unwrap();
    git(&["add", "topic.txt"], &repository);
    git(
        &[
            "-c",
            "user.name=GitNova Test",
            "-c",
            "user.email=test@gitnova.invalid",
            "commit",
            "-m",
            "topic",
        ],
        &repository,
    );
    git(&["checkout", "-"], &repository);
    commit_file(&repository, 2, "main");
    git(
        &[
            "-c",
            "user.name=GitNova Test",
            "-c",
            "user.email=test@gitnova.invalid",
            "merge",
            "--no-ff",
            "topic",
            "-m",
            "merge",
        ],
        &repository,
    );
    let merge_oid = head_oid(&repository);
    git(&["checkout", "--detach", "HEAD"], &repository);
    let detached = graph_response(&repository, json!({"limit": 1}));
    assert_eq!(detached["result"]["nodes"][0]["commit"]["oid"], merge_oid);
    assert_eq!(
        detached["result"]["nodes"][0]["commit"]["parents"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(detached["result"]["nodes"][0]["isHead"], true);
    git(
        &["clone", "--bare", repository.to_str().unwrap(), "bare.git"],
        &directory.0,
    );
    let bare = graph_response(&directory.0.join("bare.git"), json!({"limit": 1}));
    assert_eq!(bare["result"]["nodes"][0]["commit"]["oid"], merge_oid);
    let invalid = graph_response(&repository, json!({"cursor": "invalid"}));
    assert_eq!(
        invalid["error"]["data"]["stableCode"],
        "history.invalid_cursor"
    );
}

#[test]
fn returns_local_remote_tag_symbolic_and_upstream_references() {
    let directory = TestDirectory::new("references");
    git(&["init", "repo"], &directory.0);
    let repository = directory.0.join("repo");
    commit_file(&repository, 1, "root");
    let oid = head_oid(&repository);
    let branch =
        String::from_utf8(git_output(&["symbolic-ref", "--short", "HEAD"], &repository).stdout)
            .unwrap()
            .trim()
            .to_owned();
    git(&["branch", "topic"], &repository);
    git(&["tag", "lightweight"], &repository);
    git(
        &[
            "-c",
            "user.name=GitNova Test",
            "-c",
            "user.email=test@gitnova.invalid",
            "tag",
            "-a",
            "annotated",
            "-m",
            "annotated tag",
        ],
        &repository,
    );
    git(&["init", "--bare", "remote.git"], &directory.0);
    let remote = directory.0.join("remote.git");
    git(
        &["remote", "add", "origin", remote.to_str().unwrap()],
        &repository,
    );
    git(
        &["push", "origin", &format!("{branch}:{branch}")],
        &repository,
    );
    git(&["fetch", "origin"], &repository);
    git(
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            &format!("refs/remotes/origin/{branch}"),
        ],
        &repository,
    );
    git(
        &[
            "branch",
            "--set-upstream-to",
            &format!("origin/{branch}"),
            &branch,
        ],
        &repository,
    );

    let response = references_response(&repository);
    assert_eq!(response["result"]["head"]["oid"], oid);
    assert_eq!(
        response["result"]["head"]["symbolicRef"],
        format!("refs/heads/{branch}")
    );
    let refs = response["result"]["references"].as_array().unwrap();
    let local = refs
        .iter()
        .find(|item| item["fullName"] == format!("refs/heads/{branch}"))
        .unwrap();
    assert_eq!(local["upstream"], format!("refs/remotes/origin/{branch}"));
    let remote_head = refs
        .iter()
        .find(|item| item["fullName"] == "refs/remotes/origin/HEAD")
        .unwrap();
    assert_eq!(remote_head["kind"], "remoteBranch");
    assert_eq!(
        remote_head["symbolicTarget"],
        format!("refs/remotes/origin/{branch}")
    );
    let annotated = refs
        .iter()
        .find(|item| item["fullName"] == "refs/tags/annotated")
        .unwrap();
    assert_eq!(annotated["peeledTargetOid"], oid);
    let lightweight = refs
        .iter()
        .find(|item| item["fullName"] == "refs/tags/lightweight")
        .unwrap();
    assert!(lightweight["peeledTargetOid"].is_null());
}

#[test]
fn references_distinguish_unborn_detached_and_bare_head() {
    let directory = TestDirectory::new("reference-heads");
    git(&["init", "empty"], &directory.0);
    let empty = directory.0.join("empty");
    let unborn = references_response(&empty);
    assert!(unborn["result"]["head"]["oid"].is_null());
    assert!(
        unborn["result"]["head"]["symbolicRef"]
            .as_str()
            .unwrap()
            .starts_with("refs/heads/")
    );
    assert!(
        unborn["result"]["references"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    commit_file(&empty, 1, "root");
    let oid = head_oid(&empty);
    git(&["checkout", "--detach", "HEAD"], &empty);
    let detached = references_response(&empty);
    assert_eq!(detached["result"]["head"]["oid"], oid);
    assert!(detached["result"]["head"]["symbolicRef"].is_null());
    git(
        &["clone", "--bare", empty.to_str().unwrap(), "bare.git"],
        &directory.0,
    );
    let bare = references_response(&directory.0.join("bare.git"));
    assert_eq!(bare["result"]["head"]["oid"], oid);
}

#[test]
fn returns_structured_root_and_single_parent_commit_diffs() {
    let directory = TestDirectory::new("commit-diff-root");
    git(&["init", "repo"], &directory.0);
    let repository = directory.0.join("repo");
    fs::write(repository.join("alpha.txt"), "one\ntwo\n").unwrap();
    fs::write(repository.join("binary.bin"), [0, 1, 2, 0, 3]).unwrap();
    git(&["add", "."], &repository);
    git(
        &[
            "-c",
            "user.name=GitNova Test",
            "-c",
            "user.email=test@gitnova.invalid",
            "commit",
            "-m",
            "root",
        ],
        &repository,
    );
    let root_oid = head_oid(&repository);
    let root = commit_diff_response(&repository, &root_oid, json!({"contextLines": 0}));
    assert!(root["result"]["parentOid"].is_null());
    assert_eq!(root["result"]["files"].as_array().unwrap().len(), 2);
    assert!(
        root["result"]["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|file| file["isBinary"] == true)
    );
    assert!(
        root["result"]["files"][0]["hunks"][0]["lines"]
            .as_array()
            .unwrap()
            .iter()
            .all(|line| line["kind"] != "context")
    );

    git(&["mv", "alpha.txt", "renamed.txt"], &repository);
    git(&["add", "-A"], &repository);
    git(
        &[
            "-c",
            "user.name=GitNova Test",
            "-c",
            "user.email=test@gitnova.invalid",
            "commit",
            "-m",
            "rename",
        ],
        &repository,
    );
    let oid = head_oid(&repository);
    let changed = commit_diff_response(&repository, &oid, json!({"contextLines": 0}));
    assert_eq!(changed["result"]["parentOid"], root_oid);
    assert_eq!(changed["result"]["files"][0]["oldPath"], "alpha.txt");
    assert_eq!(changed["result"]["files"][0]["newPath"], "renamed.txt");
    assert!(
        changed["result"]["files"][0]["hunks"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn requires_and_validates_a_direct_merge_parent() {
    let directory = TestDirectory::new("commit-diff-merge");
    git(&["init", "repo"], &directory.0);
    let repository = directory.0.join("repo");
    commit_file(&repository, 1, "root");
    git(&["switch", "-c", "topic"], &repository);
    fs::write(repository.join("topic.txt"), "topic\n").unwrap();
    git(&["add", "topic.txt"], &repository);
    git(
        &[
            "-c",
            "user.name=GitNova Test",
            "-c",
            "user.email=test@gitnova.invalid",
            "commit",
            "-m",
            "topic",
        ],
        &repository,
    );
    let topic_oid = head_oid(&repository);
    git(&["checkout", "-"], &repository);
    commit_file(&repository, 2, "main");
    git(
        &[
            "-c",
            "user.name=GitNova Test",
            "-c",
            "user.email=test@gitnova.invalid",
            "merge",
            "--no-ff",
            "topic",
            "-m",
            "merge",
        ],
        &repository,
    );
    let merge_oid = head_oid(&repository);
    let required = commit_diff_response(&repository, &merge_oid, json!({}));
    assert_eq!(
        required["error"]["data"]["stableCode"],
        "commit.parent_required"
    );
    let selected = commit_diff_response(&repository, &merge_oid, json!({"parentOid": topic_oid}));
    assert_eq!(selected["result"]["files"][0]["newPath"], "history.txt");
    let invalid = commit_diff_response(
        &repository,
        &merge_oid,
        json!({"parentOid": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}),
    );
    assert_eq!(
        invalid["error"]["data"]["stableCode"],
        "commit.invalid_parent"
    );
}

#[test]
fn commit_diff_supports_bare_and_reports_invalid_objects() {
    let directory = TestDirectory::new("commit-diff-bare");
    git(&["init", "repo"], &directory.0);
    let repository = directory.0.join("repo");
    commit_file(&repository, 1, "root");
    let oid = head_oid(&repository);
    git(&["checkout", "--detach", "HEAD"], &repository);
    let detached = commit_diff_response(&repository, &oid, json!({}));
    assert_eq!(detached["result"]["commit"]["oid"], oid);
    git(
        &[
            "-c",
            "user.name=GitNova Test",
            "-c",
            "user.email=test@gitnova.invalid",
            "commit",
            "--allow-empty",
            "-m",
            "empty",
        ],
        &repository,
    );
    let empty_oid = head_oid(&repository);
    let empty = commit_diff_response(&repository, &empty_oid, json!({}));
    assert!(empty["result"]["files"].as_array().unwrap().is_empty());
    git(
        &["clone", "--bare", repository.to_str().unwrap(), "bare.git"],
        &directory.0,
    );
    let bare = commit_diff_response(
        &directory.0.join("bare.git"),
        &oid.to_uppercase(),
        json!({}),
    );
    assert_eq!(bare["result"]["commit"]["oid"], oid);
    let malformed = commit_diff_response(&repository, "HEAD", json!({}));
    assert_eq!(malformed["error"]["code"], -32602);
    let missing = commit_diff_response(
        &repository,
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        json!({}),
    );
    assert_eq!(missing["error"]["data"]["stableCode"], "commit.not_found");
}

#[test]
fn paginates_a_fixed_head_snapshot_without_duplicates() {
    let directory = TestDirectory::new("history-pages");
    git(&["init", "repo"], &directory.0);
    let repository = directory.0.join("repo");
    for number in 1..=5 {
        commit_file(&repository, number, &format!("commit {number}"));
    }

    let first = history_response(&repository, json!({"limit": 2}));
    assert_eq!(first["result"]["commits"].as_array().unwrap().len(), 2);
    let cursor = first["result"]["nextCursor"].as_str().unwrap().to_owned();
    commit_file(&repository, 6, "commit added after snapshot");
    let second = history_response(&repository, json!({"limit": 2, "cursor": cursor}));
    let second_summaries: Vec<_> = second["result"]["commits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|commit| commit["summary"].as_str().unwrap())
        .collect();
    assert_eq!(second_summaries, ["commit 3", "commit 2"]);
    let cursor = second["result"]["nextCursor"].as_str().unwrap();
    let third = history_response(&repository, json!({"limit": 2, "cursor": cursor}));
    assert_eq!(third["result"]["commits"][0]["summary"], "commit 1");
    assert!(third["result"]["nextCursor"].is_null());
}

#[test]
fn returns_merge_parents_identities_and_multiline_message() {
    let directory = TestDirectory::new("history-merge");
    git(&["init", "repo"], &directory.0);
    let repository = directory.0.join("repo");
    commit_file(&repository, 1, "initial");
    git(&["switch", "-c", "topic"], &repository);
    fs::write(repository.join("topic.txt"), "topic").unwrap();
    git(&["add", "topic.txt"], &repository);
    git(
        &[
            "-c",
            "user.name=Topic Author",
            "-c",
            "user.email=topic@gitnova.invalid",
            "commit",
            "-m",
            "topic",
        ],
        &repository,
    );
    git(&["checkout", "-"], &repository);
    commit_file(&repository, 2, "main");
    git(
        &[
            "-c",
            "user.name=Merge Author",
            "-c",
            "user.email=merge@gitnova.invalid",
            "merge",
            "--no-ff",
            "topic",
            "-m",
            "merge summary\n\nmerge body",
        ],
        &repository,
    );

    let response = history_response(&repository, json!({"limit": 1}));
    let commit = &response["result"]["commits"][0];
    assert_eq!(commit["parents"].as_array().unwrap().len(), 2);
    assert_eq!(commit["author"]["name"], "Merge Author");
    assert_eq!(commit["committer"]["email"], "merge@gitnova.invalid");
    assert_eq!(commit["summary"], "merge summary");
    assert_eq!(commit["message"], "merge summary\n\nmerge body\n");
    assert!(
        commit["author"]["timestamp"]
            .as_str()
            .unwrap()
            .contains('T')
    );
}

#[test]
fn supports_empty_detached_and_bare_repositories() {
    let directory = TestDirectory::new("history-kinds");
    git(&["init", "repo"], &directory.0);
    let repository = directory.0.join("repo");
    let empty = history_response(&repository, json!({}));
    assert!(empty["result"]["commits"].as_array().unwrap().is_empty());
    commit_file(&repository, 1, "initial");
    git(&["checkout", "--detach", "HEAD"], &repository);
    let detached = history_response(&repository, Value::Null);
    assert_eq!(detached["result"]["commits"][0]["summary"], "initial");
    git(
        &["clone", "--bare", repository.to_str().unwrap(), "bare.git"],
        &directory.0,
    );
    let bare = history_response(&directory.0.join("bare.git"), json!({}));
    assert_eq!(bare["result"]["commits"][0]["summary"], "initial");
}

#[test]
fn rejects_invalid_history_parameters_and_cursor_stably() {
    let directory = TestDirectory::new("history-errors");
    git(&["init", "repo"], &directory.0);
    let repository = directory.0.join("repo");
    commit_file(&repository, 1, "initial");
    let invalid_limit = history_response(&repository, json!({"limit": 0}));
    assert_eq!(invalid_limit["error"]["code"], -32602);
    let invalid_cursor = history_response(&repository, json!({"cursor": "not-a-cursor"}));
    assert_eq!(invalid_cursor["error"]["code"], -32111);
    assert_eq!(
        invalid_cursor["error"]["data"]["stableCode"],
        "history.invalid_cursor"
    );
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
fn github_repository_requires_session_and_valid_provider_identity() {
    let without_repository = run(&[
        initialize(json!(1)),
        json!({"jsonrpc":"2.0","id":2,"method":"github/repository","params":{"nameWithOwner":"owner/repo"}}),
    ]);
    let values = responses(&without_repository.stdout);
    assert_eq!(
        values[1]["error"]["data"]["stableCode"],
        "repository.not_open"
    );

    let directory = TestDirectory::new("github-context");
    git(&["init", "repo"], &directory.0);
    let repository = directory.0.join("repo");
    let output = run(&[
        initialize(json!(1)),
        repository_request(2, "repository/open", &repository),
        json!({"jsonrpc":"2.0","id":3,"method":"github/repository"}),
        json!({"jsonrpc":"2.0","id":4,"method":"github/repository","params":{"nameWithOwner":"owner/repo/extra"}}),
        json!({"jsonrpc":"2.0","id":5,"method":"github/repository","params":{"remote":"--upload-pack=evil"}}),
        json!({"jsonrpc":"2.0","id":6,"method":"github/repository","params":{"unexpected":true}}),
    ]);
    let values = responses(&output.stdout);
    assert_eq!(
        values[2]["error"]["data"]["stableCode"],
        "github.remote_not_found"
    );
    assert_eq!(
        values[3]["error"]["data"]["stableCode"],
        "github.unsupported_remote"
    );
    assert_eq!(
        values[4]["error"]["data"]["stableCode"],
        "github.invalid_remote"
    );
    assert_eq!(values[5]["error"]["code"], -32602);
    for value in &values[2..] {
        let serialized = serde_json::to_string(value).unwrap();
        assert!(!serialized.contains("evil"));
        assert!(!serialized.contains("token"));
    }
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
