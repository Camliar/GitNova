mod ai;
mod framing;
mod github;
mod gitlab;
mod repository;

use gitnova_protocol::{
    AiGenerateCommitDraftParams, AiInputPreviewParams, BranchParams, CancelParams,
    CancellationRegistry, CommitDiffParams, CommitFileDiffParams, CommitFilesParams, CommitParams,
    DiffParams, ERROR_AI_CREDENTIAL_MISSING, ERROR_AI_EXTERNAL_CONFIRMATION_REQUIRED,
    ERROR_AI_INPUT_LIMIT, ERROR_AI_INVALID_PROVIDER, ERROR_AI_NOTHING_STAGED,
    ERROR_AI_PREVIEW_STALE, ERROR_AI_PROVIDER_UNAVAILABLE, ERROR_AI_REQUEST_FAILED,
    ERROR_AI_RESPONSE_INVALID, ERROR_ALREADY_INITIALIZED, ERROR_BRANCH_ALREADY_EXISTS,
    ERROR_BRANCH_NOT_FOUND, ERROR_COMMIT_DIFF_PARSE, ERROR_COMMIT_FILE_DIFF_LIMIT,
    ERROR_COMMIT_FILE_LIMIT, ERROR_COMMIT_MESSAGE_REQUIRED, ERROR_COMMIT_NOT_FOUND,
    ERROR_COMMIT_PARENT_REQUIRED, ERROR_COMMIT_PARSE, ERROR_DIFF_PARSE,
    ERROR_DIFFERENT_REPOSITORY_OPEN, ERROR_GH_UNAVAILABLE, ERROR_GIT_COMMAND_FAILED,
    ERROR_GIT_UNAVAILABLE, ERROR_GITHUB_AUTH_REQUIRED, ERROR_GITHUB_COMMIT_ASSOCIATION_AMBIGUOUS,
    ERROR_GITHUB_COMMIT_FILE_LIMIT, ERROR_GITHUB_COMMIT_NOT_IN_PR, ERROR_GITHUB_INVALID_REMOTE,
    ERROR_GITHUB_PR_COMMIT_LIMIT, ERROR_GITHUB_REMOTE_NOT_FOUND, ERROR_GITHUB_REQUEST_FAILED,
    ERROR_GITHUB_RESPONSE_PARSE, ERROR_GITHUB_UNSUPPORTED_REMOTE, ERROR_GITLAB_AUTH_REQUIRED,
    ERROR_GITLAB_COMMIT_FILE_LIMIT, ERROR_GITLAB_COMMIT_NOT_IN_MR, ERROR_GITLAB_INVALID_REMOTE,
    ERROR_GITLAB_MR_COMMIT_LIMIT, ERROR_GITLAB_REMOTE_NOT_FOUND, ERROR_GITLAB_REQUEST_FAILED,
    ERROR_GITLAB_RESPONSE_PARSE, ERROR_GITLAB_UNSUPPORTED_REMOTE, ERROR_GLAB_UNAVAILABLE,
    ERROR_HISTORY_ENCODING, ERROR_INCOMPATIBLE_PROTOCOL, ERROR_INVALID_BRANCH_NAME,
    ERROR_INVALID_COMMIT_PARENT, ERROR_INVALID_HISTORY_CURSOR, ERROR_INVALID_PARAMS,
    ERROR_INVALID_PATH, ERROR_INVALID_REPOSITORY_PATH, ERROR_INVALID_REQUEST,
    ERROR_METHOD_NOT_FOUND, ERROR_MUTATION_FAILED, ERROR_NOT_INITIALIZED, ERROR_NOTHING_STAGED,
    ERROR_PARSE, ERROR_REFERENCE_ENCODING, ERROR_REFERENCE_PARSE, ERROR_REPOSITORY_NOT_FOUND,
    ERROR_REPOSITORY_NOT_OPEN, ERROR_REQUEST_CANCELLED, ERROR_STATUS_PARSE,
    ERROR_SYNC_BRANCH_REQUIRED, ERROR_SYNC_DIVERGED, ERROR_SYNC_FETCH_FAILED,
    ERROR_SYNC_INVALID_REMOTE, ERROR_SYNC_PULL_FAILED, ERROR_SYNC_PUSH_FAILED,
    ERROR_SYNC_REMOTE_NOT_FOUND, ERROR_SYNC_STALE_HEAD, ERROR_SYNC_UPSTREAM_REQUIRED,
    ERROR_UNBORN_HEAD, ERROR_UNRESOLVED_CONFLICTS, ERROR_UNSAFE_REPOSITORY,
    ERROR_WORKTREE_REQUIRED, GitHubCommitSquashTraceParams, GitHubPullRequestCommitDiff,
    GitHubPullRequestCommitDiffParams, GitHubPullRequestCommitFileDiffParams,
    GitHubPullRequestCommitFilesParams, GitHubPullRequestParams, GitHubRepositoryParams,
    GitLabMergeRequestCommitDiffParams, GitLabMergeRequestParams, GitLabProjectParams,
    HistoryParams, ImplementationInfo, InitializeParams, InitializeResult, JSON_RPC_VERSION,
    Notification, PROTOCOL_VERSION, RepositoryDescriptor, RepositoryFetchParams,
    RepositoryPathParams, RepositorySyncOperation, RepositorySyncParams, Request, Response,
    ResponseError, ServerCapabilities,
};
use serde_json::Value;
use std::io::{self, BufRead, Write};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Lifecycle {
    Uninitialized,
    Initialized,
    Shutdown,
}

struct CoreState {
    lifecycle: Lifecycle,
    active_repository: Option<RepositoryDescriptor>,
    github_commit_diff_cache: Option<CachedGitHubCommitDiff>,
}

struct CachedGitHubCommitDiff {
    remote: Option<String>,
    requested_name_with_owner: Option<String>,
    value: GitHubPullRequestCommitDiff,
}

impl Default for CoreState {
    fn default() -> Self {
        Self {
            lifecycle: Lifecycle::Uninitialized,
            active_repository: None,
            github_commit_diff_cache: None,
        }
    }
}

pub fn run(reader: &mut impl BufRead, writer: &mut impl Write) -> io::Result<i32> {
    let mut state = CoreState::default();
    let cancellations = CancellationRegistry::default();

    while let Some(body) = framing::read_frame(reader)? {
        let value: Value = match serde_json::from_slice(&body) {
            Ok(value) => value,
            Err(_) => {
                write_response(
                    writer,
                    &Response::error(
                        None,
                        ResponseError::new(
                            ERROR_PARSE,
                            "protocol.parse_error",
                            "Invalid JSON payload",
                            false,
                        ),
                    ),
                )?;
                continue;
            }
        };

        if value.get("id").is_some() {
            let request: Request = match serde_json::from_value::<Request>(value) {
                Ok(request) if request.jsonrpc == JSON_RPC_VERSION => request,
                _ => {
                    write_response(
                        writer,
                        &Response::error(
                            None,
                            ResponseError::new(
                                ERROR_INVALID_REQUEST,
                                "protocol.invalid_request",
                                "Invalid JSON-RPC request",
                                false,
                            ),
                        ),
                    )?;
                    continue;
                }
            };
            let response = dispatch_request(request, &mut state, &cancellations);
            write_response(writer, &response)?;
        } else {
            let notification: Notification = match serde_json::from_value::<Notification>(value) {
                Ok(notification) if notification.jsonrpc == JSON_RPC_VERSION => notification,
                _ => continue,
            };
            if dispatch_notification(notification, state.lifecycle, &cancellations) {
                return Ok(if state.lifecycle == Lifecycle::Shutdown {
                    0
                } else {
                    1
                });
            }
        }
    }

    Ok(0)
}

