use gitnova_protocol::{
    DiffHunk, DiffLine, DiffLineKind, GitHubPatchState, GitLabCommitFileDiff, GitLabCommitIdentity,
    GitLabFileStatus, GitLabMergeRequest, GitLabMergeRequestCommit, GitLabMergeRequestCommitDiff,
    GitLabMergeRequestCommitDiffParams, GitLabMergeRequestParams, GitLabMergeRequestRef,
    GitLabMergeRequestState, GitLabProject, GitLabProjectParams, GitLabSquashTrace,
    RepositoryDescriptor, SquashTraceClassification, SquashTraceConfidence, SquashTraceEvidence,
    SquashTraceLocalAvailability, SquashTraceRelationship,
};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::collections::HashSet;
use std::ffi::OsString;
use std::io;
use std::process::Command;

const MAX_REMOTE_OUTPUT_BYTES: usize = 16 * 1024;
const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
const MAX_COMMIT_RESPONSE_BYTES: usize = 16 * 1024 * 1024;
const MAX_DIFF_RESPONSE_BYTES: usize = 32 * 1024 * 1024;
const MAX_MERGE_REQUEST_COMMITS: usize = 1_000;
const MAX_COMMIT_FILES: usize = 3_000;

#[derive(Debug, Eq, PartialEq)]
pub enum GitLabError {
    InvalidRemote,
    RemoteNotFound,
    UnsupportedRemote,
    GlabUnavailable,
    AuthenticationRequired,
    RequestFailed,
    ResponseParse,
    MergeRequestCommitLimit,
    CommitNotInMergeRequest,
    CommitFileLimit,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SquashTraceError {
    GitLab(GitLabError),
    Repository(crate::repository::RepositoryError),
}

#[derive(Debug)]
struct CommandOutput {
    exit_code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

trait CommandRunner {
    fn run(
        &self,
        program: &str,
        arguments: &[OsString],
        environment: &[(&str, &str)],
    ) -> Result<CommandOutput, io::Error>;
}

struct SystemCommand;

impl CommandRunner for SystemCommand {
    fn run(
        &self,
        program: &str,
        arguments: &[OsString],
        environment: &[(&str, &str)],
    ) -> Result<CommandOutput, io::Error> {
        let output = Command::new(program)
            .args(arguments)
            .envs(environment.iter().copied())
            .env("LC_ALL", "C")
            .output()?;
        Ok(CommandOutput {
            exit_code: output.status.code(),
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

pub fn project(
    descriptor: &RepositoryDescriptor,
    params: &GitLabProjectParams,
) -> Result<GitLabProject, GitLabError> {
    project_with(&SystemCommand, descriptor, params)
}

pub fn merge_request(
    descriptor: &RepositoryDescriptor,
    params: &GitLabMergeRequestParams,
) -> Result<GitLabMergeRequest, GitLabError> {
    merge_request_with(&SystemCommand, descriptor, params)
}

pub fn merge_request_commit_diff(
    descriptor: &RepositoryDescriptor,
    params: &GitLabMergeRequestCommitDiffParams,
) -> Result<GitLabMergeRequestCommitDiff, GitLabError> {
    merge_request_commit_diff_with(&SystemCommand, descriptor, params)
}

pub fn squash_trace(
    descriptor: &RepositoryDescriptor,
    params: &GitLabMergeRequestParams,
) -> Result<GitLabSquashTrace, SquashTraceError> {
    squash_trace_with(&SystemCommand, descriptor, params, |oid| {
        crate::repository::commit_parents_if_available(descriptor, oid)
    })
}

fn project_with(
    runner: &impl CommandRunner,
    descriptor: &RepositoryDescriptor,
    params: &GitLabProjectParams,
) -> Result<GitLabProject, GitLabError> {
    let identity = resolve_project_identity(
        runner,
        descriptor,
        params.remote.as_deref(),
        params.path_with_namespace.as_deref(),
    )?;
    let endpoint = format!("projects/{}", encode_path(&identity.path));
    let bytes = run_glab_api(runner, &identity.host, &[endpoint], MAX_RESPONSE_BYTES)?;
    normalize_project(&bytes, &identity)
}

fn merge_request_with(
    runner: &impl CommandRunner,
    descriptor: &RepositoryDescriptor,
    params: &GitLabMergeRequestParams,
) -> Result<GitLabMergeRequest, GitLabError> {
    let project = project_with(
        runner,
        descriptor,
        &GitLabProjectParams {
            remote: params.remote.clone(),
            path_with_namespace: params.path_with_namespace.clone(),
        },
    )?;
    let encoded = encode_path(&project.path_with_namespace);
    let detail_endpoint = format!("projects/{encoded}/merge_requests/{}", params.iid);
    let detail_bytes = run_glab_api(
        runner,
        &project.host,
        &[detail_endpoint],
        MAX_RESPONSE_BYTES,
    )?;
    let detail: ApiMergeRequest =
        serde_json::from_slice(&detail_bytes).map_err(|_| GitLabError::ResponseParse)?;
    let commits_endpoint = format!(
        "projects/{encoded}/merge_requests/{}/commits?per_page=100",
        params.iid
    );
    let commit_bytes = run_glab_api(
        runner,
        &project.host,
        &[
            commits_endpoint,
            "--paginate".into(),
            "--output".into(),
            "ndjson".into(),
        ],
        MAX_COMMIT_RESPONSE_BYTES,
    )?;
    let commits: Vec<ApiCommit> = parse_paginated(&commit_bytes)?;
    if commits.len() > MAX_MERGE_REQUEST_COMMITS {
        return Err(GitLabError::MergeRequestCommitLimit);
    }
    normalize_merge_request(project, params.iid, detail, commits)
}

fn merge_request_commit_diff_with(
    runner: &impl CommandRunner,
    descriptor: &RepositoryDescriptor,
    params: &GitLabMergeRequestCommitDiffParams,
) -> Result<GitLabMergeRequestCommitDiff, GitLabError> {
    let merge_request = merge_request_with(
        runner,
        descriptor,
        &GitLabMergeRequestParams {
            iid: params.iid,
            remote: params.remote.clone(),
            path_with_namespace: params.path_with_namespace.clone(),
        },
    )?;
    let commit = merge_request
        .commits
        .iter()
        .find(|commit| commit.oid.eq_ignore_ascii_case(&params.oid))
        .cloned()
        .ok_or(GitLabError::CommitNotInMergeRequest)?;
    let endpoint = format!(
        "projects/{}/repository/commits/{}/diff?per_page=100&unidiff=true",
        encode_path(&merge_request.path_with_namespace),
        commit.oid
    );
    let bytes = run_glab_api(
        runner,
        &merge_request.host,
        &[
            endpoint,
            "--paginate".into(),
            "--output".into(),
            "ndjson".into(),
        ],
        MAX_DIFF_RESPONSE_BYTES,
    )?;
    let raw_files: Vec<ApiCommitDiff> = parse_paginated(&bytes)?;
    if raw_files.len() > MAX_COMMIT_FILES {
        return Err(GitLabError::CommitFileLimit);
    }
    let files = raw_files
        .into_iter()
        .map(normalize_file_diff)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(GitLabMergeRequestCommitDiff {
        host: merge_request.host,
        path_with_namespace: merge_request.path_with_namespace,
        merge_request_iid: params.iid,
        commit,
        files,
    })
}

fn squash_trace_with(
    runner: &impl CommandRunner,
    descriptor: &RepositoryDescriptor,
    params: &GitLabMergeRequestParams,
    inspect: impl FnOnce(&str) -> Result<Option<Vec<String>>, crate::repository::RepositoryError>,
) -> Result<GitLabSquashTrace, SquashTraceError> {
    let merge_request =
        merge_request_with(runner, descriptor, params).map_err(SquashTraceError::GitLab)?;
    let final_oid = merge_request
        .squash_commit_oid
        .as_deref()
        .or(merge_request.merge_commit_oid.as_deref());
    let local_parents = if merge_request.state == GitLabMergeRequestState::Merged
        && final_oid.is_some_and(|oid| {
            !merge_request
                .commits
                .iter()
                .any(|commit| commit.oid.eq_ignore_ascii_case(oid))
        }) {
        inspect(final_oid.expect("checked above")).map_err(SquashTraceError::Repository)?
    } else {
        None
    };
    let relationship = classify_relationship(&merge_request, local_parents);
    Ok(GitLabSquashTrace {
        merge_request,
        relationship,
    })
}

fn classify_relationship(
    merge_request: &GitLabMergeRequest,
    local_parents: Option<Vec<String>>,
) -> SquashTraceRelationship {
    if merge_request.state != GitLabMergeRequestState::Merged {
        return SquashTraceRelationship {
            classification: SquashTraceClassification::NotMerged,
            confidence: SquashTraceConfidence::High,
            merge_commit_oid: None,
            local_availability: SquashTraceLocalAvailability::NotInspected,
            local_parent_oids: Vec::new(),
            evidence: vec![SquashTraceEvidence::ProviderNotMerged],
        };
    }
    let Some(final_oid) = merge_request
        .squash_commit_oid
        .clone()
        .or_else(|| merge_request.merge_commit_oid.clone())
    else {
        return SquashTraceRelationship {
            classification: SquashTraceClassification::Unresolved,
            confidence: SquashTraceConfidence::None,
            merge_commit_oid: None,
            local_availability: SquashTraceLocalAvailability::NotInspected,
            local_parent_oids: Vec::new(),
            evidence: vec![SquashTraceEvidence::ProviderMergeOidMissing],
        };
    };
    if merge_request
        .commits
        .iter()
        .any(|commit| commit.oid.eq_ignore_ascii_case(&final_oid))
    {
        return SquashTraceRelationship {
            classification: SquashTraceClassification::OriginalCommit,
            confidence: SquashTraceConfidence::High,
            merge_commit_oid: Some(final_oid),
            local_availability: SquashTraceLocalAvailability::NotInspected,
            local_parent_oids: Vec::new(),
            evidence: vec![SquashTraceEvidence::MergeOidMatchesOriginalCommit],
        };
    }
    if merge_request.squash_commit_oid.is_some() {
        return SquashTraceRelationship {
            classification: SquashTraceClassification::SquashCandidate,
            confidence: SquashTraceConfidence::High,
            merge_commit_oid: Some(final_oid),
            local_availability: if local_parents.is_some() {
                SquashTraceLocalAvailability::Available
            } else {
                SquashTraceLocalAvailability::Missing
            },
            local_parent_oids: local_parents.unwrap_or_default(),
            evidence: vec![
                SquashTraceEvidence::MergeOidDistinctFromOriginalCommits,
                SquashTraceEvidence::ProviderSquashCommitReported,
            ],
        };
    }
    let Some(parents) = local_parents else {
        return SquashTraceRelationship {
            classification: SquashTraceClassification::Unresolved,
            confidence: SquashTraceConfidence::None,
            merge_commit_oid: Some(final_oid),
            local_availability: SquashTraceLocalAvailability::Missing,
            local_parent_oids: Vec::new(),
            evidence: vec![
                SquashTraceEvidence::MergeOidDistinctFromOriginalCommits,
                SquashTraceEvidence::LocalCommitMissing,
                SquashTraceEvidence::ProviderMergeStrategyUnavailable,
            ],
        };
    };
    let multiple = parents.len() >= 2;
    SquashTraceRelationship {
        classification: if multiple {
            SquashTraceClassification::MergeCommit
        } else {
            SquashTraceClassification::SquashCandidate
        },
        confidence: if multiple {
            SquashTraceConfidence::High
        } else {
            SquashTraceConfidence::Medium
        },
        merge_commit_oid: Some(final_oid),
        local_availability: SquashTraceLocalAvailability::Available,
        local_parent_oids: parents,
        evidence: vec![
            SquashTraceEvidence::MergeOidDistinctFromOriginalCommits,
            SquashTraceEvidence::LocalCommitAvailable,
            if multiple {
                SquashTraceEvidence::LocalCommitHasMultipleParents
            } else {
                SquashTraceEvidence::LocalCommitHasAtMostOneParent
            },
            SquashTraceEvidence::ProviderMergeStrategyUnavailable,
        ],
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProjectIdentity {
    host: String,
    path: String,
}

fn resolve_project_identity(
    runner: &impl CommandRunner,
    descriptor: &RepositoryDescriptor,
    remote: Option<&str>,
    explicit_path: Option<&str>,
) -> Result<ProjectIdentity, GitLabError> {
    let remote = remote.unwrap_or("origin");
    if !valid_remote_name(remote) {
        return Err(GitLabError::InvalidRemote);
    }
    let base = descriptor
        .worktree_root
        .as_ref()
        .unwrap_or(&descriptor.git_directory);
    let output = runner
        .run(
            "git",
            &[
                "-C".into(),
                base.into(),
                "config".into(),
                "-z".into(),
                "--get-all".into(),
                format!("remote.{remote}.url").into(),
            ],
            &[("GIT_OPTIONAL_LOCKS", "0")],
        )
        .map_err(|_| GitLabError::RemoteNotFound)?;
    if output.exit_code != Some(0) {
        return Err(GitLabError::RemoteNotFound);
    }
    if output.stdout.len() > MAX_REMOTE_OUTPUT_BYTES {
        return Err(GitLabError::UnsupportedRemote);
    }
    let url = output
        .stdout
        .split(|byte| *byte == 0)
        .find(|value| !value.is_empty())
        .ok_or(GitLabError::RemoteNotFound)?;
    let url = std::str::from_utf8(url).map_err(|_| GitLabError::UnsupportedRemote)?;
    let mut identity = parse_gitlab_url_with(runner, url)?;
    if let Some(path) = explicit_path {
        validate_project_path(path)?;
        identity.path = path.to_owned();
    }
    Ok(identity)
}

fn parse_gitlab_url(value: &str) -> Result<ProjectIdentity, GitLabError> {
    let (host, path) = if let Some(rest) = value.strip_prefix("https://") {
        rest.split_once('/').ok_or(GitLabError::UnsupportedRemote)?
    } else if let Some(rest) = value.strip_prefix("ssh://git@") {
        rest.split_once('/').ok_or(GitLabError::UnsupportedRemote)?
    } else if let Some(rest) = value.strip_prefix("git@") {
        rest.split_once(':').ok_or(GitLabError::UnsupportedRemote)?
    } else {
        return Err(GitLabError::UnsupportedRemote);
    };
    if !valid_host(host) || path.contains(['?', '#']) {
        return Err(GitLabError::UnsupportedRemote);
    }
    let path = path.strip_suffix(".git").unwrap_or(path);
    validate_project_path(path)?;
    Ok(ProjectIdentity {
        host: host.to_ascii_lowercase(),
        path: path.to_owned(),
    })
}

fn parse_gitlab_url_with(
    runner: &impl CommandRunner,
    value: &str,
) -> Result<ProjectIdentity, GitLabError> {
    let direct = parse_gitlab_url(value);
    if let Ok(identity) = &direct {
        let is_ssh = value.starts_with("git@") || value.starts_with("ssh://git@");
        if !is_ssh || identity.host.contains('.') {
            return Ok(identity.clone());
        }
    }
    let (alias, path) =
        crate::provider_remote::ssh_alias_and_path(value).ok_or(GitLabError::UnsupportedRemote)?;
    let output = runner
        .run(
            "ssh",
            &["-G".into(), "--".into(), alias.into()],
            &[("SSH_ASKPASS_REQUIRE", "never")],
        )
        .map_err(|_| GitLabError::UnsupportedRemote)?;
    if output.exit_code != Some(0) {
        return Err(GitLabError::UnsupportedRemote);
    }
    let host = crate::provider_remote::configured_hostname(&output.stdout)
        .ok_or(GitLabError::UnsupportedRemote)?;
    let path = path.strip_suffix(".git").unwrap_or(path);
    validate_project_path(path)?;
    Ok(ProjectIdentity {
        host: host.to_ascii_lowercase(),
        path: path.to_owned(),
    })
}

fn valid_host(value: &str) -> bool {
    crate::provider_remote::valid_hostname(value)
}

fn validate_project_path(value: &str) -> Result<(), GitLabError> {
    if value.len() > 512
        || value.starts_with('/')
        || value.ends_with('/')
        || value.split('/').count() < 2
        || value.split('/').any(|part| {
            part.is_empty()
                || part == "."
                || part == ".."
                || part.starts_with('-')
                || !part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        })
    {
        return Err(GitLabError::UnsupportedRemote);
    }
    Ok(())
}

fn valid_remote_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 255
        && !value.starts_with('-')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'/' | b'-'))
}

fn encode_path(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn run_glab_api(
    runner: &impl CommandRunner,
    hostname: &str,
    api_arguments: &[String],
    maximum_bytes: usize,
) -> Result<Vec<u8>, GitLabError> {
    if !valid_host(hostname) {
        return Err(GitLabError::UnsupportedRemote);
    }
    let mut arguments = vec![OsString::from("api")];
    arguments.extend(api_arguments.iter().map(OsString::from));
    arguments.extend([OsString::from("--hostname"), OsString::from(hostname)]);
    let output = runner
        .run(
            "glab",
            &arguments,
            &[
                ("GLAB_NO_PROMPT", "1"),
                ("NO_PROMPT", "1"),
                ("GLAMOUR_STYLE", "notty"),
                ("NO_COLOR", "1"),
            ],
        )
        .map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                GitLabError::GlabUnavailable
            } else {
                GitLabError::RequestFailed
            }
        })?;
    match output.exit_code {
        Some(0) if output.stdout.len() <= maximum_bytes => Ok(output.stdout),
        Some(0) => Err(GitLabError::ResponseParse),
        _ if looks_like_auth_error(&output.stderr) => Err(GitLabError::AuthenticationRequired),
        _ => Err(GitLabError::RequestFailed),
    }
}

fn looks_like_auth_error(stderr: &[u8]) -> bool {
    if stderr.len() > 64 * 1024 {
        return false;
    }
    let text = String::from_utf8_lossy(stderr).to_ascii_lowercase();
    text.contains("auth") || text.contains("token") || text.contains("401")
}

fn parse_paginated<T: DeserializeOwned>(bytes: &[u8]) -> Result<Vec<T>, GitLabError> {
    if let Ok(items) = serde_json::from_slice::<Vec<T>>(bytes) {
        return Ok(items);
    }
    let mut items = Vec::new();
    for line in bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
    {
        if let Ok(mut page) = serde_json::from_slice::<Vec<T>>(line) {
            items.append(&mut page);
        } else {
            items.push(serde_json::from_slice::<T>(line).map_err(|_| GitLabError::ResponseParse)?);
        }
    }
    if items.is_empty() && !bytes.iter().all(u8::is_ascii_whitespace) {
        return Err(GitLabError::ResponseParse);
    }
    Ok(items)
}

#[derive(Deserialize)]
struct ApiProject {
    name: String,
    path_with_namespace: String,
    web_url: String,
    default_branch: Option<String>,
    visibility: String,
}

fn normalize_project(
    bytes: &[u8],
    expected: &ProjectIdentity,
) -> Result<GitLabProject, GitLabError> {
    let project: ApiProject =
        serde_json::from_slice(bytes).map_err(|_| GitLabError::ResponseParse)?;
    validate_project_path(&project.path_with_namespace).map_err(|_| GitLabError::ResponseParse)?;
    if project.path_with_namespace != expected.path
        || project.default_branch.as_deref().is_none_or(str::is_empty)
    {
        return Err(GitLabError::ResponseParse);
    }
    let (namespace, name) = project
        .path_with_namespace
        .rsplit_once('/')
        .ok_or(GitLabError::ResponseParse)?;
    if name != project.name
        || project.web_url != format!("https://{}/{}", expected.host, expected.path)
    {
        return Err(GitLabError::ResponseParse);
    }
    Ok(GitLabProject {
        host: expected.host.clone(),
        namespace: namespace.into(),
        name: name.into(),
        path_with_namespace: project.path_with_namespace,
        url: project.web_url,
        default_branch: project.default_branch.expect("validated above"),
        visibility: project.visibility,
    })
}

#[derive(Deserialize)]
struct ApiUser {
    username: String,
}

#[derive(Deserialize)]
struct ApiDiffRefs {
    base_sha: String,
    head_sha: String,
}

#[derive(Deserialize)]
struct ApiMergeRequest {
    iid: u64,
    title: String,
    description: Option<String>,
    state: String,
    #[serde(default)]
    draft: bool,
    author: Option<ApiUser>,
    web_url: String,
    created_at: String,
    updated_at: String,
    closed_at: Option<String>,
    merged_at: Option<String>,
    target_branch: String,
    source_branch: String,
    sha: String,
    diff_refs: Option<ApiDiffRefs>,
    merge_commit_sha: Option<String>,
    squash_commit_sha: Option<String>,
    #[serde(default)]
    squash_on_merge: bool,
}

#[derive(Deserialize)]
struct ApiCommit {
    id: String,
    #[serde(default)]
    parent_ids: Vec<String>,
    title: String,
    message: String,
    author_name: String,
    author_email: String,
    authored_date: String,
    committer_name: String,
    committer_email: String,
    committed_date: String,
    web_url: String,
}

fn normalize_merge_request(
    project: GitLabProject,
    expected_iid: u64,
    detail: ApiMergeRequest,
    commits: Vec<ApiCommit>,
) -> Result<GitLabMergeRequest, GitLabError> {
    if detail.iid != expected_iid
        || detail.title.is_empty()
        || detail.target_branch.is_empty()
        || detail.source_branch.is_empty()
        || detail.sha.is_empty()
        || detail.web_url != format!("{}/-/merge_requests/{expected_iid}", project.url)
    {
        return Err(GitLabError::ResponseParse);
    }
    let state = match detail.state.as_str() {
        "opened" => GitLabMergeRequestState::Open,
        "closed" | "locked" => GitLabMergeRequestState::Closed,
        "merged" => GitLabMergeRequestState::Merged,
        _ => return Err(GitLabError::ResponseParse),
    };
    let base_oid = detail
        .diff_refs
        .as_ref()
        .map(|refs| refs.base_sha.clone())
        .unwrap_or_default();
    let head_oid = detail
        .diff_refs
        .as_ref()
        .map(|refs| refs.head_sha.clone())
        .unwrap_or(detail.sha);
    let commits = commits
        .into_iter()
        .map(normalize_commit)
        .collect::<Result<Vec<_>, _>>()?;
    let mut seen = HashSet::new();
    if commits
        .iter()
        .any(|commit| !seen.insert(commit.oid.clone()))
    {
        return Err(GitLabError::ResponseParse);
    }
    Ok(GitLabMergeRequest {
        host: project.host,
        path_with_namespace: project.path_with_namespace.clone(),
        iid: expected_iid,
        title: detail.title,
        description: detail.description,
        state,
        is_draft: detail.draft,
        author_username: detail.author.map(|author| author.username),
        url: detail.web_url,
        created_at: detail.created_at,
        updated_at: detail.updated_at,
        closed_at: detail.closed_at,
        merged_at: detail.merged_at,
        target: GitLabMergeRequestRef {
            name: detail.target_branch,
            oid: base_oid,
            project: Some(project.path_with_namespace),
        },
        source: GitLabMergeRequestRef {
            name: detail.source_branch,
            oid: head_oid,
            project: None,
        },
        merge_commit_oid: detail.merge_commit_sha,
        squash_commit_oid: detail.squash_commit_sha,
        squash_on_merge: detail.squash_on_merge,
        commits,
    })
}

fn normalize_commit(commit: ApiCommit) -> Result<GitLabMergeRequestCommit, GitLabError> {
    if !valid_full_oid(&commit.id)
        || commit.title.is_empty()
        || commit.message.is_empty()
        || commit.author_name.is_empty()
        || commit.committer_name.is_empty()
        || commit.parent_ids.iter().any(|oid| !valid_full_oid(oid))
    {
        return Err(GitLabError::ResponseParse);
    }
    Ok(GitLabMergeRequestCommit {
        oid: commit.id,
        parents: commit.parent_ids,
        author: GitLabCommitIdentity {
            name: commit.author_name,
            email: commit.author_email,
            timestamp: commit.authored_date,
            username: None,
        },
        committer: GitLabCommitIdentity {
            name: commit.committer_name,
            email: commit.committer_email,
            timestamp: commit.committed_date,
            username: None,
        },
        summary: commit.title,
        message: commit.message,
        url: commit.web_url,
    })
}

#[derive(Deserialize)]
struct ApiCommitDiff {
    old_path: String,
    new_path: String,
    diff: String,
    #[serde(default)]
    new_file: bool,
    #[serde(default)]
    renamed_file: bool,
    #[serde(default)]
    deleted_file: bool,
    #[serde(default)]
    collapsed: bool,
    #[serde(default)]
    too_large: bool,
}

fn normalize_file_diff(file: ApiCommitDiff) -> Result<GitLabCommitFileDiff, GitLabError> {
    if file.old_path.is_empty() || file.new_path.is_empty() {
        return Err(GitLabError::ResponseParse);
    }
    let status = if file.new_file {
        GitLabFileStatus::Added
    } else if file.deleted_file {
        GitLabFileStatus::Removed
    } else if file.renamed_file {
        GitLabFileStatus::Renamed
    } else {
        GitLabFileStatus::Modified
    };
    let unavailable = file.collapsed || file.too_large;
    let hunks = if unavailable {
        Vec::new()
    } else {
        parse_patch(&file.diff)?
    };
    let additions = hunks
        .iter()
        .flat_map(|hunk| &hunk.lines)
        .filter(|line| line.kind == DiffLineKind::Addition)
        .count() as u64;
    let deletions = hunks
        .iter()
        .flat_map(|hunk| &hunk.lines)
        .filter(|line| line.kind == DiffLineKind::Deletion)
        .count() as u64;
    Ok(GitLabCommitFileDiff {
        old_path: file.old_path,
        new_path: file.new_path,
        status,
        additions,
        deletions,
        changes: additions + deletions,
        patch_state: if unavailable {
            GitHubPatchState::Unavailable
        } else {
            GitHubPatchState::Available
        },
        hunks,
    })
}

fn parse_patch(patch: &str) -> Result<Vec<DiffHunk>, GitLabError> {
    if patch.is_empty() {
        return Ok(Vec::new());
    }
    let mut hunks: Vec<DiffHunk> = Vec::new();
    let mut old_line = 0;
    let mut new_line = 0;
    for line in patch.lines() {
        if line.starts_with("@@ ") {
            let end = line.find(" @@").ok_or(GitLabError::ResponseParse)?;
            let ranges = &line[3..end];
            let mut parts = ranges.split_whitespace();
            let (old_start, old_lines) = parse_patch_range(parts.next(), '-')?;
            let (new_start, new_lines) = parse_patch_range(parts.next(), '+')?;
            if parts.next().is_some() {
                return Err(GitLabError::ResponseParse);
            }
            old_line = old_start;
            new_line = new_start;
            hunks.push(DiffHunk {
                old_start,
                old_lines,
                new_start,
                new_lines,
                header: line.to_owned(),
                lines: Vec::new(),
            });
            continue;
        }
        if line == "\\ No newline at end of file" {
            continue;
        }
        let hunk = hunks.last_mut().ok_or(GitLabError::ResponseParse)?;
        let (kind, old, new, content) = match line.as_bytes().first() {
            Some(b' ') => {
                let current_old = old_line;
                let current_new = new_line;
                old_line += 1;
                new_line += 1;
                (
                    DiffLineKind::Context,
                    Some(current_old),
                    Some(current_new),
                    &line[1..],
                )
            }
            Some(b'-') => {
                let current = old_line;
                old_line += 1;
                (DiffLineKind::Deletion, Some(current), None, &line[1..])
            }
            Some(b'+') => {
                let current = new_line;
                new_line += 1;
                (DiffLineKind::Addition, None, Some(current), &line[1..])
            }
            _ => return Err(GitLabError::ResponseParse),
        };
        hunk.lines.push(DiffLine {
            kind,
            content: content.to_owned(),
            old_line: old,
            new_line: new,
        });
    }
    Ok(hunks)
}

fn parse_patch_range(value: Option<&str>, prefix: char) -> Result<(u64, u64), GitLabError> {
    let value = value
        .and_then(|value| value.strip_prefix(prefix))
        .ok_or(GitLabError::ResponseParse)?;
    let mut parts = value.split(',');
    let start = parts
        .next()
        .ok_or(GitLabError::ResponseParse)?
        .parse()
        .map_err(|_| GitLabError::ResponseParse)?;
    let count = parts
        .next()
        .map(str::parse)
        .transpose()
        .map_err(|_| GitLabError::ResponseParse)?
        .unwrap_or(1);
    if parts.next().is_some() {
        return Err(GitLabError::ResponseParse);
    }
    Ok((start, count))
}

fn valid_full_oid(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    struct FakeRunner {
        outputs: Mutex<VecDeque<Result<CommandOutput, io::Error>>>,
        calls: Mutex<Vec<(String, Vec<OsString>)>>,
    }

    impl FakeRunner {
        fn new(outputs: Vec<Result<CommandOutput, io::Error>>) -> Self {
            Self {
                outputs: Mutex::new(outputs.into()),
                calls: Mutex::new(Vec::new()),
            }
        }
    }

    impl CommandRunner for FakeRunner {
        fn run(
            &self,
            program: &str,
            arguments: &[OsString],
            _environment: &[(&str, &str)],
        ) -> Result<CommandOutput, io::Error> {
            self.calls
                .lock()
                .unwrap()
                .push((program.into(), arguments.to_vec()));
            self.outputs.lock().unwrap().pop_front().unwrap()
        }
    }

    fn output(stdout: &str) -> Result<CommandOutput, io::Error> {
        Ok(CommandOutput {
            exit_code: Some(0),
            stdout: stdout.as_bytes().to_vec(),
            stderr: Vec::new(),
        })
    }

    fn descriptor() -> RepositoryDescriptor {
        RepositoryDescriptor {
            worktree_root: Some("/repo".into()),
            git_directory: "/repo/.git".into(),
            common_git_directory: "/repo/.git".into(),
            kind: gitnova_protocol::RepositoryKind::Worktree,
            git_version: "2.50.0".into(),
        }
    }

    fn oid(value: char) -> String {
        std::iter::repeat_n(value, 40).collect()
    }

    #[test]
    fn parses_gitlab_and_self_managed_remote_without_argument_injection() {
        assert_eq!(
            parse_gitlab_url("git@gitlab.example.com:team/sub/project.git").unwrap(),
            ProjectIdentity {
                host: "gitlab.example.com".into(),
                path: "team/sub/project".into()
            }
        );
        assert_eq!(
            parse_gitlab_url("https://gitlab.com/team/project.git")
                .unwrap()
                .path,
            "team/project"
        );
        assert_eq!(
            parse_gitlab_url("git@gitlab.com:--hostname/evil.git"),
            Err(GitLabError::UnsupportedRemote)
        );
    }

    #[test]
    fn normalizes_project_and_uses_explicit_hostname() {
        let runner = FakeRunner::new(vec![
            output("git@gitlab.example.com:team/sub/project.git\0"),
            output(
                r#"{"name":"project","path_with_namespace":"team/sub/project","web_url":"https://gitlab.example.com/team/sub/project","default_branch":"main","visibility":"private"}"#,
            ),
        ]);
        let result = project_with(&runner, &descriptor(), &GitLabProjectParams::default()).unwrap();
        assert_eq!(result.namespace, "team/sub");
        assert_eq!(result.host, "gitlab.example.com");
        let calls = runner.calls.lock().unwrap();
        assert!(
            calls[1]
                .1
                .windows(2)
                .any(|pair| pair == ["--hostname", "gitlab.example.com"])
        );
    }

    #[test]
    fn resolves_a_safe_ssh_alias_to_the_gitlab_api_hostname() {
        let runner = FakeRunner::new(vec![
            output("git@gitlab-work:team/project.git\0"),
            output("host gitlab-work\nhostname gitlab.example.com\nuser git\n"),
            output(
                r#"{"name":"project","path_with_namespace":"team/project","web_url":"https://gitlab.example.com/team/project","default_branch":"main","visibility":"private"}"#,
            ),
        ]);
        let result = project_with(&runner, &descriptor(), &GitLabProjectParams::default()).unwrap();
        assert_eq!(result.host, "gitlab.example.com");
        let calls = runner.calls.lock().unwrap();
        assert_eq!(calls[1].0, "ssh");
        assert_eq!(calls[1].1, ["-G", "--", "gitlab-work"]);
        assert!(
            calls[2]
                .1
                .windows(2)
                .any(|pair| pair == ["--hostname", "gitlab.example.com"])
        );
    }

    #[test]
    fn returns_original_commits_diff_and_provider_confirmed_squash() {
        let first = oid('a');
        let squash = oid('b');
        let project_json = r#"{"name":"project","path_with_namespace":"team/project","web_url":"https://gitlab.com/team/project","default_branch":"main","visibility":"public"}"#;
        let detail = format!(
            r#"{{"iid":7,"title":"Ship","description":"body","state":"merged","draft":false,"author":{{"username":"alice"}},"web_url":"https://gitlab.com/team/project/-/merge_requests/7","created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-02T00:00:00Z","closed_at":"2026-01-02T00:00:00Z","merged_at":"2026-01-02T00:00:00Z","target_branch":"main","source_branch":"feature","sha":"{first}","diff_refs":{{"base_sha":"{squash}","head_sha":"{first}"}},"merge_commit_sha":null,"squash_commit_sha":"{squash}","squash_on_merge":true}}"#
        );
        let commits = format!(
            r#"{{"id":"{first}","parent_ids":[],"title":"one","message":"one\n","author_name":"Alice","author_email":"a@example.com","authored_date":"2026-01-01T00:00:00Z","committer_name":"Alice","committer_email":"a@example.com","committed_date":"2026-01-01T00:00:00Z","web_url":"https://gitlab.com/team/project/-/commit/{first}"}}"#
        );
        let diff = r#"{"old_path":"a.txt","new_path":"a.txt","diff":"@@ -1 +1 @@\n-old\n+new","new_file":false,"renamed_file":false,"deleted_file":false,"collapsed":false,"too_large":false}"#;
        let runner = FakeRunner::new(vec![
            output("git@gitlab.com:team/project.git\0"),
            output(project_json),
            output(&detail),
            output(&commits),
            output(diff),
        ]);
        let result = merge_request_commit_diff_with(
            &runner,
            &descriptor(),
            &GitLabMergeRequestCommitDiffParams {
                iid: 7,
                oid: first.clone(),
                remote: None,
                path_with_namespace: None,
            },
        )
        .unwrap();
        assert_eq!(result.files[0].additions, 1);
        assert_eq!(result.files[0].deletions, 1);

        let runner = FakeRunner::new(vec![
            output("git@gitlab.com:team/project.git\0"),
            output(project_json),
            output(&detail),
            output(&commits),
        ]);
        let trace = squash_trace_with(
            &runner,
            &descriptor(),
            &GitLabMergeRequestParams {
                iid: 7,
                remote: None,
                path_with_namespace: None,
            },
            |_| Ok(Some(vec![oid('c')])),
        )
        .unwrap();
        assert_eq!(
            trace.relationship.classification,
            SquashTraceClassification::SquashCandidate
        );
        assert_eq!(trace.relationship.confidence, SquashTraceConfidence::High);
        assert!(
            trace
                .relationship
                .evidence
                .contains(&SquashTraceEvidence::ProviderSquashCommitReported)
        );
    }

    #[test]
    fn rejects_non_member_before_requesting_commit_diff() {
        let first = oid('a');
        let project_json = r#"{"name":"project","path_with_namespace":"team/project","web_url":"https://gitlab.com/team/project","default_branch":"main","visibility":"public"}"#;
        let detail = format!(
            r#"{{"iid":7,"title":"Ship","description":null,"state":"opened","draft":false,"author":null,"web_url":"https://gitlab.com/team/project/-/merge_requests/7","created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z","closed_at":null,"merged_at":null,"target_branch":"main","source_branch":"feature","sha":"{first}","diff_refs":null,"merge_commit_sha":null,"squash_commit_sha":null,"squash_on_merge":false}}"#
        );
        let runner = FakeRunner::new(vec![
            output("git@gitlab.com:team/project.git\0"),
            output(project_json),
            output(&detail),
            output("[]"),
        ]);
        let result = merge_request_commit_diff_with(
            &runner,
            &descriptor(),
            &GitLabMergeRequestCommitDiffParams {
                iid: 7,
                oid: oid('f'),
                remote: None,
                path_with_namespace: None,
            },
        );
        assert_eq!(result, Err(GitLabError::CommitNotInMergeRequest));
        assert_eq!(runner.calls.lock().unwrap().len(), 4);
    }
}
