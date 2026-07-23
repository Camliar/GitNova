use gitnova_protocol::{
    GitHubCommitIdentity, GitHubPullRequest, GitHubPullRequestCommit, GitHubPullRequestParams,
    GitHubPullRequestRef, GitHubPullRequestState, GitHubRepository, GitHubRepositoryParams,
    RepositoryDescriptor,
};
use serde::Deserialize;
use std::ffi::OsString;
use std::io;
use std::process::Command;

const MAX_REMOTE_OUTPUT_BYTES: usize = 16 * 1024;
const MAX_GITHUB_RESPONSE_BYTES: usize = 1024 * 1024;
const MAX_GITHUB_COMMIT_RESPONSE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Eq, PartialEq)]
pub enum GitHubError {
    InvalidRemote,
    RemoteNotFound,
    UnsupportedRemote,
    GhUnavailable,
    AuthenticationRequired,
    RequestFailed,
    ResponseParse,
    PullRequestCommitLimit,
}

#[derive(Debug)]
struct CommandOutput {
    exit_code: Option<i32>,
    stdout: Vec<u8>,
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
        })
    }
}

pub fn repository(
    descriptor: &RepositoryDescriptor,
    params: &GitHubRepositoryParams,
) -> Result<GitHubRepository, GitHubError> {
    repository_with(&SystemCommand, descriptor, params)
}

pub fn pull_request(
    descriptor: &RepositoryDescriptor,
    params: &GitHubPullRequestParams,
) -> Result<GitHubPullRequest, GitHubError> {
    pull_request_with(&SystemCommand, descriptor, params)
}

fn repository_with(
    runner: &impl CommandRunner,
    descriptor: &RepositoryDescriptor,
    params: &GitHubRepositoryParams,
) -> Result<GitHubRepository, GitHubError> {
    let (owner, name) = resolve_repository_identity(
        runner,
        descriptor,
        params.remote.as_deref(),
        params.name_with_owner.as_deref(),
    )?;

    let endpoint = format!("repos/{owner}/{name}");
    let bytes = run_gh_api(runner, &[endpoint], MAX_GITHUB_RESPONSE_BYTES)?;
    parse_response(&bytes)
}

fn resolve_repository_identity(
    runner: &impl CommandRunner,
    descriptor: &RepositoryDescriptor,
    remote: Option<&str>,
    name_with_owner: Option<&str>,
) -> Result<(String, String), GitHubError> {
    if let Some(name_with_owner) = name_with_owner {
        parse_name_with_owner(name_with_owner)
    } else {
        let remote = remote.unwrap_or("origin");
        if !valid_remote_name(remote) {
            return Err(GitHubError::InvalidRemote);
        }
        let base = descriptor
            .worktree_root
            .as_ref()
            .unwrap_or(&descriptor.git_directory);
        let arguments = [
            OsString::from("-C"),
            OsString::from(base),
            OsString::from("config"),
            OsString::from("-z"),
            OsString::from("--get-all"),
            OsString::from(format!("remote.{remote}.url")),
        ];
        let output = runner
            .run("git", &arguments, &[("GIT_OPTIONAL_LOCKS", "0")])
            .map_err(|_| GitHubError::RemoteNotFound)?;
        if output.exit_code != Some(0) {
            return Err(GitHubError::RemoteNotFound);
        }
        if output.stdout.len() > MAX_REMOTE_OUTPUT_BYTES {
            return Err(GitHubError::UnsupportedRemote);
        }
        let url = output
            .stdout
            .split(|byte| *byte == 0)
            .find(|value| !value.is_empty())
            .ok_or(GitHubError::RemoteNotFound)?;
        let url = std::str::from_utf8(url).map_err(|_| GitHubError::UnsupportedRemote)?;
        parse_github_url(url)
    }
}