fn dispatch_request(
    request: Request,
    state: &mut CoreState,
    cancellations: &CancellationRegistry,
) -> Response {
    if cancellations.take_cancelled(&request.id) {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_REQUEST_CANCELLED,
                "request.cancelled",
                "Request cancelled",
                true,
            ),
        );
    }

    match request.method.as_str() {
        "gitnova/initialize" => initialize(request, state),
        "gitnova/shutdown" => shutdown(request, state),
        _ if state.lifecycle == Lifecycle::Uninitialized => Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_NOT_INITIALIZED,
                "core.not_initialized",
                "Core must be initialized before handling this request",
                true,
            ),
        ),
        _ if state.lifecycle == Lifecycle::Shutdown => Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_INVALID_REQUEST,
                "core.already_shutdown",
                "Core has already shut down",
                false,
            ),
        ),
        "repository/discover" => repository_request(request, state, false),
        "repository/open" => repository_request(request, state, true),
        "repository/status" => status_request(request, state),
        "repository/diff" => diff_request(request, state),
        "repository/history" => history_request(request, state),
        "repository/commitDiff" => commit_diff_request(request, state),
        "repository/commitFiles" => commit_files_request(request, state),
        "repository/commitFileDiff" => commit_file_diff_request(request, state),
        "repository/references" => references_request(request, state),
        "repository/graph" => graph_request(request, state),
        "repository/commit" => commit_request(request, state),
        "repository/createBranch" => branch_request(request, state, false),
        "repository/switchBranch" => branch_request(request, state, true),
        "repository/fetch" => repository_fetch_request(request, state),
        "repository/pull" => repository_sync_request(request, state, RepositorySyncOperation::Pull),
        "repository/push" => repository_sync_request(request, state, RepositorySyncOperation::Push),
        "github/repository" => github_repository_request(request, state),
        "github/pullRequest" => github_pull_request_request(request, state),
        "github/pullRequestCommitDiff" => github_pull_request_commit_diff_request(request, state),
        "github/squashTrace" => github_squash_trace_request(request, state),
        "github/commitSquashTrace" => github_commit_squash_trace_request(request, state),
        "github/pullRequestCommitFiles" => github_pull_request_commit_files_request(request, state),
        "github/pullRequestCommitFileDiff" => {
            github_pull_request_commit_file_diff_request(request, state)
        }
        "gitlab/project" => gitlab_project_request(request, state),
        "gitlab/mergeRequest" => gitlab_merge_request_request(request, state),
        "gitlab/mergeRequestCommitDiff" => gitlab_merge_request_commit_diff_request(request, state),
        "gitlab/squashTrace" => gitlab_squash_trace_request(request, state),
        "ai/inputPreview" => ai_input_preview_request(request, state),
        "ai/generateCommitDraft" => ai_generate_commit_draft_request(request, state),
        _ => Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_METHOD_NOT_FOUND,
                "protocol.method_not_found",
                "Method not found",
                false,
            ),
        ),
    }
}

fn initialize(request: Request, state: &mut CoreState) -> Response {
    if state.lifecycle != Lifecycle::Uninitialized {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_ALREADY_INITIALIZED,
                "core.already_initialized",
                "Core can only be initialized once",
                false,
            ),
        );
    }

    let params: InitializeParams = match serde_json::from_value(request.params) {
        Ok(params) => params,
        Err(_) => {
            return Response::error(
                Some(request.id),
                ResponseError::new(
                    ERROR_INVALID_PARAMS,
                    "protocol.invalid_params",
                    "Invalid initialize parameters",
                    false,
                ),
            );
        }
    };

    if major_version(&params.protocol_version) != major_version(PROTOCOL_VERSION) {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_INCOMPATIBLE_PROTOCOL,
                "protocol.incompatible_version",
                "Incompatible protocol major version",
                false,
            ),
        );
    }

    state.lifecycle = Lifecycle::Initialized;
    let result = InitializeResult {
        core_info: ImplementationInfo {
            name: "gitnova-core".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        },
        protocol_version: PROTOCOL_VERSION.into(),
        capabilities: ServerCapabilities {
            cancellation: true,
            repository_discovery: true,
            working_tree_status: true,
            structured_file_diff: true,
            paginated_commit_history: true,
            structured_commit_diff: true,
            lazy_commit_diff: true,
            history_squash_trace: true,
            repository_sync: true,
            repository_references: true,
            commit_graph_projection: true,
            github_repository: true,
            github_pull_request: true,
            github_pull_request_commit_diff: true,
            github_squash_trace: true,
            gitlab_project: true,
            gitlab_merge_request: true,
            gitlab_merge_request_commit_diff: true,
            gitlab_squash_trace: true,
            ai_assist: true,
            repository_mutations: true,
        },
    };
    Response::success(
        request.id,
        serde_json::to_value(result).expect("serializable result"),
    )
}

fn shutdown(request: Request, state: &mut CoreState) -> Response {
    if state.lifecycle == Lifecycle::Uninitialized {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_NOT_INITIALIZED,
                "core.not_initialized",
                "Core must be initialized before shutdown",
                true,
            ),
        );
    }
    if state.lifecycle == Lifecycle::Shutdown {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_INVALID_REQUEST,
                "core.already_shutdown",
                "Core has already shut down",
                false,
            ),
        );
    }
    state.lifecycle = Lifecycle::Shutdown;
    Response::success(request.id, Value::Null)
}

fn repository_request(request: Request, state: &mut CoreState, open: bool) -> Response {
    let params: RepositoryPathParams = match serde_json::from_value(request.params) {
        Ok(params) => params,
        Err(_) => {
            return Response::error(
                Some(request.id),
                ResponseError::new(
                    ERROR_INVALID_PARAMS,
                    "protocol.invalid_params",
                    "Invalid repository path parameters",
                    false,
                ),
            );
        }
    };
    let descriptor = match repository::discover(&params.path) {
        Ok(descriptor) => descriptor,
        Err(error) => return Response::error(Some(request.id), repository_error(error)),
    };

    if open {
        if let Some(active) = &state.active_repository {
            if active.git_directory != descriptor.git_directory {
                return Response::error(
                    Some(request.id),
                    ResponseError::new(
                        ERROR_DIFFERENT_REPOSITORY_OPEN,
                        "repository.different_repository_open",
                        "A different repository is already open in this Core session",
                        false,
                    ),
                );
            }
        } else {
            state.active_repository = Some(descriptor.clone());
        }
    }

    Response::success(
        request.id,
        serde_json::to_value(descriptor).expect("serializable repository descriptor"),
    )
}

