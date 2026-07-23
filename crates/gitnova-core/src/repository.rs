use gitnova_protocol::{
    BranchStatus, FileStatus, RepositoryDescriptor, RepositoryKind, StatusEntry, StatusEntryKind,
    WorkingTreeStatus,
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
}
