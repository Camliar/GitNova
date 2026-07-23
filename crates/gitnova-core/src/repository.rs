use gitnova_protocol::{RepositoryDescriptor, RepositoryKind};
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
    let output = git.run(&arguments).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            RepositoryError::GitUnavailable
        } else {
            RepositoryError::GitCommandFailed
        }
    })?;
    if !output.success {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("dubious ownership") {
            return Err(RepositoryError::UnsafeRepository);
        }
        if stderr.contains("not a git repository") {
            return Err(RepositoryError::NotFound);
        }
        return Err(RepositoryError::GitCommandFailed);
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
}