fn repository_error(error: repository::RepositoryError) -> ResponseError {
    match error {
        repository::RepositoryError::InvalidPath => ResponseError::new(
            ERROR_INVALID_PATH,
            "path.invalid",
            "Repository path does not exist or is invalid",
            false,
        ),
        repository::RepositoryError::UnsupportedPathEncoding => ResponseError::new(
            ERROR_INVALID_PATH,
            "path.unsupported_encoding",
            "Repository path cannot be represented by the protocol",
            false,
        ),
        repository::RepositoryError::NotFound => ResponseError::new(
            ERROR_REPOSITORY_NOT_FOUND,
            "repository.not_found",
            "No Git repository was found for this path",
            false,
        ),
        repository::RepositoryError::GitUnavailable => ResponseError::new(
            ERROR_GIT_UNAVAILABLE,
            "git.unavailable",
            "System Git is unavailable",
            true,
        ),
        repository::RepositoryError::GitCommandFailed => ResponseError::new(
            ERROR_GIT_COMMAND_FAILED,
            "git.command_failed",
            "System Git could not inspect the repository",
            true,
        ),
        repository::RepositoryError::UnsafeRepository => ResponseError::new(
            ERROR_UNSAFE_REPOSITORY,
            "repository.unsafe_ownership",
            "Git rejected the repository ownership as unsafe",
            false,
        ),
        repository::RepositoryError::WorktreeRequired => ResponseError::new(
            ERROR_WORKTREE_REQUIRED,
            "repository.worktree_required",
            "This operation requires a non-bare worktree",
            false,
        ),
        repository::RepositoryError::StatusParse => ResponseError::new(
            ERROR_STATUS_PARSE,
            "git.status_parse_failed",
            "System Git returned an invalid status payload",
            false,
        ),
        repository::RepositoryError::DiffParse => ResponseError::new(
            ERROR_DIFF_PARSE,
            "git.diff_parse_failed",
            "System Git returned an invalid patch payload",
            false,
        ),
        repository::RepositoryError::InvalidRepositoryPath => ResponseError::new(
            ERROR_INVALID_REPOSITORY_PATH,
            "path.invalid_repository_relative",
            "Diff path must be a safe repository-relative file path",
            false,
        ),
        repository::RepositoryError::InvalidHistoryCursor => ResponseError::new(
            ERROR_INVALID_HISTORY_CURSOR,
            "history.invalid_cursor",
            "History cursor is invalid or no longer available",
            false,
        ),
        repository::RepositoryError::CommitParse => ResponseError::new(
            ERROR_COMMIT_PARSE,
            "git.commit_parse_failed",
            "System Git returned an invalid commit object",
            false,
        ),
        repository::RepositoryError::HistoryEncoding => ResponseError::new(
            ERROR_HISTORY_ENCODING,
            "history.unsupported_encoding",
            "Commit metadata is not UTF-8 encoded",
            false,
        ),
        repository::RepositoryError::CommitNotFound => ResponseError::new(
            ERROR_COMMIT_NOT_FOUND,
            "commit.not_found",
            "Commit does not exist in the opened repository",
            false,
        ),
        repository::RepositoryError::CommitParentRequired => ResponseError::new(
            ERROR_COMMIT_PARENT_REQUIRED,
            "commit.parent_required",
            "A direct parent must be selected for a merge commit",
            false,
        ),
        repository::RepositoryError::InvalidCommitParent => ResponseError::new(
            ERROR_INVALID_COMMIT_PARENT,
            "commit.invalid_parent",
            "Selected parent is not a direct parent of the commit",
            false,
        ),
        repository::RepositoryError::CommitDiffParse => ResponseError::new(
            ERROR_COMMIT_DIFF_PARSE,
            "git.commit_diff_parse_failed",
            "System Git returned an invalid commit diff payload",
            false,
        ),
        repository::RepositoryError::CommitFileLimit => ResponseError::new(
            ERROR_COMMIT_FILE_LIMIT,
            "commit.file_limit",
            "Commit changes exceed the safe file-list limit",
            false,
        ),
        repository::RepositoryError::CommitFileDiffLimit => ResponseError::new(
            ERROR_COMMIT_FILE_DIFF_LIMIT,
            "commit.file_diff_limit",
            "Selected file diff exceeds the safe response limit",
            false,
        ),
        repository::RepositoryError::ReferenceParse => ResponseError::new(
            ERROR_REFERENCE_PARSE,
            "git.reference_parse_failed",
            "System Git returned an invalid reference payload",
            false,
        ),
        repository::RepositoryError::ReferenceEncoding => ResponseError::new(
            ERROR_REFERENCE_ENCODING,
            "reference.unsupported_encoding",
            "Reference metadata is not UTF-8 encoded",
            false,
        ),
        repository::RepositoryError::CommitMessageRequired => ResponseError::new(
            ERROR_COMMIT_MESSAGE_REQUIRED,
            "commit.message_required",
            "Commit message must contain text and be at most 65536 bytes",
            false,
        ),
        repository::RepositoryError::NothingStaged => ResponseError::new(
            ERROR_NOTHING_STAGED,
            "commit.nothing_staged",
            "There are no staged changes to commit",
            false,
        ),
        repository::RepositoryError::UnresolvedConflicts => ResponseError::new(
            ERROR_UNRESOLVED_CONFLICTS,
            "commit.unresolved_conflicts",
            "Resolve index conflicts before committing",
            false,
        ),
        repository::RepositoryError::InvalidBranchName => ResponseError::new(
            ERROR_INVALID_BRANCH_NAME,
            "branch.invalid_name",
            "Branch name is invalid",
            false,
        ),
        repository::RepositoryError::BranchAlreadyExists => ResponseError::new(
            ERROR_BRANCH_ALREADY_EXISTS,
            "branch.already_exists",
            "Local branch already exists",
            false,
        ),
        repository::RepositoryError::BranchNotFound => ResponseError::new(
            ERROR_BRANCH_NOT_FOUND,
            "branch.not_found",
            "Local branch was not found",
            false,
        ),
        repository::RepositoryError::UnbornHead => ResponseError::new(
            ERROR_UNBORN_HEAD,
            "branch.unborn_head",
            "Create the first commit before creating another branch",
            false,
        ),
        repository::RepositoryError::MutationFailed => ResponseError::new(
            ERROR_MUTATION_FAILED,
            "git.mutation_failed",
            "System Git rejected the requested mutation",
            true,
        ),
        repository::RepositoryError::SyncInvalidRemote => ResponseError::new(
            ERROR_SYNC_INVALID_REMOTE,
            "sync.invalid_remote",
            "Remote name is invalid",
            false,
        ),
        repository::RepositoryError::SyncRemoteNotFound => ResponseError::new(
            ERROR_SYNC_REMOTE_NOT_FOUND,
            "sync.remote_not_found",
            "The selected Git remote does not exist",
            false,
        ),
        repository::RepositoryError::SyncBranchRequired => ResponseError::new(
            ERROR_SYNC_BRANCH_REQUIRED,
            "sync.branch_required",
            "Repository sync requires an attached local branch",
            false,
        ),
        repository::RepositoryError::SyncUpstreamRequired => ResponseError::new(
            ERROR_SYNC_UPSTREAM_REQUIRED,
            "sync.upstream_required",
            "Pull requires an upstream branch",
            false,
        ),
        repository::RepositoryError::SyncStaleHead => ResponseError::new(
            ERROR_SYNC_STALE_HEAD,
            "sync.stale_head",
            "The current branch or HEAD changed after confirmation",
            true,
        ),
        repository::RepositoryError::SyncDiverged => ResponseError::new(
            ERROR_SYNC_DIVERGED,
            "sync.diverged",
            "Local and upstream branches have diverged; fast-forward pull was refused",
            false,
        ),
        repository::RepositoryError::SyncFetchFailed => ResponseError::new(
            ERROR_SYNC_FETCH_FAILED,
            "sync.fetch_failed",
            "Git fetch failed without changing the worktree",
            true,
        ),
        repository::RepositoryError::SyncPullFailed => ResponseError::new(
            ERROR_SYNC_PULL_FAILED,
            "sync.pull_failed",
            "Fast-forward pull could not be applied",
            true,
        ),
        repository::RepositoryError::SyncPushFailed => ResponseError::new(
            ERROR_SYNC_PUSH_FAILED,
            "sync.push_failed",
            "Non-force push was rejected or unavailable",
            true,
        ),
    }
}