fn run_gh_api(
    runner: &impl CommandRunner,
    api_arguments: &[String],
    maximum_bytes: usize,
) -> Result<Vec<u8>, GitHubError> {
    let mut arguments = vec![OsString::from("api")];
    arguments.extend(api_arguments.iter().map(OsString::from));
    arguments.extend([OsString::from("--hostname"), OsString::from("github.com")]);
    let output = runner
        .run(
            "gh",
            &arguments,
            &[
                ("GH_PROMPT_DISABLED", "1"),
                ("GH_PAGER", "cat"),
                ("NO_COLOR", "1"),
            ],
        )
        .map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                GitHubError::GhUnavailable
            } else {
                GitHubError::RequestFailed
            }
        })?;
    match output.exit_code {
        Some(0) if output.stdout.len() <= maximum_bytes => Ok(output.stdout),
        Some(0) => Err(GitHubError::ResponseParse),
        Some(4) => Err(GitHubError::AuthenticationRequired),
        _ => Err(GitHubError::RequestFailed),
    }
}

fn valid_remote_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 255
        && !value.starts_with('-')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'/' | b'-'))
}

fn parse_github_url(value: &str) -> Result<(String, String), GitHubError> {
    let path = value
        .strip_prefix("https://github.com/")
        .or_else(|| value.strip_prefix("ssh://git@github.com/"))
        .or_else(|| value.strip_prefix("git@github.com:"))
        .ok_or(GitHubError::UnsupportedRemote)?;
    if path.contains(['?', '#']) {
        return Err(GitHubError::UnsupportedRemote);
    }
    let path = path.strip_suffix(".git").unwrap_or(path);
    parse_name_with_owner(path).map_err(|_| GitHubError::UnsupportedRemote)
}

