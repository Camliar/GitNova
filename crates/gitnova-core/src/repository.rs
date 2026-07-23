use gitnova_protocol::{
    BranchStatus, CommitDiff, CommitGraphNode, CommitGraphPage, CommitIdentity, CommitSummary,
    DiffHunk, DiffLine, DiffLineKind, DiffScope, FileDiff, FileStatus, HistoryPage, ReferenceKind,
    RepositoryDescriptor, RepositoryHead, RepositoryKind, RepositoryReference,
    RepositoryReferences, StatusEntry, StatusEntryKind, WorkingTreeStatus,
};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Eq, PartialEq)]
pub enum RepositoryError {
    InvalidPath,
    UnsupportedPathEncoding,
    NotFound,
    GitUnavailable,
    GitCommandFailed,
    UnsafeRepository,
    WorktreeRequired,
    StatusParse,
    DiffParse,
    InvalidRepositoryPath,
    InvalidHistoryCursor,
    CommitParse,
    HistoryEncoding,
    CommitNotFound,
    CommitParentRequired,
    InvalidCommitParent,
    CommitDiffParse,
    ReferenceParse,
    ReferenceEncoding,
}

#[derive(Debug)]
struct CommandOutput {
    success: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

trait GitRunner {
    fn run(&self, arguments: &[OsString]) -> Result<CommandOutput, io::Error>;
}

struct SystemGit;

impl GitRunner for SystemGit {
    fn run(&self, arguments: &[OsString]) -> Result<CommandOutput, io::Error> {
        let output = Command::new("git")
            .args(arguments)
            .env("GIT_OPTIONAL_LOCKS", "0")
            .env("LC_ALL", "C")
            .output()?;
        Ok(CommandOutput {
            success: output.status.success(),
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

pub fn discover(path: &str) -> Result<RepositoryDescriptor, RepositoryError> {
    discover_with(&SystemGit, path)
}

pub fn status(descriptor: &RepositoryDescriptor) -> Result<WorkingTreeStatus, RepositoryError> {
    let worktree = descriptor
        .worktree_root
        .as_ref()
        .ok_or(RepositoryError::WorktreeRequired)?;
    let arguments = [
        OsString::from("-C"),
        OsString::from(worktree),
        OsString::from("status"),
        OsString::from("--porcelain=v2"),
        OsString::from("-z"),
        OsString::from("--branch"),
        OsString::from("--untracked-files=all"),
        OsString::from("--renames"),
    ];
    let output = SystemGit.run(&arguments).map_err(map_io_error)?;
    if !output.success {
        return Err(map_failed_output(&output.stderr));
    }
    parse_status(&output.stdout)
}

pub fn diff(
    descriptor: &RepositoryDescriptor,
    path: &str,
    scope: DiffScope,
    context_lines: u8,
) -> Result<FileDiff, RepositoryError> {
    let worktree = descriptor
        .worktree_root
        .as_ref()
        .ok_or(RepositoryError::WorktreeRequired)?;
    validate_repository_path(path)?;
    let mut arguments = vec![
        OsString::from("--literal-pathspecs"),
        OsString::from("-C"),
        OsString::from(worktree),
        OsString::from("-c"),
        OsString::from("core.quotePath=false"),
        OsString::from("diff"),
        OsString::from("--patch"),
        OsString::from("--no-color"),
        OsString::from("--no-ext-diff"),
        OsString::from("--no-textconv"),
        OsString::from("--find-renames"),
        OsString::from(format!("--unified={context_lines}")),
    ];
    if scope == DiffScope::Staged {
        arguments.push(OsString::from("--cached"));
    }
    arguments.push(OsString::from("--"));
    arguments.push(OsString::from(path));
    let output = SystemGit.run(&arguments).map_err(map_io_error)?;
    if !output.success {
        return Err(map_failed_output(&output.stderr));
    }
    parse_diff(&output.stdout, path)
}

pub fn history(
    descriptor: &RepositoryDescriptor,
    limit: u16,
    cursor: Option<&str>,
) -> Result<HistoryPage, RepositoryError> {
    let base = descriptor
        .worktree_root
        .as_ref()
        .unwrap_or(&descriptor.git_directory);
    let (snapshot, offset) = match cursor {
        Some(cursor) => parse_history_cursor(cursor)?,
        None => match resolve_head(base)? {
            Some(head) => (head, 0),
            None => {
                return Ok(HistoryPage {
                    commits: Vec::new(),
                    next_cursor: None,
                });
            }
        },
    };
    let requested = usize::from(limit) + 1;
    let arguments = [
        OsString::from("-C"),
        OsString::from(base),
        OsString::from("rev-list"),
        OsString::from("--topo-order"),
        OsString::from("--date-order"),
        OsString::from(format!("--max-count={requested}")),
        OsString::from(format!("--skip={offset}")),
        OsString::from(&snapshot),
    ];
    let output = SystemGit.run(&arguments).map_err(map_io_error)?;
    if !output.success {
        return Err(RepositoryError::InvalidHistoryCursor);
    }
    let oid_list = std::str::from_utf8(&output.stdout).map_err(|_| RepositoryError::CommitParse)?;
    let mut oids: Vec<&str> = oid_list.lines().filter(|line| !line.is_empty()).collect();
    let has_more = oids.len() > usize::from(limit);
    oids.truncate(usize::from(limit));
    let mut commits = Vec::with_capacity(oids.len());
    for oid in oids {
        let arguments = [
            OsString::from("-C"),
            OsString::from(base),
            OsString::from("cat-file"),
            OsString::from("commit"),
            OsString::from(oid),
        ];
        let output = SystemGit.run(&arguments).map_err(map_io_error)?;
        if !output.success {
            return Err(RepositoryError::CommitParse);
        }
        commits.push(parse_commit(oid, &output.stdout)?);
    }
    let next_offset = offset
        .checked_add(usize::from(limit))
        .ok_or(RepositoryError::InvalidHistoryCursor)?;
    Ok(HistoryPage {
        commits,
        next_cursor: has_more.then(|| format_history_cursor(&snapshot, next_offset)),
    })
}

pub fn commit_diff(
    descriptor: &RepositoryDescriptor,
    oid: &str,
    parent_oid: Option<&str>,
    context_lines: u8,
) -> Result<CommitDiff, RepositoryError> {
    let base = descriptor
        .worktree_root
        .as_ref()
        .unwrap_or(&descriptor.git_directory);
    let oid = oid.to_ascii_lowercase();
    let commit = load_commit(base, &oid)?;
    let selected_parent = match (commit.parents.as_slice(), parent_oid) {
        ([], None) => None,
        ([], Some(_)) => return Err(RepositoryError::InvalidCommitParent),
        ([only], None) => Some(only.clone()),
        ([_, _, ..], None) => return Err(RepositoryError::CommitParentRequired),
        (parents, Some(parent)) => match parents
            .iter()
            .find(|candidate| candidate.eq_ignore_ascii_case(parent))
        {
            Some(parent) => Some(parent.clone()),
            None => return Err(RepositoryError::InvalidCommitParent),
        },
    };

    let changed_paths = commit_changed_paths(base, &oid, selected_parent.as_deref())?;
    let mut files = Vec::with_capacity(changed_paths.len());
    for changed_path in changed_paths {
        let mut arguments = vec![
            OsString::from("--literal-pathspecs"),
            OsString::from("-C"),
            OsString::from(base),
            OsString::from("-c"),
            OsString::from("core.quotePath=false"),
        ];
        if let Some(parent) = &selected_parent {
            arguments.extend([
                OsString::from("diff"),
                OsString::from("--patch"),
                OsString::from("--no-color"),
                OsString::from("--no-ext-diff"),
                OsString::from("--no-textconv"),
                OsString::from("--find-renames"),
                OsString::from(format!("--unified={context_lines}")),
                OsString::from(parent),
                OsString::from(&oid),
            ]);
        } else {
            arguments.extend([
                OsString::from("diff-tree"),
                OsString::from("--root"),
                OsString::from("--no-commit-id"),
                OsString::from("-r"),
                OsString::from("--patch"),
                OsString::from("--no-color"),
                OsString::from("--no-ext-diff"),
                OsString::from("--no-textconv"),
                OsString::from("--find-renames"),
                OsString::from(format!("--unified={context_lines}")),
                OsString::from(&oid),
            ]);
        }
        arguments.push(OsString::from("--"));
        arguments.push(OsString::from(&changed_path.old_path));
        if changed_path.new_path != changed_path.old_path {
            arguments.push(OsString::from(&changed_path.new_path));
        }
        let output = SystemGit.run(&arguments).map_err(map_io_error)?;
        if !output.success {
            return Err(map_failed_output(&output.stderr));
        }
        files.push(parse_diff(&output.stdout, &changed_path.new_path)?);
    }
    Ok(CommitDiff {
        commit,
        parent_oid: selected_parent,
        files,
    })
}

pub fn commit_parents_if_available(
    descriptor: &RepositoryDescriptor,
    oid: &str,
) -> Result<Option<Vec<String>>, RepositoryError> {
    commit_parents_if_available_with(&SystemGit, descriptor, oid)
}

fn commit_parents_if_available_with(
    git: &impl GitRunner,
    descriptor: &RepositoryDescriptor,
    oid: &str,
) -> Result<Option<Vec<String>>, RepositoryError> {
    let base = descriptor
        .worktree_root
        .as_ref()
        .unwrap_or(&descriptor.git_directory);
    let arguments = [
        OsString::from("-C"),
        OsString::from(base),
        OsString::from("cat-file"),
        OsString::from("-e"),
        OsString::from(format!("{oid}^{{commit}}")),
    ];
    let output = git.run(&arguments).map_err(map_io_error)?;
    if !output.success {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Not a valid object")
            || stderr.contains("bad object")
            || stderr.contains("could not get object info")
        {
            return Ok(None);
        }
        return Err(map_failed_output(&output.stderr));
    }
    let arguments = [
        OsString::from("-C"),
        OsString::from(base),
        OsString::from("rev-list"),
        OsString::from("--parents"),
        OsString::from("--max-count=1"),
        OsString::from(oid),
    ];
    let output = git.run(&arguments).map_err(map_io_error)?;
    if !output.success {
        return Err(map_failed_output(&output.stderr));
    }
    let line = std::str::from_utf8(&output.stdout)
        .map_err(|_| RepositoryError::CommitParse)?
        .trim_end_matches(['\r', '\n']);
    let mut oids = line.split_whitespace();
    let commit_oid = oids.next().ok_or(RepositoryError::CommitParse)?;
    if !commit_oid.eq_ignore_ascii_case(oid) {
        return Err(RepositoryError::CommitParse);
    }
    let parents = oids
        .map(|parent| {
            if valid_oid(parent) {
                Ok(parent.to_owned())
            } else {
                Err(RepositoryError::CommitParse)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(parents))
}

pub fn references(
    descriptor: &RepositoryDescriptor,
) -> Result<RepositoryReferences, RepositoryError> {
    let base = descriptor
        .worktree_root
        .as_ref()
        .unwrap_or(&descriptor.git_directory);
    let head = RepositoryHead {
        oid: resolve_head(base)?,
        symbolic_ref: resolve_symbolic_head(base)?,
    };
    let arguments = [
        OsString::from("-C"),
        OsString::from(base),
        OsString::from("for-each-ref"),
        OsString::from("--sort=refname"),
        OsString::from(
            "--format=%(refname)%00%(objectname)%00%(*objectname)%00%(symref)%00%(upstream)",
        ),
        OsString::from("refs/heads"),
        OsString::from("refs/remotes"),
        OsString::from("refs/tags"),
    ];
    let output = SystemGit.run(&arguments).map_err(map_io_error)?;
    if !output.success {
        return Err(map_failed_output(&output.stderr));
    }
    Ok(RepositoryReferences {
        head,
        references: parse_references(&output.stdout)?,
    })
}

pub fn graph(
    descriptor: &RepositoryDescriptor,
    limit: u16,
    cursor: Option<&str>,
) -> Result<CommitGraphPage, RepositoryError> {
    let history = history(descriptor, limit, cursor)?;
    let repository_references = references(descriptor)?;
    let head_oid = repository_references.head.oid.as_deref();
    let nodes = history
        .commits
        .into_iter()
        .map(|commit| {
            let references = repository_references
                .references
                .iter()
                .filter(|reference| {
                    let decoration_oid = if reference.kind == ReferenceKind::Tag {
                        reference
                            .peeled_target_oid
                            .as_ref()
                            .unwrap_or(&reference.target_oid)
                    } else {
                        &reference.target_oid
                    };
                    decoration_oid == &commit.oid
                })
                .cloned()
                .collect();
            CommitGraphNode {
                is_head: head_oid == Some(commit.oid.as_str()),
                commit,
                references,
            }
        })
        .collect();
    Ok(CommitGraphPage {
        nodes,
        next_cursor: history.next_cursor,
    })
}

fn resolve_symbolic_head(base: &str) -> Result<Option<String>, RepositoryError> {
    let arguments = [
        OsString::from("-C"),
        OsString::from(base),
        OsString::from("symbolic-ref"),
        OsString::from("-q"),
        OsString::from("HEAD"),
    ];
    let output = SystemGit.run(&arguments).map_err(map_io_error)?;
    if !output.success {
        if output.stderr.is_empty() {
            return Ok(None);
        }
        return Err(map_failed_output(&output.stderr));
    }
    let value = std::str::from_utf8(&output.stdout)
        .map_err(|_| RepositoryError::ReferenceEncoding)?
        .trim_end_matches(['\r', '\n']);
    if value.starts_with("refs/heads/") {
        Ok(Some(value.to_owned()))
    } else {
        Err(RepositoryError::ReferenceParse)
    }
}

fn parse_references(output: &[u8]) -> Result<Vec<RepositoryReference>, RepositoryError> {
    let mut references = Vec::new();
    for record in output.split(|byte| *byte == b'\n') {
        let record = record.strip_suffix(b"\r").unwrap_or(record);
        if record.is_empty() {
            continue;
        }
        let fields: Vec<&[u8]> = record.split(|byte| *byte == 0).collect();
        if fields.len() != 5 {
            return Err(RepositoryError::ReferenceParse);
        }
        let full_name = decode_reference_field(fields[0])?;
        let (kind, name) = if let Some(name) = full_name.strip_prefix("refs/heads/") {
            (ReferenceKind::LocalBranch, name)
        } else if let Some(name) = full_name.strip_prefix("refs/remotes/") {
            (ReferenceKind::RemoteBranch, name)
        } else if let Some(name) = full_name.strip_prefix("refs/tags/") {
            (ReferenceKind::Tag, name)
        } else {
            return Err(RepositoryError::ReferenceParse);
        };
        let target_oid = decode_reference_field(fields[1])?;
        if !valid_oid(target_oid) {
            return Err(RepositoryError::ReferenceParse);
        }
        let optional_oid = |field: &[u8]| -> Result<Option<String>, RepositoryError> {
            if field.is_empty() {
                return Ok(None);
            }
            let oid = decode_reference_field(field)?;
            if valid_oid(oid) {
                Ok(Some(oid.to_owned()))
            } else {
                Err(RepositoryError::ReferenceParse)
            }
        };
        let optional_text = |field: &[u8]| -> Result<Option<String>, RepositoryError> {
            if field.is_empty() {
                Ok(None)
            } else {
                Ok(Some(decode_reference_field(field)?.to_owned()))
            }
        };
        references.push(RepositoryReference {
            name: name.to_owned(),
            full_name: full_name.to_owned(),
            kind,
            target_oid: target_oid.to_owned(),
            peeled_target_oid: optional_oid(fields[2])?,
            symbolic_target: optional_text(fields[3])?,
            upstream: optional_text(fields[4])?,
        });
    }
    Ok(references)
}

fn decode_reference_field(field: &[u8]) -> Result<&str, RepositoryError> {
    std::str::from_utf8(field).map_err(|_| RepositoryError::ReferenceEncoding)
}

fn load_commit(base: &str, oid: &str) -> Result<CommitSummary, RepositoryError> {
    let arguments = [
        OsString::from("-C"),
        OsString::from(base),
        OsString::from("cat-file"),
        OsString::from("commit"),
        OsString::from(oid),
    ];
    let output = SystemGit.run(&arguments).map_err(map_io_error)?;
    if !output.success {
        return Err(RepositoryError::CommitNotFound);
    }
    parse_commit(oid, &output.stdout)
}

fn commit_changed_paths(
    base: &str,
    oid: &str,
    parent_oid: Option<&str>,
) -> Result<Vec<ChangedPath>, RepositoryError> {
    let mut arguments = vec![
        OsString::from("-C"),
        OsString::from(base),
        OsString::from("-c"),
        OsString::from("core.quotePath=false"),
    ];
    if let Some(parent) = parent_oid {
        arguments.extend([
            OsString::from("diff"),
            OsString::from("--name-status"),
            OsString::from("-z"),
            OsString::from("--find-renames"),
            OsString::from(parent),
            OsString::from(oid),
        ]);
    } else {
        arguments.extend([
            OsString::from("diff-tree"),
            OsString::from("--root"),
            OsString::from("--no-commit-id"),
            OsString::from("-r"),
            OsString::from("--name-status"),
            OsString::from("-z"),
            OsString::from("--find-renames"),
            OsString::from(oid),
        ]);
    }
    let output = SystemGit.run(&arguments).map_err(map_io_error)?;
    if !output.success {
        return Err(map_failed_output(&output.stderr));
    }
    parse_changed_paths(&output.stdout)
}

#[derive(Debug, Eq, PartialEq)]
struct ChangedPath {
    old_path: String,
    new_path: String,
}

fn parse_changed_paths(output: &[u8]) -> Result<Vec<ChangedPath>, RepositoryError> {
    let records: Vec<&[u8]> = output.split(|byte| *byte == 0).collect();
    let mut paths = Vec::new();
    let mut index = 0;
    while index < records.len() {
        let status = records[index];
        index += 1;
        if status.is_empty() {
            continue;
        }
        let status = std::str::from_utf8(status).map_err(|_| RepositoryError::CommitDiffParse)?;
        let status_kind = status
            .as_bytes()
            .first()
            .ok_or(RepositoryError::CommitDiffParse)?;
        if !matches!(
            status_kind,
            b'A' | b'C' | b'D' | b'M' | b'R' | b'T' | b'U' | b'X' | b'B'
        ) {
            return Err(RepositoryError::CommitDiffParse);
        }
        let renamed = matches!(status_kind, b'R' | b'C');
        let old_record = records.get(index).ok_or(RepositoryError::CommitDiffParse)?;
        let new_record = if renamed {
            records
                .get(index + 1)
                .ok_or(RepositoryError::CommitDiffParse)?
        } else {
            old_record
        };
        let old_path = std::str::from_utf8(old_record)
            .map_err(|_| RepositoryError::UnsupportedPathEncoding)?;
        let new_path = std::str::from_utf8(new_record)
            .map_err(|_| RepositoryError::UnsupportedPathEncoding)?;
        validate_repository_path(old_path).map_err(|_| RepositoryError::CommitDiffParse)?;
        validate_repository_path(new_path).map_err(|_| RepositoryError::CommitDiffParse)?;
        paths.push(ChangedPath {
            old_path: old_path.to_owned(),
            new_path: new_path.to_owned(),
        });
        let path_count = if renamed { 2 } else { 1 };
        index += path_count;
    }
    Ok(paths)
}

fn resolve_head(base: &str) -> Result<Option<String>, RepositoryError> {
    let arguments = [
        OsString::from("-C"),
        OsString::from(base),
        OsString::from("rev-parse"),
        OsString::from("--verify"),
        OsString::from("HEAD"),
    ];
    let output = SystemGit.run(&arguments).map_err(map_io_error)?;
    if !output.success {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Needed a single revision")
            || stderr.contains("unknown revision")
            || stderr.contains("ambiguous argument 'HEAD'")
        {
            return Ok(None);
        }
        return Err(map_failed_output(&output.stderr));
    }
    let head = std::str::from_utf8(&output.stdout)
        .map_err(|_| RepositoryError::CommitParse)?
        .trim_end_matches(['\r', '\n']);
    if valid_oid(head) {
        Ok(Some(head.to_owned()))
    } else {
        Err(RepositoryError::CommitParse)
    }
}

fn format_history_cursor(snapshot: &str, offset: usize) -> String {
    format!("v1:{snapshot}:{offset}")
}

fn parse_history_cursor(cursor: &str) -> Result<(String, usize), RepositoryError> {
    let mut parts = cursor.split(':');
    let version = parts.next();
    let snapshot = parts.next();
    let offset = parts.next();
    if version != Some("v1") || parts.next().is_some() {
        return Err(RepositoryError::InvalidHistoryCursor);
    }
    let snapshot = snapshot
        .filter(|value| valid_oid(value))
        .ok_or(RepositoryError::InvalidHistoryCursor)?;
    let offset = offset
        .ok_or(RepositoryError::InvalidHistoryCursor)?
        .parse::<usize>()
        .map_err(|_| RepositoryError::InvalidHistoryCursor)?;
    Ok((snapshot.to_owned(), offset))
}

pub(crate) fn valid_oid(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn parse_commit(oid: &str, raw: &[u8]) -> Result<CommitSummary, RepositoryError> {
    let separator = raw
        .windows(2)
        .position(|window| window == b"\n\n")
        .ok_or(RepositoryError::CommitParse)?;
    let headers =
        std::str::from_utf8(&raw[..separator]).map_err(|_| RepositoryError::HistoryEncoding)?;
    let message = std::str::from_utf8(&raw[separator + 2..])
        .map_err(|_| RepositoryError::HistoryEncoding)?
        .to_owned();
    let mut parents = Vec::new();
    let mut author = None;
    let mut committer = None;
    for line in headers.lines() {
        if line.starts_with(' ') {
            continue;
        }
        if let Some(value) = line.strip_prefix("parent ") {
            if !valid_oid(value) {
                return Err(RepositoryError::CommitParse);
            }
            parents.push(value.to_owned());
        } else if let Some(value) = line.strip_prefix("author ") {
            author = Some(parse_identity(value)?);
        } else if let Some(value) = line.strip_prefix("committer ") {
            committer = Some(parse_identity(value)?);
        } else if let Some(encoding) = line.strip_prefix("encoding ")
            && !encoding.eq_ignore_ascii_case("utf-8")
        {
            return Err(RepositoryError::HistoryEncoding);
        }
    }
    let summary = message.lines().next().unwrap_or_default().to_owned();
    Ok(CommitSummary {
        oid: oid.to_owned(),
        parents,
        author: author.ok_or(RepositoryError::CommitParse)?,
        committer: committer.ok_or(RepositoryError::CommitParse)?,
        summary,
        message,
    })
}

fn parse_identity(value: &str) -> Result<CommitIdentity, RepositoryError> {
    let email_end = value.rfind('>').ok_or(RepositoryError::CommitParse)?;
    let email_start = value[..email_end]
        .rfind('<')
        .ok_or(RepositoryError::CommitParse)?;
    let name = value[..email_start].trim_end().to_owned();
    let email = value[email_start + 1..email_end].to_owned();
    let mut timestamp_parts = value[email_end + 1..].split_whitespace();
    let seconds = timestamp_parts
        .next()
        .ok_or(RepositoryError::CommitParse)?
        .parse::<i64>()
        .map_err(|_| RepositoryError::CommitParse)?;
    let offset = timestamp_parts.next().ok_or(RepositoryError::CommitParse)?;
    if timestamp_parts.next().is_some() {
        return Err(RepositoryError::CommitParse);
    }
    Ok(CommitIdentity {
        name,
        email,
        timestamp: format_git_timestamp(seconds, offset)?,
    })
}

fn format_git_timestamp(seconds: i64, offset: &str) -> Result<String, RepositoryError> {
    if offset.len() != 5 || !matches!(offset.as_bytes()[0], b'+' | b'-') {
        return Err(RepositoryError::CommitParse);
    }
    let hours = offset[1..3]
        .parse::<i64>()
        .map_err(|_| RepositoryError::CommitParse)?;
    let minutes = offset[3..5]
        .parse::<i64>()
        .map_err(|_| RepositoryError::CommitParse)?;
    if hours > 23 || minutes > 59 {
        return Err(RepositoryError::CommitParse);
    }
    let sign = if &offset[..1] == "+" { 1 } else { -1 };
    let local_seconds = seconds + sign * (hours * 3600 + minutes * 60);
    let days = local_seconds.div_euclid(86_400);
    let day_seconds = local_seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = day_seconds / 3600;
    let minute = day_seconds % 3600 / 60;
    let second = day_seconds % 60;
    Ok(format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}{}:{}",
        &offset[..3],
        &offset[3..]
    ))
}

fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

fn validate_repository_path(path: &str) -> Result<(), RepositoryError> {
    if path.is_empty()
        || path.contains(['\r', '\n'])
        || path.starts_with(':')
        || path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(RepositoryError::InvalidRepositoryPath);
    }
    let mut has_normal = false;
    for component in Path::new(path).components() {
        match component {
            std::path::Component::Normal(_) => has_normal = true,
            _ => return Err(RepositoryError::InvalidRepositoryPath),
        }
    }
    if has_normal {
        Ok(())
    } else {
        Err(RepositoryError::InvalidRepositoryPath)
    }
}

fn discover_with(
    git: &impl GitRunner,
    path: &str,
) -> Result<RepositoryDescriptor, RepositoryError> {
    if path.is_empty() {
        return Err(RepositoryError::InvalidPath);
    }
    let canonical = fs::canonicalize(path).map_err(|error| match error.kind() {
        io::ErrorKind::NotFound => RepositoryError::InvalidPath,
        _ => RepositoryError::GitCommandFailed,
    })?;
    let search_path = if canonical.is_file() {
        canonical.parent().ok_or(RepositoryError::InvalidPath)?
    } else {
        &canonical
    };

    let git_version = run_git(git, [OsStr::new("--version")])?;
    let git_version = git_version
        .strip_prefix("git version ")
        .unwrap_or(&git_version)
        .to_owned();
    let bare = run_in(git, search_path, ["rev-parse", "--is-bare-repository"])? == "true";
    let inside_worktree =
        run_in(git, search_path, ["rev-parse", "--is-inside-work-tree"])? == "true";
    if !bare && !inside_worktree {
        return Err(RepositoryError::NotFound);
    }

    let git_directory = resolve_git_path(
        search_path,
        &run_in(git, search_path, ["rev-parse", "--git-dir"])?,
    )?;
    let common_git_directory = resolve_git_path(
        search_path,
        &run_in(git, search_path, ["rev-parse", "--git-common-dir"])?,
    )?;
    let worktree_root = if bare {
        None
    } else {
        Some(resolve_git_path(
            search_path,
            &run_in(git, search_path, ["rev-parse", "--show-toplevel"])?,
        )?)
    };
    let kind = if bare {
        RepositoryKind::Bare
    } else if git_directory != common_git_directory {
        RepositoryKind::LinkedWorktree
    } else {
        RepositoryKind::Worktree
    };

    Ok(RepositoryDescriptor {
        worktree_root: worktree_root.map(path_to_string).transpose()?,
        git_directory: path_to_string(git_directory)?,
        common_git_directory: path_to_string(common_git_directory)?,
        kind,
        git_version,
    })
}

fn run_in<const N: usize>(
    git: &impl GitRunner,
    directory: &Path,
    arguments: [&str; N],
) -> Result<String, RepositoryError> {
    let mut complete = vec![OsString::from("-C"), directory.as_os_str().to_owned()];
    complete.extend(arguments.into_iter().map(OsString::from));
    run_git(git, complete)
}

fn run_git(
    git: &impl GitRunner,
    arguments: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<String, RepositoryError> {
    let arguments: Vec<OsString> = arguments
        .into_iter()
        .map(|argument| argument.as_ref().to_owned())
        .collect();
    let output = git.run(&arguments).map_err(map_io_error)?;
    if !output.success {
        return Err(map_failed_output(&output.stderr));
    }
    String::from_utf8(output.stdout)
        .map(|value| {
            value
                .strip_suffix("\r\n")
                .or_else(|| value.strip_suffix('\n'))
                .unwrap_or(&value)
                .to_owned()
        })
        .map_err(|_| RepositoryError::UnsupportedPathEncoding)
}

fn map_io_error(error: io::Error) -> RepositoryError {
    if error.kind() == io::ErrorKind::NotFound {
        RepositoryError::GitUnavailable
    } else {
        RepositoryError::GitCommandFailed
    }
}

fn map_failed_output(stderr: &[u8]) -> RepositoryError {
    let stderr = String::from_utf8_lossy(stderr);
    if stderr.contains("dubious ownership") {
        RepositoryError::UnsafeRepository
    } else if stderr.contains("not a git repository") {
        RepositoryError::NotFound
    } else {
        RepositoryError::GitCommandFailed
    }
}

fn parse_status(output: &[u8]) -> Result<WorkingTreeStatus, RepositoryError> {
    let mut branch = BranchStatus {
        head: None,
        oid: None,
        upstream: None,
        ahead: 0,
        behind: 0,
    };
    let records: Vec<&[u8]> = output.split(|byte| *byte == 0).collect();
    let mut entries = Vec::new();
    let mut index = 0;
    while index < records.len() {
        let record = records[index];
        index += 1;
        if record.is_empty() {
            continue;
        }
        let record =
            std::str::from_utf8(record).map_err(|_| RepositoryError::UnsupportedPathEncoding)?;
        if let Some(header) = record.strip_prefix("# ") {
            parse_branch_header(header, &mut branch)?;
            continue;
        }
        if let Some(path) = record.strip_prefix("? ") {
            entries.push(StatusEntry {
                path: path.to_owned(),
                original_path: None,
                kind: StatusEntryKind::Untracked,
                index_status: FileStatus::Unmodified,
                worktree_status: FileStatus::Untracked,
            });
            continue;
        }
        let record_type = record.as_bytes()[0];
        let field_count = match record_type {
            b'1' => 9,
            b'2' => 10,
            b'u' => 11,
            _ => return Err(RepositoryError::StatusParse),
        };
        let fields: Vec<&str> = record.splitn(field_count, ' ').collect();
        if fields.len() != field_count || fields[1].len() != 2 {
            return Err(RepositoryError::StatusParse);
        }
        let mut xy = fields[1].chars();
        let index_status = map_status(xy.next().ok_or(RepositoryError::StatusParse)?);
        let worktree_status = map_status(xy.next().ok_or(RepositoryError::StatusParse)?);
        let (path, original_path, kind) = match record_type {
            b'1' => (fields[8].to_owned(), None, StatusEntryKind::Ordinary),
            b'2' => {
                let original = records.get(index).ok_or(RepositoryError::StatusParse)?;
                index += 1;
                let original = std::str::from_utf8(original)
                    .map_err(|_| RepositoryError::UnsupportedPathEncoding)?;
                (
                    fields[9].to_owned(),
                    Some(original.to_owned()),
                    StatusEntryKind::RenameOrCopy,
                )
            }
            b'u' => (fields[10].to_owned(), None, StatusEntryKind::Unmerged),
            _ => unreachable!(),
        };
        entries.push(StatusEntry {
            path,
            original_path,
            kind,
            index_status,
            worktree_status,
        });
    }
    Ok(WorkingTreeStatus { branch, entries })
}

fn parse_branch_header(header: &str, branch: &mut BranchStatus) -> Result<(), RepositoryError> {
    if let Some(value) = header.strip_prefix("branch.oid ") {
        branch.oid = (value != "(initial)").then(|| value.to_owned());
    } else if let Some(value) = header.strip_prefix("branch.head ") {
        branch.head = (value != "(detached)").then(|| value.to_owned());
    } else if let Some(value) = header.strip_prefix("branch.upstream ") {
        branch.upstream = Some(value.to_owned());
    } else if let Some(value) = header.strip_prefix("branch.ab ") {
        let (ahead, behind) = value.split_once(' ').ok_or(RepositoryError::StatusParse)?;
        branch.ahead = ahead
            .strip_prefix('+')
            .ok_or(RepositoryError::StatusParse)?
            .parse()
            .map_err(|_| RepositoryError::StatusParse)?;
        branch.behind = behind
            .strip_prefix('-')
            .ok_or(RepositoryError::StatusParse)?
            .parse()
            .map_err(|_| RepositoryError::StatusParse)?;
    }
    Ok(())
}

fn map_status(status: char) -> FileStatus {
    match status {
        '.' => FileStatus::Unmodified,
        'M' => FileStatus::Modified,
        'A' => FileStatus::Added,
        'D' => FileStatus::Deleted,
        'R' => FileStatus::Renamed,
        'C' => FileStatus::Copied,
        'U' => FileStatus::Unmerged,
        '?' => FileStatus::Untracked,
        'T' => FileStatus::TypeChanged,
        _ => FileStatus::Unknown,
    }
}

fn parse_diff(output: &[u8], path: &str) -> Result<FileDiff, RepositoryError> {
    let patch =
        std::str::from_utf8(output).map_err(|_| RepositoryError::UnsupportedPathEncoding)?;
    let mut old_path = path.to_owned();
    let mut new_path = path.to_owned();
    let mut is_binary = false;
    for line in patch.split_terminator('\n') {
        if let Some(value) = line.strip_prefix("rename from ") {
            old_path = value.to_owned();
        } else if let Some(value) = line.strip_prefix("rename to ") {
            new_path = value.to_owned();
        } else if let Some(value) = line.strip_prefix("--- a/") {
            old_path = value.to_owned();
        } else if let Some(value) = line.strip_prefix("+++ b/") {
            new_path = value.to_owned();
        } else if line.starts_with("Binary files ") || line == "GIT binary patch" {
            is_binary = true;
        }
    }
    if is_binary {
        return Ok(FileDiff {
            old_path,
            new_path,
            is_binary: true,
            hunks: Vec::new(),
        });
    }

    let mut hunks: Vec<DiffHunk> = Vec::new();
    let mut old_line = 0;
    let mut new_line = 0;
    for line in patch.split_terminator('\n') {
        if line.starts_with("@@ ") {
            let (old_start, old_lines, new_start, new_lines, header) = parse_hunk_header(line)?;
            old_line = old_start;
            new_line = new_start;
            hunks.push(DiffHunk {
                old_start,
                old_lines,
                new_start,
                new_lines,
                header,
                lines: Vec::new(),
            });
            continue;
        }
        let Some(hunk) = hunks.last_mut() else {
            continue;
        };
        if line == "\\ No newline at end of file" {
            continue;
        }
        let (kind, content, old_number, new_number) = match line.as_bytes().first() {
            Some(b' ') => {
                let numbers = (Some(old_line), Some(new_line));
                old_line += 1;
                new_line += 1;
                (DiffLineKind::Context, &line[1..], numbers.0, numbers.1)
            }
            Some(b'+') => {
                let number = new_line;
                new_line += 1;
                (DiffLineKind::Addition, &line[1..], None, Some(number))
            }
            Some(b'-') => {
                let number = old_line;
                old_line += 1;
                (DiffLineKind::Deletion, &line[1..], Some(number), None)
            }
            _ => return Err(RepositoryError::DiffParse),
        };
        hunk.lines.push(DiffLine {
            kind,
            content: content.to_owned(),
            old_line: old_number,
            new_line: new_number,
        });
    }

    for hunk in &hunks {
        let old_count = hunk
            .lines
            .iter()
            .filter(|line| line.old_line.is_some())
            .count() as u64;
        let new_count = hunk
            .lines
            .iter()
            .filter(|line| line.new_line.is_some())
            .count() as u64;
        if old_count != hunk.old_lines || new_count != hunk.new_lines {
            return Err(RepositoryError::DiffParse);
        }
    }

    Ok(FileDiff {
        old_path,
        new_path,
        is_binary: false,
        hunks,
    })
}

fn parse_hunk_header(line: &str) -> Result<(u64, u64, u64, u64, String), RepositoryError> {
    let rest = line
        .strip_prefix("@@ -")
        .ok_or(RepositoryError::DiffParse)?;
    let (ranges, header) = rest.split_once(" @@").ok_or(RepositoryError::DiffParse)?;
    let (old_range, new_range) = ranges.split_once(" +").ok_or(RepositoryError::DiffParse)?;
    let (old_start, old_lines) = parse_hunk_range(old_range)?;
    let (new_start, new_lines) = parse_hunk_range(new_range)?;
    Ok((
        old_start,
        old_lines,
        new_start,
        new_lines,
        header.strip_prefix(' ').unwrap_or(header).to_owned(),
    ))
}

fn parse_hunk_range(range: &str) -> Result<(u64, u64), RepositoryError> {
    let (start, lines) = range.split_once(',').unwrap_or((range, "1"));
    Ok((
        start.parse().map_err(|_| RepositoryError::DiffParse)?,
        lines.parse().map_err(|_| RepositoryError::DiffParse)?,
    ))
}

fn resolve_git_path(base: &Path, value: &str) -> Result<PathBuf, RepositoryError> {
    let path = Path::new(value);
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        base.join(path)
    };
    fs::canonicalize(absolute).map_err(|_| RepositoryError::GitCommandFailed)
}

fn path_to_string(path: PathBuf) -> Result<String, RepositoryError> {
    path.into_os_string()
        .into_string()
        .map_err(|_| RepositoryError::UnsupportedPathEncoding)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    struct FakeGit {
        outputs: Mutex<VecDeque<Result<CommandOutput, io::Error>>>,
    }

    impl GitRunner for FakeGit {
        fn run(&self, _arguments: &[OsString]) -> Result<CommandOutput, io::Error> {
            self.outputs.lock().unwrap().pop_front().unwrap()
        }
    }

    #[test]
    fn unavailable_git_has_a_distinct_error() {
        let git = FakeGit {
            outputs: Mutex::new(VecDeque::from([Err(io::Error::new(
                io::ErrorKind::NotFound,
                "git missing",
            ))])),
        };
        assert_eq!(
            discover_with(&git, "."),
            Err(RepositoryError::GitUnavailable)
        );
    }

    #[test]
    fn inspects_commit_parents_and_treats_only_missing_objects_as_unavailable() {
        let descriptor = RepositoryDescriptor {
            worktree_root: Some("/repo".into()),
            git_directory: "/repo/.git".into(),
            common_git_directory: "/repo/.git".into(),
            kind: RepositoryKind::Worktree,
            git_version: "test".into(),
        };
        let oid = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let parent = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let present = FakeGit {
            outputs: Mutex::new(VecDeque::from([
                Ok(CommandOutput {
                    success: true,
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                }),
                Ok(CommandOutput {
                    success: true,
                    stdout: format!("{oid} {parent}\n").into_bytes(),
                    stderr: Vec::new(),
                }),
            ])),
        };
        assert_eq!(
            commit_parents_if_available_with(&present, &descriptor, oid).unwrap(),
            Some(vec![parent.into()])
        );

        let missing = FakeGit {
            outputs: Mutex::new(VecDeque::from([Ok(CommandOutput {
                success: false,
                stdout: Vec::new(),
                stderr: b"fatal: Not a valid object name".to_vec(),
            })])),
        };
        assert_eq!(
            commit_parents_if_available_with(&missing, &descriptor, oid).unwrap(),
            None
        );

        let unsafe_repository = FakeGit {
            outputs: Mutex::new(VecDeque::from([Ok(CommandOutput {
                success: false,
                stdout: Vec::new(),
                stderr: b"fatal: detected dubious ownership in repository".to_vec(),
            })])),
        };
        assert_eq!(
            commit_parents_if_available_with(&unsafe_repository, &descriptor, oid),
            Err(RepositoryError::UnsafeRepository)
        );
    }

    #[test]
    fn unsafe_repository_error_is_not_bypassed() {
        let git = FakeGit {
            outputs: Mutex::new(VecDeque::from([
                Ok(CommandOutput {
                    success: true,
                    stdout: b"git version 2.40.0\n".to_vec(),
                    stderr: Vec::new(),
                }),
                Ok(CommandOutput {
                    success: false,
                    stdout: Vec::new(),
                    stderr: b"fatal: detected dubious ownership in repository".to_vec(),
                }),
            ])),
        };
        assert_eq!(
            discover_with(&git, "."),
            Err(RepositoryError::UnsafeRepository)
        );
    }

    #[test]
    fn removes_only_the_git_record_terminator() {
        let git = FakeGit {
            outputs: Mutex::new(VecDeque::from([Ok(CommandOutput {
                success: true,
                stdout: b"path-ending-in-newline\n\n".to_vec(),
                stderr: Vec::new(),
            })])),
        };
        assert_eq!(
            run_git(&git, [OsStr::new("test")]).unwrap(),
            "path-ending-in-newline\n"
        );
    }

    #[test]
    fn parses_raw_merge_commit_without_message_delimiters() {
        let oid = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let raw = concat!(
            "tree bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n",
            "parent cccccccccccccccccccccccccccccccccccccccc\n",
            "parent dddddddddddddddddddddddddddddddddddddddd\n",
            "author Alice Example <alice@example.com> 0 +0000\n",
            "committer Bob Example <bob@example.com> 3600 +0100\n",
            "gpgsig -----BEGIN SIGNATURE-----\n",
            " continuation\n",
            "\n",
            "summary\n\nbody with a delimiter-like value: ::gitnova::\n"
        );
        let commit = parse_commit(oid, raw.as_bytes()).unwrap();
        assert_eq!(commit.parents.len(), 2);
        assert_eq!(commit.author.name, "Alice Example");
        assert_eq!(commit.author.timestamp, "1970-01-01T00:00:00+00:00");
        assert_eq!(commit.committer.timestamp, "1970-01-01T02:00:00+01:00");
        assert_eq!(commit.summary, "summary");
        assert_eq!(
            commit.message,
            "summary\n\nbody with a delimiter-like value: ::gitnova::\n"
        );
    }

    #[test]
    fn validates_history_cursors_and_commit_encoding() {
        let oid = "abcdefabcdefabcdefabcdefabcdefabcdefabcd";
        let cursor = format_history_cursor(oid, 42);
        assert_eq!(parse_history_cursor(&cursor).unwrap(), (oid.to_owned(), 42));
        assert_eq!(
            parse_history_cursor("v2:bad:0"),
            Err(RepositoryError::InvalidHistoryCursor)
        );
        let raw = b"tree abcdefabcdefabcdefabcdefabcdefabcdefabcd\nauthor A <a@b> 0 +0000\ncommitter A <a@b> 0 +0000\nencoding ISO-8859-1\n\nmessage";
        assert_eq!(
            parse_commit(oid, raw),
            Err(RepositoryError::HistoryEncoding)
        );
    }

    #[test]
    fn parses_nul_delimited_changed_paths_and_renames() {
        let paths =
            parse_changed_paths(b"M\0src/lib.rs\0R100\0old name.txt\0new name.txt\0D\0gone.txt\0")
                .unwrap();
        assert_eq!(
            paths,
            [
                ChangedPath {
                    old_path: "src/lib.rs".into(),
                    new_path: "src/lib.rs".into(),
                },
                ChangedPath {
                    old_path: "old name.txt".into(),
                    new_path: "new name.txt".into(),
                },
                ChangedPath {
                    old_path: "gone.txt".into(),
                    new_path: "gone.txt".into(),
                },
            ]
        );
        assert_eq!(
            parse_changed_paths(b"R100\0old-only\0"),
            Err(RepositoryError::CommitDiffParse)
        );
        assert_eq!(
            parse_changed_paths(b"invalid\0path\0"),
            Err(RepositoryError::CommitDiffParse)
        );
    }

    #[test]
    fn parses_reference_fields_without_human_branch_output() {
        let oid = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let tag_oid = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let output = format!(
            "refs/heads/main\0{oid}\0\0\0refs/remotes/origin/main\nrefs/remotes/origin/HEAD\0{oid}\0\0refs/remotes/origin/main\0\nrefs/tags/v1\0{tag_oid}\0{oid}\0\0\n"
        );
        let references = parse_references(output.as_bytes()).unwrap();
        assert_eq!(references.len(), 3);
        assert_eq!(references[0].kind, ReferenceKind::LocalBranch);
        assert_eq!(
            references[0].upstream.as_deref(),
            Some("refs/remotes/origin/main")
        );
        assert_eq!(
            references[1].symbolic_target.as_deref(),
            Some("refs/remotes/origin/main")
        );
        assert_eq!(references[2].kind, ReferenceKind::Tag);
        assert_eq!(references[2].peeled_target_oid.as_deref(), Some(oid));
        assert_eq!(
            parse_references(b"refs/other/x\0bad\0\0\0\n"),
            Err(RepositoryError::ReferenceParse)
        );
        assert_eq!(
            parse_references(b"refs/heads/\xff\0aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\0\0\0\n"),
            Err(RepositoryError::ReferenceEncoding)
        );
    }

    #[test]
    fn parses_porcelain_v2_branch_and_all_entry_shapes() {
        let output = concat!(
            "# branch.oid abc123\0",
            "# branch.head main\0",
            "# branch.upstream origin/main\0",
            "# branch.ab +2 -3\0",
            "1 M. N... 100644 100644 100644 aaa bbb modified.txt\0",
            "2 R. N... 100644 100644 100644 aaa bbb R100 renamed.txt\0",
            "original.txt\0",
            "u UU N... 100644 100644 100644 100644 aaa bbb ccc conflict.txt\0",
            "? untracked.txt\0"
        )
        .as_bytes();
        let status = parse_status(output).unwrap();
        assert_eq!(status.branch.head.as_deref(), Some("main"));
        assert_eq!(status.branch.oid.as_deref(), Some("abc123"));
        assert_eq!(status.branch.upstream.as_deref(), Some("origin/main"));
        assert_eq!((status.branch.ahead, status.branch.behind), (2, 3));
        assert_eq!(status.entries.len(), 4);
        assert_eq!(status.entries[0].index_status, FileStatus::Modified);
        assert_eq!(status.entries[0].worktree_status, FileStatus::Unmodified);
        assert_eq!(status.entries[1].kind, StatusEntryKind::RenameOrCopy);
        assert_eq!(status.entries[1].path, "renamed.txt");
        assert_eq!(
            status.entries[1].original_path.as_deref(),
            Some("original.txt")
        );
        assert_eq!(status.entries[2].kind, StatusEntryKind::Unmerged);
        assert_eq!(status.entries[3].worktree_status, FileStatus::Untracked);
    }

    #[test]
    fn parses_initial_and_detached_branch_markers() {
        let status = parse_status(b"# branch.oid (initial)\0# branch.head (detached)\0").unwrap();
        assert_eq!(status.branch.oid, None);
        assert_eq!(status.branch.head, None);
        assert_eq!(status.branch.upstream, None);
        assert_eq!((status.branch.ahead, status.branch.behind), (0, 0));
    }

    #[test]
    fn parses_diff_hunks_and_line_numbers() {
        let patch = b"diff --git a/file.txt b/file.txt\n--- a/file.txt\n+++ b/file.txt\n@@ -1,2 +1,2 @@ section\n context\n-old\n+new\n\\ No newline at end of file\n";
        let diff = parse_diff(patch, "file.txt").unwrap();
        assert!(!diff.is_binary);
        assert_eq!(diff.hunks.len(), 1);
        let hunk = &diff.hunks[0];
        assert_eq!((hunk.old_start, hunk.old_lines), (1, 2));
        assert_eq!((hunk.new_start, hunk.new_lines), (1, 2));
        assert_eq!(hunk.header, "section");
        assert_eq!(hunk.lines.len(), 3);
        assert_eq!(
            (hunk.lines[0].old_line, hunk.lines[0].new_line),
            (Some(1), Some(1))
        );
        assert_eq!(
            (hunk.lines[1].old_line, hunk.lines[1].new_line),
            (Some(2), None)
        );
        assert_eq!(
            (hunk.lines[2].old_line, hunk.lines[2].new_line),
            (None, Some(2))
        );
    }

    #[test]
    fn detects_binary_diff_without_returning_content() {
        let diff = parse_diff(
            b"diff --git a/image.bin b/image.bin\nBinary files a/image.bin and b/image.bin differ\n",
            "image.bin",
        )
        .unwrap();
        assert!(diff.is_binary);
        assert!(diff.hunks.is_empty());
    }

    #[test]
    fn parses_rename_only_paths_without_hunks() {
        let patch = b"diff --git a/old name.txt b/new name.txt\nsimilarity index 100%\nrename from old name.txt\nrename to new name.txt\n";
        let diff = parse_diff(patch, "new name.txt").unwrap();
        assert_eq!(diff.old_path, "old name.txt");
        assert_eq!(diff.new_path, "new name.txt");
        assert!(diff.hunks.is_empty());
    }

    #[test]
    fn preserves_carriage_returns_that_belong_to_file_content() {
        let patch = b"@@ -1 +1 @@\n-old\r\n+new\r\n";
        let diff = parse_diff(patch, "crlf.txt").unwrap();
        assert_eq!(diff.hunks[0].lines[0].content, "old\r");
        assert_eq!(diff.hunks[0].lines[1].content, "new\r");
    }

    #[test]
    fn rejects_unsafe_repository_relative_paths() {
        for path in [
            "",
            "/absolute",
            "../outside",
            "a/../outside",
            ":(glob)*",
            "a//b",
        ] {
            assert_eq!(
                validate_repository_path(path),
                Err(RepositoryError::InvalidRepositoryPath)
            );
        }
        assert_eq!(validate_repository_path("src/lib.rs"), Ok(()));
    }
}