fn github_error(error: github::GitHubError) -> ResponseError {
    match error {
        github::GitHubError::InvalidRemote => ResponseError::new(
            ERROR_GITHUB_INVALID_REMOTE,
            "github.invalid_remote",
            "Remote name is invalid",
            false,
        ),
        github::GitHubError::RemoteNotFound => ResponseError::new(
            ERROR_GITHUB_REMOTE_NOT_FOUND,
            "github.remote_not_found",
            "GitHub remote was not found",
            false,
        ),
        github::GitHubError::UnsupportedRemote => ResponseError::new(
            ERROR_GITHUB_UNSUPPORTED_REMOTE,
            "github.unsupported_remote",
            "Remote is not a supported github.com repository",
            false,
        ),
        github::GitHubError::GhUnavailable => ResponseError::new(
            ERROR_GH_UNAVAILABLE,
            "github.gh_unavailable",
            "GitHub CLI is unavailable in the repository environment",
            true,
        ),
        github::GitHubError::AuthenticationRequired => ResponseError::new(
            ERROR_GITHUB_AUTH_REQUIRED,
            "github.authentication_required",
            "GitHub CLI authentication is required",
            true,
        ),
        github::GitHubError::RequestFailed => ResponseError::new(
            ERROR_GITHUB_REQUEST_FAILED,
            "github.request_failed",
            "GitHub request failed",
            true,
        ),
        github::GitHubError::ResponseParse => ResponseError::new(
            ERROR_GITHUB_RESPONSE_PARSE,
            "github.response_parse_failed",
            "GitHub returned an invalid response",
            false,
        ),
        github::GitHubError::PullRequestCommitLimit => ResponseError::new(
            ERROR_GITHUB_PR_COMMIT_LIMIT,
            "github.pr_commit_limit_exceeded",
            "Pull request exceeds the supported original commit limit",
            false,
        ),
        github::GitHubError::CommitNotInPullRequest => ResponseError::new(
            ERROR_GITHUB_COMMIT_NOT_IN_PR,
            "github.commit_not_in_pull_request",
            "Commit is not an original commit of the pull request",
            false,
        ),
        github::GitHubError::CommitFileLimit => ResponseError::new(
            ERROR_GITHUB_COMMIT_FILE_LIMIT,
            "github.commit_file_limit_exceeded",
            "Commit reaches the supported GitHub file limit",
            false,
        ),
        github::GitHubError::CommitAssociationAmbiguous => ResponseError::new(
            ERROR_GITHUB_COMMIT_ASSOCIATION_AMBIGUOUS,
            "github.commit_association_ambiguous",
            "Multiple pull requests report the selected commit as their merge result",
            false,
        ),
    }
}

fn squash_trace_error(error: github::SquashTraceError) -> ResponseError {
    match error {
        github::SquashTraceError::GitHub(error) => github_error(error),
        github::SquashTraceError::Repository(error) => repository_error(error),
    }
}

fn gitlab_error(error: gitlab::GitLabError) -> ResponseError {
    match error {
        gitlab::GitLabError::InvalidRemote => ResponseError::new(
            ERROR_GITLAB_INVALID_REMOTE,
            "gitlab.invalid_remote",
            "Remote name is invalid",
            false,
        ),
        gitlab::GitLabError::RemoteNotFound => ResponseError::new(
            ERROR_GITLAB_REMOTE_NOT_FOUND,
            "gitlab.remote_not_found",
            "GitLab remote was not found",
            false,
        ),
        gitlab::GitLabError::UnsupportedRemote => ResponseError::new(
            ERROR_GITLAB_UNSUPPORTED_REMOTE,
            "gitlab.unsupported_remote",
            "Remote is not a supported GitLab repository",
            false,
        ),
        gitlab::GitLabError::GlabUnavailable => ResponseError::new(
            ERROR_GLAB_UNAVAILABLE,
            "gitlab.glab_unavailable",
            "GitLab CLI is unavailable in the repository environment",
            true,
        ),
        gitlab::GitLabError::AuthenticationRequired => ResponseError::new(
            ERROR_GITLAB_AUTH_REQUIRED,
            "gitlab.authentication_required",
            "GitLab CLI authentication is required",
            true,
        ),
        gitlab::GitLabError::RequestFailed => ResponseError::new(
            ERROR_GITLAB_REQUEST_FAILED,
            "gitlab.request_failed",
            "GitLab request failed",
            true,
        ),
        gitlab::GitLabError::ResponseParse => ResponseError::new(
            ERROR_GITLAB_RESPONSE_PARSE,
            "gitlab.response_parse_failed",
            "GitLab returned an invalid response",
            false,
        ),
        gitlab::GitLabError::MergeRequestCommitLimit => ResponseError::new(
            ERROR_GITLAB_MR_COMMIT_LIMIT,
            "gitlab.mr_commit_limit_exceeded",
            "Merge request exceeds the supported original commit limit",
            false,
        ),
        gitlab::GitLabError::CommitNotInMergeRequest => ResponseError::new(
            ERROR_GITLAB_COMMIT_NOT_IN_MR,
            "gitlab.commit_not_in_merge_request",
            "Commit is not an original commit of the merge request",
            false,
        ),
        gitlab::GitLabError::CommitFileLimit => ResponseError::new(
            ERROR_GITLAB_COMMIT_FILE_LIMIT,
            "gitlab.commit_file_limit_exceeded",
            "Commit reaches the supported GitLab file limit",
            false,
        ),
    }
}

fn gitlab_squash_trace_error(error: gitlab::SquashTraceError) -> ResponseError {
    match error {
        gitlab::SquashTraceError::GitLab(error) => gitlab_error(error),
        gitlab::SquashTraceError::Repository(error) => repository_error(error),
    }
}

fn status_request(request: Request, state: &CoreState) -> Response {
    let params_are_empty = request.params.is_null()
        || request
            .params
            .as_object()
            .is_some_and(serde_json::Map::is_empty);
    if !params_are_empty {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_INVALID_PARAMS,
                "protocol.invalid_params",
                "repository/status does not accept parameters",
                false,
            ),
        );
    }
    let Some(descriptor) = &state.active_repository else {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_REPOSITORY_NOT_OPEN,
                "repository.not_open",
                "Open a repository before requesting status",
                true,
            ),
        );
    };
    match repository::status(descriptor) {
        Ok(status) => Response::success(
            request.id,
            serde_json::to_value(status).expect("serializable working tree status"),
        ),
        Err(error) => Response::error(Some(request.id), repository_error(error)),
    }
}

fn diff_request(request: Request, state: &CoreState) -> Response {
    let params: DiffParams = match serde_json::from_value(request.params) {
        Ok(params) => params,
        Err(_) => {
            return Response::error(
                Some(request.id),
                ResponseError::new(
                    ERROR_INVALID_PARAMS,
                    "protocol.invalid_params",
                    "Invalid repository diff parameters",
                    false,
                ),
            );
        }
    };
    let context_lines = params.context_lines.unwrap_or(3);
    if context_lines > 20 {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_INVALID_PARAMS,
                "protocol.invalid_params",
                "contextLines must be between 0 and 20",
                false,
            ),
        );
    }
    let Some(descriptor) = &state.active_repository else {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_REPOSITORY_NOT_OPEN,
                "repository.not_open",
                "Open a repository before requesting a diff",
                true,
            ),
        );
    };
    match repository::diff(descriptor, &params.path, params.scope, context_lines) {
        Ok(diff) => Response::success(
            request.id,
            serde_json::to_value(diff).expect("serializable file diff"),
        ),
        Err(error) => Response::error(Some(request.id), repository_error(error)),
    }
}