fn parse_name_with_owner(value: &str) -> Result<(String, String), GitHubError> {
    let mut parts = value.split('/');
    let owner = parts.next().ok_or(GitHubError::UnsupportedRemote)?;
    let name = parts.next().ok_or(GitHubError::UnsupportedRemote)?;
    if parts.next().is_some() || !valid_owner(owner) || !valid_repository_name(name) {
        return Err(GitHubError::UnsupportedRemote);
    }
    Ok((owner.to_owned(), name.to_owned()))
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 39
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn valid_repository_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 100
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

#[derive(Deserialize)]
struct ApiRepository {
    name: String,
    full_name: String,
    owner: ApiOwner,
    html_url: String,
    default_branch: String,
    private: bool,
}

#[derive(Deserialize)]
struct ApiOwner {
    login: String,
}

fn parse_response(bytes: &[u8]) -> Result<GitHubRepository, GitHubError> {
    if bytes.len() > MAX_GITHUB_RESPONSE_BYTES {
        return Err(GitHubError::ResponseParse);
    }
    let response: ApiRepository =
        serde_json::from_slice(bytes).map_err(|_| GitHubError::ResponseParse)?;
    let (full_owner, full_name) =
        parse_name_with_owner(&response.full_name).map_err(|_| GitHubError::ResponseParse)?;
    let expected_url = format!("https://github.com/{}", response.full_name);
    if response.owner.login != full_owner
        || response.name != full_name
        || response.default_branch.is_empty()
        || response.html_url != expected_url
    {
        return Err(GitHubError::ResponseParse);
    }
    Ok(GitHubRepository {
        host: "github.com".into(),
        owner: response.owner.login,
        name: response.name,
        name_with_owner: response.full_name,
        url: response.html_url,
        default_branch: response.default_branch,
        is_private: response.private,
    })
}

fn pull_request_with(
    runner: &impl CommandRunner,
    descriptor: &RepositoryDescriptor,
    params: &GitHubPullRequestParams,
) -> Result<GitHubPullRequest, GitHubError> {
    let (owner, name) = resolve_repository_identity(
        runner,
        descriptor,
        params.remote.as_deref(),
        params.name_with_owner.as_deref(),
    )?;
    let name_with_owner = format!("{owner}/{name}");
    let detail_endpoint = format!("repos/{owner}/{name}/pulls/{}", params.number);
    let detail_bytes = run_gh_api(runner, &[detail_endpoint], MAX_GITHUB_RESPONSE_BYTES)?;
    let detail: ApiPullRequest =
        serde_json::from_slice(&detail_bytes).map_err(|_| GitHubError::ResponseParse)?;
    if detail.commits > 250 {
        return Err(GitHubError::PullRequestCommitLimit);
    }

    let commits_endpoint = format!(
        "repos/{owner}/{name}/pulls/{}/commits?per_page=100",
        params.number
    );
    let commit_bytes = run_gh_api(
        runner,
        &[commits_endpoint, "--paginate".into(), "--slurp".into()],
        MAX_GITHUB_COMMIT_RESPONSE_BYTES,
    )?;
    let pages: Vec<Vec<ApiPullRequestCommit>> =
        serde_json::from_slice(&commit_bytes).map_err(|_| GitHubError::ResponseParse)?;
    let api_commits: Vec<ApiPullRequestCommit> = pages.into_iter().flatten().collect();
    if api_commits.len() != detail.commits {
        return Err(if detail.commits >= 250 {
            GitHubError::PullRequestCommitLimit
        } else {
            GitHubError::ResponseParse
        });
    }
    normalize_pull_request(detail, params.number, &name_with_owner, api_commits)
}

#[derive(Deserialize)]
struct ApiPullRequest {
    number: u64,
    title: String,
    body: Option<String>,
    state: String,
    draft: bool,
    merged: bool,
    user: Option<ApiLogin>,
    html_url: String,
    created_at: String,
    updated_at: String,
    closed_at: Option<String>,
    merged_at: Option<String>,
    base: ApiPullRequestRef,
    head: ApiPullRequestRef,
    merge_commit_sha: Option<String>,
    commits: usize,
}

#[derive(Deserialize)]
struct ApiLogin {
    login: String,
}

#[derive(Deserialize)]
struct ApiPullRequestRef {
    #[serde(rename = "ref")]
    name: String,
    sha: String,
    repo: Option<ApiRepositoryName>,
}

#[derive(Deserialize)]
struct ApiRepositoryName {
    full_name: String,
}

#[derive(Deserialize)]
struct ApiPullRequestCommit {
    sha: String,
    html_url: String,
    commit: ApiGitCommit,
    author: Option<ApiLogin>,
    committer: Option<ApiLogin>,
    parents: Vec<ApiParent>,
}

#[derive(Deserialize)]
struct ApiGitCommit {
    author: Option<ApiGitIdentity>,
    committer: Option<ApiGitIdentity>,
    message: String,
}

#[derive(Deserialize)]
struct ApiGitIdentity {
    name: String,
    email: String,
    date: String,
}

#[derive(Deserialize)]
struct ApiParent {
    sha: String,
}

fn normalize_pull_request(
    detail: ApiPullRequest,
    requested_number: u64,
    name_with_owner: &str,
    commits: Vec<ApiPullRequestCommit>,
) -> Result<GitHubPullRequest, GitHubError> {
    let canonical_name_with_owner = detail
        .base
        .repo
        .as_ref()
        .ok_or(GitHubError::ResponseParse)?
        .full_name
        .clone();
    parse_name_with_owner(&canonical_name_with_owner).map_err(|_| GitHubError::ResponseParse)?;
    if detail.number != requested_number
        || !canonical_name_with_owner.eq_ignore_ascii_case(name_with_owner)
        || detail.title.is_empty()
        || detail.created_at.is_empty()
        || detail.updated_at.is_empty()
        || detail.html_url
            != format!("https://github.com/{canonical_name_with_owner}/pull/{requested_number}")
        || detail.merged != detail.merged_at.is_some()
    {
        return Err(GitHubError::ResponseParse);
    }
    let state = if detail.merged {
        GitHubPullRequestState::Merged
    } else {
        match detail.state.as_str() {
            "open" => GitHubPullRequestState::Open,
            "closed" => GitHubPullRequestState::Closed,
            _ => return Err(GitHubError::ResponseParse),
        }
    };
    if let Some(oid) = &detail.merge_commit_sha
        && !valid_oid(oid)
    {
        return Err(GitHubError::ResponseParse);
    }
    let commits = commits
        .into_iter()
        .map(|commit| normalize_commit(commit, &canonical_name_with_owner))
        .collect::<Result<_, _>>()?;
    Ok(GitHubPullRequest {
        host: "github.com".into(),
        name_with_owner: canonical_name_with_owner,
        number: detail.number,
        title: detail.title,
        body: detail.body,
        state,
        is_draft: detail.draft,
        author_login: detail.user.map(|user| user.login),
        url: detail.html_url,
        created_at: detail.created_at,
        updated_at: detail.updated_at,
        closed_at: detail.closed_at,
        merged_at: detail.merged_at,
        base: normalize_pull_request_ref(detail.base)?,
        head: normalize_pull_request_ref(detail.head)?,
        merge_commit_oid: detail.merge_commit_sha,
        commits,
    })
}

fn normalize_pull_request_ref(
    reference: ApiPullRequestRef,
) -> Result<GitHubPullRequestRef, GitHubError> {
    if reference.name.is_empty() || !valid_oid(&reference.sha) {
        return Err(GitHubError::ResponseParse);
    }
    let repository = reference
        .repo
        .map(|repository| {
            parse_name_with_owner(&repository.full_name)
                .map(|_| repository.full_name)
                .map_err(|_| GitHubError::ResponseParse)
        })
        .transpose()?;
    Ok(GitHubPullRequestRef {
        name: reference.name,
        oid: reference.sha,
        repository,
    })
}

fn normalize_commit(
    commit: ApiPullRequestCommit,
    name_with_owner: &str,
) -> Result<GitHubPullRequestCommit, GitHubError> {
    if !valid_oid(&commit.sha)
        || commit.html_url != format!("https://github.com/{name_with_owner}/commit/{}", commit.sha)
        || commit.parents.iter().any(|parent| !valid_oid(&parent.sha))
    {
        return Err(GitHubError::ResponseParse);
    }
    let author = commit.commit.author.ok_or(GitHubError::ResponseParse)?;
    let committer = commit.commit.committer.ok_or(GitHubError::ResponseParse)?;
    let summary = commit
        .commit
        .message
        .lines()
        .next()
        .unwrap_or_default()
        .to_owned();
    Ok(GitHubPullRequestCommit {
        oid: commit.sha,
        parents: commit
            .parents
            .into_iter()
            .map(|parent| parent.sha)
            .collect(),
        author: normalize_identity(author, commit.author)?,
        committer: normalize_identity(committer, commit.committer)?,
        summary,
        message: commit.commit.message,
        url: commit.html_url,
    })
}

fn normalize_identity(
    identity: ApiGitIdentity,
    account: Option<ApiLogin>,
) -> Result<GitHubCommitIdentity, GitHubError> {
    if identity.name.is_empty() || identity.date.is_empty() {
        return Err(GitHubError::ResponseParse);
    }
    Ok(GitHubCommitIdentity {
        name: identity.name,
        email: identity.email,
        timestamp: identity.date,
        login: account.map(|account| account.login),
    })
}

fn valid_oid(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitnova_protocol::RepositoryKind;
    use serde_json::{Value, json};
    use std::collections::VecDeque;
    use std::sync::Mutex;

    const OID_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const OID_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const OID_C: &str = "cccccccccccccccccccccccccccccccccccccccc";

    type RecordedCall = (String, Vec<OsString>, Vec<(String, String)>);

    struct FakeRunner {
        outputs: Mutex<VecDeque<Result<CommandOutput, io::Error>>>,
        calls: Mutex<Vec<RecordedCall>>,
    }

    impl FakeRunner {
        fn new(outputs: impl IntoIterator<Item = Result<CommandOutput, io::Error>>) -> Self {
            Self {
                outputs: Mutex::new(outputs.into_iter().collect()),
                calls: Mutex::new(Vec::new()),
            }
        }
    }

    impl CommandRunner for FakeRunner {
        fn run(
            &self,
            program: &str,
            arguments: &[OsString],
            environment: &[(&str, &str)],
        ) -> Result<CommandOutput, io::Error> {
            self.calls.lock().unwrap().push((
                program.to_owned(),
                arguments.to_vec(),
                environment
                    .iter()
                    .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
                    .collect(),
            ));
            self.outputs.lock().unwrap().pop_front().unwrap()
        }
    }

    fn descriptor() -> RepositoryDescriptor {
        RepositoryDescriptor {
            worktree_root: Some("/repo".into()),
            git_directory: "/repo/.git".into(),
            common_git_directory: "/repo/.git".into(),
            kind: RepositoryKind::Worktree,
            git_version: "test".into(),
        }
    }

    fn success(bytes: &[u8]) -> Result<CommandOutput, io::Error> {
        Ok(CommandOutput {
            exit_code: Some(0),
            stdout: bytes.to_vec(),
        })
    }

    fn success_value(value: Value) -> Result<CommandOutput, io::Error> {
        success(&serde_json::to_vec(&value).unwrap())
    }

    fn pull_request_params() -> GitHubPullRequestParams {
        GitHubPullRequestParams {
            number: 42,
            remote: None,
            name_with_owner: Some("owner/repo".into()),
        }
    }

    fn pull_request_detail(commit_count: usize) -> Value {
        json!({
            "number": 42,
            "title": "Preserve original commits",
            "body": "PR body",
            "state": "closed",
            "draft": false,
            "merged": true,
            "user": {"login": "octocat"},
            "html_url": "https://github.com/owner/repo/pull/42",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-02T00:00:00Z",
            "closed_at": "2026-01-02T00:00:00Z",
            "merged_at": "2026-01-02T00:00:00Z",
            "base": {"ref": "main", "sha": OID_A, "repo": {"full_name": "owner/repo"}},
            "head": {"ref": "feature", "sha": OID_B, "repo": null},
            "merge_commit_sha": OID_C,
            "commits": commit_count
        })
    }

    fn api_commit(oid: &str, parent: &str, message: &str, login: Option<&str>) -> Value {
        json!({
            "sha": oid,
            "html_url": format!("https://github.com/owner/repo/commit/{oid}"),
            "commit": {
                "author": {"name": "Author", "email": "author@example.com", "date": "2026-01-01T00:00:00Z"},
                "committer": {"name": "Committer", "email": "committer@example.com", "date": "2026-01-01T00:01:00Z"},
                "message": message
            },
            "author": login.map(|login| json!({"login": login})),
            "committer": null,
            "parents": [{"sha": parent}]
        })
    }

    #[test]
    fn parses_supported_urls_and_rejects_endpoint_injection() {
        for url in [
            "https://github.com/Camliar/GitNova.git",
            "ssh://git@github.com/Camliar/GitNova.git",
            "git@github.com:Camliar/GitNova.git",
        ] {
            assert_eq!(
                parse_github_url(url).unwrap(),
                ("Camliar".into(), "GitNova".into())
            );
        }
        for value in [
            "https://gitlab.com/a/b.git",
            "https://github.com/a/b/issues",
            "https://github.com/a/b.git?x=1",
            "-R/other",
            "owner/repo/extra",
        ] {
            assert!(parse_github_url(value).is_err());
        }
    }

    #[test]
    fn resolves_remote_and_normalizes_api_response_without_exposing_raw_json() {
        let response = br#"{"name":"GitNova","full_name":"Camliar/GitNova","owner":{"login":"Camliar"},"html_url":"https://github.com/Camliar/GitNova","default_branch":"main","private":false,"token":"secret"}"#;
        let runner = FakeRunner::new([
            success(b"git@github.com:Camliar/GitNova.git\0"),
            success(response),
        ]);
        let result =
            repository_with(&runner, &descriptor(), &GitHubRepositoryParams::default()).unwrap();
        assert_eq!(result.name_with_owner, "Camliar/GitNova");
        assert_eq!(result.default_branch, "main");
        let calls = runner.calls.lock().unwrap();
        assert_eq!(calls[0].0, "git");
        assert_eq!(calls[1].0, "gh");
        assert_eq!(calls[1].1[1], "repos/Camliar/GitNova");
        assert!(
            calls[1]
                .2
                .contains(&("GH_PROMPT_DISABLED".into(), "1".into()))
        );
        assert!(!serde_json::to_string(&result).unwrap().contains("secret"));
    }

    #[test]
    fn maps_gh_unavailable_auth_and_invalid_json_stably() {
        let params = GitHubRepositoryParams {
            remote: None,
            name_with_owner: Some("owner/repo".into()),
        };
        let unavailable = FakeRunner::new([Err(io::Error::new(io::ErrorKind::NotFound, "gh"))]);
        assert_eq!(
            repository_with(&unavailable, &descriptor(), &params),
            Err(GitHubError::GhUnavailable)
        );
        let auth = FakeRunner::new([Ok(CommandOutput {
            exit_code: Some(4),
            stdout: Vec::new(),
        })]);
        assert_eq!(
            repository_with(&auth, &descriptor(), &params),
            Err(GitHubError::AuthenticationRequired)
        );
        let invalid = FakeRunner::new([success(b"not json")]);
        assert_eq!(
            repository_with(&invalid, &descriptor(), &params),
            Err(GitHubError::ResponseParse)
        );
    }

    #[test]
    fn returns_merged_pr_and_flattens_original_commit_pages_in_order() {
        let runner = FakeRunner::new([
            success_value(pull_request_detail(2)),
            success_value(json!([
                [api_commit(
                    OID_A,
                    OID_C,
                    "first summary\n\nfirst body",
                    Some("author")
                )],
                [api_commit(OID_B, OID_A, "second summary", None)]
            ])),
        ]);
        let result = pull_request_with(&runner, &descriptor(), &pull_request_params()).unwrap();
        assert_eq!(result.state, GitHubPullRequestState::Merged);
        assert_eq!(result.merge_commit_oid.as_deref(), Some(OID_C));
        assert_eq!(result.head.repository, None);
        assert_eq!(result.commits.len(), 2);
        assert_eq!(result.commits[0].oid, OID_A);
        assert_eq!(result.commits[0].summary, "first summary");
        assert_eq!(result.commits[0].author.login.as_deref(), Some("author"));
        assert_eq!(result.commits[1].oid, OID_B);
        assert_eq!(result.commits[1].author.login, None);
        let calls = runner.calls.lock().unwrap();
        assert_eq!(calls[0].1[1], "repos/owner/repo/pulls/42");
        assert_eq!(
            calls[1].1[1],
            "repos/owner/repo/pulls/42/commits?per_page=100"
        );
        assert!(calls[1].1.contains(&OsString::from("--paginate")));
        assert!(calls[1].1.contains(&OsString::from("--slurp")));
    }

    #[test]
    fn rejects_pr_commit_limit_and_incomplete_pages() {
        let over_limit = FakeRunner::new([success_value(pull_request_detail(251))]);
        assert_eq!(
            pull_request_with(&over_limit, &descriptor(), &pull_request_params()),
            Err(GitHubError::PullRequestCommitLimit)
        );

        let incomplete = FakeRunner::new([
            success_value(pull_request_detail(2)),
            success_value(json!([[api_commit(OID_A, OID_C, "only", None)]])),
        ]);
        assert_eq!(
            pull_request_with(&incomplete, &descriptor(), &pull_request_params()),
            Err(GitHubError::ResponseParse)
        );
    }
}
