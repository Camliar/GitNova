use gitnova_protocol::{GitHubRepository, GitHubRepositoryParams, RepositoryDescriptor};
use serde::Deserialize;
use std::ffi::OsString;
use std::io;
use std::process::Command;

const MAX_REMOTE_OUTPUT_BYTES: usize = 16 * 1024;
const MAX_GITHUB_RESPONSE_BYTES: usize = 1024 * 1024;

#[derive(Debug, Eq, PartialEq)]
pub enum GitHubError {
    InvalidRemote,
    RemoteNotFound,
    UnsupportedRemote,
    GhUnavailable,
    AuthenticationRequired,
    RequestFailed,
    ResponseParse,
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

fn repository_with(
    runner: &impl CommandRunner,
    descriptor: &RepositoryDescriptor,
    params: &GitHubRepositoryParams,
) -> Result<GitHubRepository, GitHubError> {
    let (owner, name) = if let Some(name_with_owner) = &params.name_with_owner {
        parse_name_with_owner(name_with_owner)?
    } else {
        let remote = params.remote.as_deref().unwrap_or("origin");
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
        parse_github_url(url)?
    };

    let endpoint = format!("repos/{owner}/{name}");
    let arguments = [
        OsString::from("api"),
        OsString::from(endpoint),
        OsString::from("--hostname"),
        OsString::from("github.com"),
    ];
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
        Some(0) => parse_response(&output.stdout),
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

#[cfg(test)]
mod tests {
    use super::*;
    use gitnova_protocol::RepositoryKind;
    use std::collections::VecDeque;
    use std::sync::Mutex;

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
}