fn history_request(request: Request, state: &CoreState) -> Response {
    let params = if request.params.is_null() {
        HistoryParams::default()
    } else {
        match serde_json::from_value::<HistoryParams>(request.params) {
            Ok(params) => params,
            Err(_) => {
                return Response::error(
                    Some(request.id),
                    ResponseError::new(
                        ERROR_INVALID_PARAMS,
                        "protocol.invalid_params",
                        "Invalid repository history parameters",
                        false,
                    ),
                );
            }
        }
    };
    let limit = params.limit.unwrap_or(50);
    if !(1..=200).contains(&limit) {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_INVALID_PARAMS,
                "protocol.invalid_params",
                "limit must be between 1 and 200",
                false,
            ),
        );
    }
    let Some(descriptor) = &state.active_repository else {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_REPOSITORY_NOT_OPEN,
                "repository.not_open",
                "Open a repository before requesting history",
                true,
            ),
        );
    };
    match repository::history(descriptor, limit, params.cursor.as_deref()) {
        Ok(page) => Response::success(
            request.id,
            serde_json::to_value(page).expect("serializable history page"),
        ),
        Err(error) => Response::error(Some(request.id), repository_error(error)),
    }
}

fn commit_diff_request(request: Request, state: &CoreState) -> Response {
    let params: CommitDiffParams = match serde_json::from_value(request.params) {
        Ok(params) => params,
        Err(_) => {
            return Response::error(
                Some(request.id),
                ResponseError::new(
                    ERROR_INVALID_PARAMS,
                    "protocol.invalid_params",
                    "Invalid commit diff parameters",
                    false,
                ),
            );
        }
    };
    if !repository::valid_oid(&params.oid)
        || params
            .parent_oid
            .as_deref()
            .is_some_and(|oid| !repository::valid_oid(oid))
    {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_INVALID_PARAMS,
                "protocol.invalid_params",
                "oid and parentOid must be full hexadecimal object IDs",
                false,
            ),
        );
    }
    let context_lines = params.context_lines.unwrap_or(3);
    if context_lines > 20 {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_INVALID_PARAMS,
                "protocol.invalid_params",
                "contextLines must be between 0 and 20",
                false,
            ),
        );
    }
    let Some(descriptor) = &state.active_repository else {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_REPOSITORY_NOT_OPEN,
                "repository.not_open",
                "Open a repository before requesting a commit diff",
                true,
            ),
        );
    };
    match repository::commit_diff(
        descriptor,
        &params.oid,
        params.parent_oid.as_deref(),
        context_lines,
    ) {
        Ok(diff) => Response::success(
            request.id,
            serde_json::to_value(diff).expect("serializable commit diff"),
        ),
        Err(error) => Response::error(Some(request.id), repository_error(error)),
    }
}

fn commit_files_request(request: Request, state: &CoreState) -> Response {
    let params: CommitFilesParams = match serde_json::from_value(request.params) {
        Ok(params) => params,
        Err(_) => return invalid_params(request.id, "Invalid commit files parameters"),
    };
    if !repository::valid_oid(&params.oid)
        || params
            .parent_oid
            .as_deref()
            .is_some_and(|oid| !repository::valid_oid(oid))
    {
        return invalid_params(
            request.id,
            "oid and parentOid must be full hexadecimal object IDs",
        );
    }
    let Some(descriptor) = &state.active_repository else {
        return repository_not_open(request.id, "requesting commit files");
    };
    match repository::commit_files(descriptor, &params.oid, params.parent_oid.as_deref()) {
        Ok(files) => Response::success(
            request.id,
            serde_json::to_value(files).expect("serializable commit files"),
        ),
        Err(error) => Response::error(Some(request.id), repository_error(error)),
    }
}

fn commit_file_diff_request(request: Request, state: &CoreState) -> Response {
    let params: CommitFileDiffParams = match serde_json::from_value(request.params) {
        Ok(params) => params,
        Err(_) => return invalid_params(request.id, "Invalid commit file diff parameters"),
    };
    if !repository::valid_oid(&params.oid)
        || params
            .parent_oid
            .as_deref()
            .is_some_and(|oid| !repository::valid_oid(oid))
        || params.path.is_empty()
        || params.path.len() > 4096
    {
        return invalid_params(request.id, "Invalid commit file diff parameters");
    }
    let context_lines = params.context_lines.unwrap_or(3);
    if context_lines > 20 {
        return invalid_params(request.id, "contextLines must be between 0 and 20");
    }
    let Some(descriptor) = &state.active_repository else {
        return repository_not_open(request.id, "requesting a commit file diff");
    };
    match repository::commit_file_diff(
        descriptor,
        &params.oid,
        params.parent_oid.as_deref(),
        &params.path,
        context_lines,
    ) {
        Ok(diff) => Response::success(
            request.id,
            serde_json::to_value(diff).expect("serializable commit file diff"),
        ),
        Err(error) => Response::error(Some(request.id), repository_error(error)),
    }
}

fn references_request(request: Request, state: &CoreState) -> Response {
    let params_are_empty = request.params.is_null()
        || request
            .params
            .as_object()
            .is_some_and(serde_json::Map::is_empty);
    if !params_are_empty {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_INVALID_PARAMS,
                "protocol.invalid_params",
                "repository/references does not accept parameters",
                false,
            ),
        );
    }
    let Some(descriptor) = &state.active_repository else {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_REPOSITORY_NOT_OPEN,
                "repository.not_open",
                "Open a repository before requesting references",
                true,
            ),
        );
    };
    match repository::references(descriptor) {
        Ok(references) => Response::success(
            request.id,
            serde_json::to_value(references).expect("serializable repository references"),
        ),
        Err(error) => Response::error(Some(request.id), repository_error(error)),
    }
}

fn graph_request(request: Request, state: &CoreState) -> Response {
    let params = if request.params.is_null() {
        HistoryParams::default()
    } else {
        match serde_json::from_value::<HistoryParams>(request.params) {
            Ok(params) => params,
            Err(_) => {
                return Response::error(
                    Some(request.id),
                    ResponseError::new(
                        ERROR_INVALID_PARAMS,
                        "protocol.invalid_params",
                        "Invalid repository graph parameters",
                        false,
                    ),
                );
            }
        }
    };
    let limit = params.limit.unwrap_or(50);
    if !(1..=200).contains(&limit) {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_INVALID_PARAMS,
                "protocol.invalid_params",
                "limit must be between 1 and 200",
                false,
            ),
        );
    }
    let Some(descriptor) = &state.active_repository else {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_REPOSITORY_NOT_OPEN,
                "repository.not_open",
                "Open a repository before requesting the commit graph",
                true,
            ),
        );
    };
    match repository::graph(descriptor, limit, params.cursor.as_deref()) {
        Ok(page) => Response::success(
            request.id,
            serde_json::to_value(page).expect("serializable commit graph page"),
        ),
        Err(error) => Response::error(Some(request.id), repository_error(error)),
    }
}

fn commit_request(request: Request, state: &CoreState) -> Response {
    let params: CommitParams = match serde_json::from_value(request.params) {
        Ok(params) => params,
        Err(_) => {
            return Response::error(
                Some(request.id),
                ResponseError::new(
                    ERROR_INVALID_PARAMS,
                    "protocol.invalid_params",
                    "Invalid commit parameters",
                    false,
                ),
            );
        }
    };
    let Some(descriptor) = &state.active_repository else {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_REPOSITORY_NOT_OPEN,
                "repository.not_open",
                "Open a repository before committing",
                true,
            ),
        );
    };
    match repository::commit(descriptor, &params.message) {
        Ok(result) => Response::success(
            request.id,
            serde_json::to_value(result).expect("serializable commit result"),
        ),
        Err(error) => Response::error(Some(request.id), repository_error(error)),
    }
}

fn branch_request(request: Request, state: &CoreState, switch: bool) -> Response {
    let params: BranchParams = match serde_json::from_value(request.params) {
        Ok(params) => params,
        Err(_) => {
            return Response::error(
                Some(request.id),
                ResponseError::new(
                    ERROR_INVALID_PARAMS,
                    "protocol.invalid_params",
                    "Invalid branch parameters",
                    false,
                ),
            );
        }
    };
    let Some(descriptor) = &state.active_repository else {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_REPOSITORY_NOT_OPEN,
                "repository.not_open",
                "Open a repository before changing branches",
                true,
            ),
        );
    };
    let result = if switch {
        repository::switch_branch(descriptor, &params.name)
    } else {
        repository::create_branch(descriptor, &params.name)
    };
    match result {
        Ok(snapshot) => Response::success(
            request.id,
            serde_json::to_value(snapshot).expect("serializable mutation snapshot"),
        ),
        Err(error) => Response::error(Some(request.id), repository_error(error)),
    }
}

fn repository_fetch_request(request: Request, state: &CoreState) -> Response {
    let params = if request.params.is_null() {
        RepositoryFetchParams::default()
    } else {
        match serde_json::from_value::<RepositoryFetchParams>(request.params) {
            Ok(params) => params,
            Err(_) => return invalid_params(request.id, "Invalid repository fetch parameters"),
        }
    };
    let Some(descriptor) = &state.active_repository else {
        return repository_not_open(request.id, "fetching from a Git remote");
    };
    match repository::fetch(descriptor, params.remote.as_deref()) {
        Ok(result) => Response::success(
            request.id,
            serde_json::to_value(result).expect("serializable repository fetch result"),
        ),
        Err(error) => Response::error(Some(request.id), repository_error(error)),
    }
}

fn repository_sync_request(
    request: Request,
    state: &CoreState,
    operation: RepositorySyncOperation,
) -> Response {
    let params = match serde_json::from_value::<RepositorySyncParams>(request.params) {
        Ok(params)
            if !params.expected_branch.is_empty()
                && params.expected_branch.len() <= 255
                && valid_full_oid(&params.expected_head_oid) =>
        {
            params
        }
        _ => return invalid_params(request.id, "Invalid repository sync parameters"),
    };
    let Some(descriptor) = &state.active_repository else {
        return repository_not_open(request.id, "synchronizing a Git branch");
    };
    let result = match operation {
        RepositorySyncOperation::Pull => repository::pull(
            descriptor,
            &params.expected_branch,
            &params.expected_head_oid,
        ),
        RepositorySyncOperation::Push => repository::push(
            descriptor,
            &params.expected_branch,
            &params.expected_head_oid,
        ),
        RepositorySyncOperation::Fetch => unreachable!("fetch uses its own parameter contract"),
    };
    match result {
        Ok(result) => Response::success(
            request.id,
            serde_json::to_value(result).expect("serializable repository sync result"),
        ),
        Err(error) => Response::error(Some(request.id), repository_error(error)),
    }
}

fn github_repository_request(request: Request, state: &CoreState) -> Response {
    let params = if request.params.is_null() {
        GitHubRepositoryParams::default()
    } else {
        match serde_json::from_value::<GitHubRepositoryParams>(request.params) {
            Ok(params) => params,
            Err(_) => {
                return Response::error(
                    Some(request.id),
                    ResponseError::new(
                        ERROR_INVALID_PARAMS,
                        "protocol.invalid_params",
                        "Invalid GitHub repository parameters",
                        false,
                    ),
                );
            }
        }
    };
    let Some(descriptor) = &state.active_repository else {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_REPOSITORY_NOT_OPEN,
                "repository.not_open",
                "Open a repository before requesting GitHub metadata",
                true,
            ),
        );
    };
    match github::repository(descriptor, &params) {
        Ok(repository) => Response::success(
            request.id,
            serde_json::to_value(repository).expect("serializable GitHub repository"),
        ),
        Err(error) => Response::error(Some(request.id), github_error(error)),
    }
}

fn github_pull_request_request(request: Request, state: &CoreState) -> Response {
    let params = match serde_json::from_value::<GitHubPullRequestParams>(request.params) {
        Ok(params) if params.number > 0 => params,
        _ => {
            return Response::error(
                Some(request.id),
                ResponseError::new(
                    ERROR_INVALID_PARAMS,
                    "protocol.invalid_params",
                    "Invalid GitHub pull request parameters",
                    false,
                ),
            );
        }
    };
    let Some(descriptor) = &state.active_repository else {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_REPOSITORY_NOT_OPEN,
                "repository.not_open",
                "Open a repository before requesting a GitHub pull request",
                true,
            ),
        );
    };
    match github::pull_request(descriptor, &params) {
        Ok(pull_request) => Response::success(
            request.id,
            serde_json::to_value(pull_request).expect("serializable GitHub pull request"),
        ),
        Err(error) => Response::error(Some(request.id), github_error(error)),
    }
}

fn github_pull_request_commit_diff_request(request: Request, state: &CoreState) -> Response {
    let params = match serde_json::from_value::<GitHubPullRequestCommitDiffParams>(request.params) {
        Ok(params) if params.number > 0 && valid_full_oid(&params.oid) => params,
        _ => {
            return Response::error(
                Some(request.id),
                ResponseError::new(
                    ERROR_INVALID_PARAMS,
                    "protocol.invalid_params",
                    "Invalid GitHub pull request commit diff parameters",
                    false,
                ),
            );
        }
    };
    let Some(descriptor) = &state.active_repository else {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_REPOSITORY_NOT_OPEN,
                "repository.not_open",
                "Open a repository before requesting a GitHub pull request commit diff",
                true,
            ),
        );
    };
    match github::pull_request_commit_diff(descriptor, &params) {
        Ok(diff) => Response::success(
            request.id,
            serde_json::to_value(diff).expect("serializable GitHub pull request commit diff"),
        ),
        Err(error) => Response::error(Some(request.id), github_error(error)),
    }
}

fn github_squash_trace_request(request: Request, state: &CoreState) -> Response {
    let params = match serde_json::from_value::<GitHubPullRequestParams>(request.params) {
        Ok(params) if params.number > 0 => params,
        _ => {
            return Response::error(
                Some(request.id),
                ResponseError::new(
                    ERROR_INVALID_PARAMS,
                    "protocol.invalid_params",
                    "Invalid GitHub Squash Trace parameters",
                    false,
                ),
            );
        }
    };
    let Some(descriptor) = &state.active_repository else {
        return Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_REPOSITORY_NOT_OPEN,
                "repository.not_open",
                "Open a repository before requesting a GitHub Squash Trace",
                true,
            ),
        );
    };
    match github::squash_trace(descriptor, &params) {
        Ok(trace) => Response::success(
            request.id,
            serde_json::to_value(trace).expect("serializable GitHub Squash Trace"),
        ),
        Err(error) => Response::error(Some(request.id), squash_trace_error(error)),
    }
}

fn github_commit_squash_trace_request(request: Request, state: &CoreState) -> Response {
    let params = match serde_json::from_value::<GitHubCommitSquashTraceParams>(request.params) {
        Ok(params) if valid_full_oid(&params.oid) => params,
        _ => return invalid_params(request.id, "Invalid GitHub commit Squash Trace parameters"),
    };
    let Some(descriptor) = &state.active_repository else {
        return repository_not_open(request.id, "checking a GitHub commit association");
    };
    match github::commit_squash_trace(descriptor, &params) {
        Ok(trace) => Response::success(
            request.id,
            serde_json::to_value(trace).expect("serializable GitHub commit Squash Trace"),
        ),
        Err(error) => Response::error(Some(request.id), squash_trace_error(error)),
    }
}

fn github_pull_request_commit_files_request(request: Request, state: &mut CoreState) -> Response {
    let params = match serde_json::from_value::<GitHubPullRequestCommitFilesParams>(request.params)
    {
        Ok(params) if params.number > 0 && valid_full_oid(&params.oid) => params,
        _ => {
            return invalid_params(
                request.id,
                "Invalid GitHub original commit files parameters",
            );
        }
    };
    if state.active_repository.is_none() {
        return repository_not_open(request.id, "requesting GitHub original commit files");
    }
    if let Err(error) = ensure_github_commit_diff(state, &params) {
        return Response::error(Some(request.id), github_error(error));
    }
    let cached = &state
        .github_commit_diff_cache
        .as_ref()
        .expect("cache populated")
        .value;
    let result = github::pull_request_commit_files(cached);
    Response::success(
        request.id,
        serde_json::to_value(result).expect("serializable GitHub original commit files"),
    )
}

fn github_pull_request_commit_file_diff_request(
    request: Request,
    state: &mut CoreState,
) -> Response {
    let params =
        match serde_json::from_value::<GitHubPullRequestCommitFileDiffParams>(request.params) {
            Ok(params)
                if params.number > 0
                    && valid_full_oid(&params.oid)
                    && !params.path.is_empty()
                    && params.path.len() <= 4096 =>
            {
                params
            }
            _ => {
                return invalid_params(
                    request.id,
                    "Invalid GitHub original commit file parameters",
                );
            }
        };
    let files_params = GitHubPullRequestCommitFilesParams {
        number: params.number,
        oid: params.oid,
        remote: params.remote,
        name_with_owner: params.name_with_owner,
    };
    if state.active_repository.is_none() {
        return repository_not_open(request.id, "requesting a GitHub original commit file diff");
    }
    if let Err(error) = ensure_github_commit_diff(state, &files_params) {
        return Response::error(Some(request.id), github_error(error));
    }
    let file = github::pull_request_commit_file_diff(
        &state
            .github_commit_diff_cache
            .as_ref()
            .expect("cache populated")
            .value,
        &params.path,
    );
    match file {
        Some(file) => Response::success(
            request.id,
            serde_json::to_value(file).expect("serializable GitHub original commit file diff"),
        ),
        None => invalid_params(
            request.id,
            "Path is not changed by the selected original commit",
        ),
    }
}

fn ensure_github_commit_diff(
    state: &mut CoreState,
    params: &GitHubPullRequestCommitFilesParams,
) -> Result<(), github::GitHubError> {
    let matches = state
        .github_commit_diff_cache
        .as_ref()
        .is_some_and(|cached| {
            cached.value.pull_request_number == params.number
                && cached.value.commit.oid.eq_ignore_ascii_case(&params.oid)
                && cached.remote == params.remote
                && cached.requested_name_with_owner == params.name_with_owner
        });
    if matches {
        return Ok(());
    }
    let descriptor = state
        .active_repository
        .as_ref()
        .ok_or(github::GitHubError::RequestFailed)?;
    let diff_params = GitHubPullRequestCommitDiffParams {
        number: params.number,
        oid: params.oid.clone(),
        remote: params.remote.clone(),
        name_with_owner: params.name_with_owner.clone(),
    };
    let value = github::pull_request_commit_diff(descriptor, &diff_params)?;
    state.github_commit_diff_cache = Some(CachedGitHubCommitDiff {
        remote: params.remote.clone(),
        requested_name_with_owner: params.name_with_owner.clone(),
        value,
    });
    Ok(())
}

fn gitlab_project_request(request: Request, state: &CoreState) -> Response {
    let params = if request.params.is_null() {
        GitLabProjectParams::default()
    } else {
        match serde_json::from_value::<GitLabProjectParams>(request.params) {
            Ok(params) => params,
            Err(_) => return invalid_params(request.id, "Invalid GitLab project parameters"),
        }
    };
    let Some(descriptor) = &state.active_repository else {
        return repository_not_open(request.id, "requesting GitLab metadata");
    };
    match gitlab::project(descriptor, &params) {
        Ok(project) => Response::success(
            request.id,
            serde_json::to_value(project).expect("serializable GitLab project"),
        ),
        Err(error) => Response::error(Some(request.id), gitlab_error(error)),
    }
}

fn gitlab_merge_request_request(request: Request, state: &CoreState) -> Response {
    let params = match serde_json::from_value::<GitLabMergeRequestParams>(request.params) {
        Ok(params) if params.iid > 0 => params,
        _ => return invalid_params(request.id, "Invalid GitLab merge request parameters"),
    };
    let Some(descriptor) = &state.active_repository else {
        return repository_not_open(request.id, "requesting a GitLab merge request");
    };
    match gitlab::merge_request(descriptor, &params) {
        Ok(merge_request) => Response::success(
            request.id,
            serde_json::to_value(merge_request).expect("serializable GitLab merge request"),
        ),
        Err(error) => Response::error(Some(request.id), gitlab_error(error)),
    }
}

fn gitlab_merge_request_commit_diff_request(request: Request, state: &CoreState) -> Response {
    let params = match serde_json::from_value::<GitLabMergeRequestCommitDiffParams>(request.params)
    {
        Ok(params) if params.iid > 0 && valid_full_oid(&params.oid) => params,
        _ => {
            return invalid_params(
                request.id,
                "Invalid GitLab merge request commit diff parameters",
            );
        }
    };
    let Some(descriptor) = &state.active_repository else {
        return repository_not_open(request.id, "requesting a GitLab merge request commit diff");
    };
    match gitlab::merge_request_commit_diff(descriptor, &params) {
        Ok(diff) => Response::success(
            request.id,
            serde_json::to_value(diff).expect("serializable GitLab merge request commit diff"),
        ),
        Err(error) => Response::error(Some(request.id), gitlab_error(error)),
    }
}

fn gitlab_squash_trace_request(request: Request, state: &CoreState) -> Response {
    let params = match serde_json::from_value::<GitLabMergeRequestParams>(request.params) {
        Ok(params) if params.iid > 0 => params,
        _ => return invalid_params(request.id, "Invalid GitLab Squash Trace parameters"),
    };
    let Some(descriptor) = &state.active_repository else {
        return repository_not_open(request.id, "requesting a GitLab Squash Trace");
    };
    match gitlab::squash_trace(descriptor, &params) {
        Ok(trace) => Response::success(
            request.id,
            serde_json::to_value(trace).expect("serializable GitLab Squash Trace"),
        ),
        Err(error) => Response::error(Some(request.id), gitlab_squash_trace_error(error)),
    }
}

fn ai_input_preview_request(request: Request, state: &CoreState) -> Response {
    let params = match serde_json::from_value::<AiInputPreviewParams>(request.params) {
        Ok(params) => params,
        Err(_) => return invalid_params(request.id, "Invalid AI input preview parameters"),
    };
    let Some(descriptor) = &state.active_repository else {
        return repository_not_open(request.id, "previewing AI Assist input");
    };
    match ai::preview(descriptor, &params) {
        Ok(preview) => Response::success(
            request.id,
            serde_json::to_value(preview).expect("serializable AI input preview"),
        ),
        Err(error) => Response::error(Some(request.id), ai_error(error)),
    }
}

fn ai_generate_commit_draft_request(request: Request, state: &CoreState) -> Response {
    let params = match serde_json::from_value::<AiGenerateCommitDraftParams>(request.params) {
        Ok(params) => params,
        Err(_) => return invalid_params(request.id, "Invalid AI commit draft parameters"),
    };
    let Some(descriptor) = &state.active_repository else {
        return repository_not_open(request.id, "generating an AI commit draft");
    };
    match ai::generate(descriptor, &params) {
        Ok(draft) => Response::success(
            request.id,
            serde_json::to_value(draft).expect("serializable AI commit draft"),
        ),
        Err(error) => Response::error(Some(request.id), ai_error(error)),
    }
}

fn ai_error(error: ai::AiError) -> ResponseError {
    match error {
        ai::AiError::WorktreeRequired => ResponseError::new(
            ERROR_WORKTREE_REQUIRED,
            "repository.worktree_required",
            "AI Assist requires a non-bare worktree",
            false,
        ),
        ai::AiError::GitUnavailable => ResponseError::new(
            ERROR_GIT_UNAVAILABLE,
            "git.unavailable",
            "System Git is unavailable",
            true,
        ),
        ai::AiError::GitCommandFailed => ResponseError::new(
            ERROR_GIT_COMMAND_FAILED,
            "git.command_failed",
            "System Git could not inspect staged changes",
            true,
        ),
        ai::AiError::InvalidPath => ResponseError::new(
            ERROR_INVALID_REPOSITORY_PATH,
            "path.invalid_repository_relative",
            "AI exclusion and staged paths must be safe repository-relative paths",
            false,
        ),
        ai::AiError::NothingStaged => ResponseError::new(
            ERROR_AI_NOTHING_STAGED,
            "ai.nothing_staged",
            "AI Assist requires staged changes",
            true,
        ),
        ai::AiError::InvalidProvider => ResponseError::new(
            ERROR_AI_INVALID_PROVIDER,
            "ai.invalid_provider",
            "AI Provider configuration is invalid",
            false,
        ),
        ai::AiError::PreviewStale => ResponseError::new(
            ERROR_AI_PREVIEW_STALE,
            "ai.preview_stale",
            "Staged input or AI Provider configuration changed after preview",
            true,
        ),
        ai::AiError::ExternalConfirmationRequired => ResponseError::new(
            ERROR_AI_EXTERNAL_CONFIRMATION_REQUIRED,
            "ai.external_confirmation_required",
            "External AI disclosure must be confirmed for this preview",
            false,
        ),
        ai::AiError::CredentialMissing => ResponseError::new(
            ERROR_AI_CREDENTIAL_MISSING,
            "ai.credential_missing",
            "The selected AI Provider credential is unavailable in the Core environment",
            true,
        ),
        ai::AiError::ProviderUnavailable => ResponseError::new(
            ERROR_AI_PROVIDER_UNAVAILABLE,
            "ai.provider_unavailable",
            "The selected AI Provider is unavailable",
            true,
        ),
        ai::AiError::RequestFailed => ResponseError::new(
            ERROR_AI_REQUEST_FAILED,
            "ai.request_failed",
            "The AI Provider rejected the request",
            true,
        ),
        ai::AiError::ResponseInvalid => ResponseError::new(
            ERROR_AI_RESPONSE_INVALID,
            "ai.response_invalid",
            "The AI Provider returned an invalid structured response",
            true,
        ),
        ai::AiError::InputLimitExceeded => ResponseError::new(
            ERROR_AI_INPUT_LIMIT,
            "ai.input_limit_exceeded",
            "Staged AI input exceeds the safe disclosure limits",
            false,
        ),
    }
}

fn invalid_params(id: gitnova_protocol::RequestId, message: &str) -> Response {
    Response::error(
        Some(id),
        ResponseError::new(
            ERROR_INVALID_PARAMS,
            "protocol.invalid_params",
            message,
            false,
        ),
    )
}

fn repository_not_open(id: gitnova_protocol::RequestId, operation: &str) -> Response {
    Response::error(
        Some(id),
        ResponseError::new(
            ERROR_REPOSITORY_NOT_OPEN,
            "repository.not_open",
            &format!("Open a repository before {operation}"),
            true,
        ),
    )
}

fn valid_full_oid(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn dispatch_notification(
    notification: Notification,
    lifecycle: Lifecycle,
    cancellations: &CancellationRegistry,
) -> bool {
    match notification.method.as_str() {
        "$/cancelRequest" => {
            if let Ok(params) = serde_json::from_value::<CancelParams>(notification.params) {
                cancellations.cancel(params.id);
            }
            false
        }
        "exit" => true,
        _ => {
            let _ = lifecycle;
            false
        }
    }
}

fn major_version(version: &str) -> Option<&str> {
    version.split_once('.').map(|(major, _)| major)
}

fn write_response(writer: &mut impl Write, response: &Response) -> io::Result<()> {
    let body = serde_json::to_vec(response).map_err(io::Error::other)?;
    framing::write_frame(writer, &body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitnova_protocol::RequestId;
    use serde_json::json;

    fn request(method: &str, params: Value) -> Request {
        Request {
            jsonrpc: JSON_RPC_VERSION.into(),
            id: RequestId::Number(1),
            method: method.into(),
            params,
        }
    }

    #[test]
    fn incompatible_major_version_does_not_initialize_core() {
        let mut state = CoreState::default();
        let response = dispatch_request(
            request(
                "gitnova/initialize",
                json!({
                    "clientInfo": {"name": "test", "version": "1"},
                    "protocolVersion": "2.0",
                    "capabilities": {}
                }),
            ),
            &mut state,
            &CancellationRegistry::default(),
        );
        assert_eq!(
            response.error.expect("error response").code,
            ERROR_INCOMPATIBLE_PROTOCOL
        );
        assert_eq!(state.lifecycle, Lifecycle::Uninitialized);
    }

    #[test]
    fn cancelled_request_returns_stable_error() {
        let registry = CancellationRegistry::default();
        registry.cancel(RequestId::Number(1));
        let mut state = CoreState {
            lifecycle: Lifecycle::Initialized,
            active_repository: None,
            github_commit_diff_cache: None,
        };
        let response = dispatch_request(request("unknown", json!({})), &mut state, &registry);
        let error = response.error.expect("error response");
        assert_eq!(error.code, ERROR_REQUEST_CANCELLED);
        assert_eq!(error.data.stable_code, "request.cancelled");
    }
}
