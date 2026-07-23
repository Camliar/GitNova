mod framing;
mod repository;

use gitnova_protocol::{
    CancelParams, CancellationRegistry, CommitDiffParams, DiffParams, ERROR_ALREADY_INITIALIZED,
    ERROR_COMMIT_DIFF_PARSE, ERROR_COMMIT_NOT_FOUND, ERROR_COMMIT_PARENT_REQUIRED,
    ERROR_COMMIT_PARSE, ERROR_DIFF_PARSE, ERROR_DIFFERENT_REPOSITORY_OPEN,
    ERROR_GIT_COMMAND_FAILED, ERROR_GIT_UNAVAILABLE, ERROR_HISTORY_ENCODING,
    ERROR_INCOMPATIBLE_PROTOCOL, ERROR_INVALID_COMMIT_PARENT, ERROR_INVALID_HISTORY_CURSOR,
    ERROR_INVALID_PARAMS, ERROR_INVALID_PATH, ERROR_INVALID_REPOSITORY_PATH, ERROR_INVALID_REQUEST,
    ERROR_METHOD_NOT_FOUND, ERROR_NOT_INITIALIZED, ERROR_PARSE, ERROR_REFERENCE_ENCODING,
    ERROR_REFERENCE_PARSE, ERROR_REPOSITORY_NOT_FOUND, ERROR_REPOSITORY_NOT_OPEN,
    ERROR_REQUEST_CANCELLED, ERROR_STATUS_PARSE, ERROR_UNSAFE_REPOSITORY, ERROR_WORKTREE_REQUIRED,
    HistoryParams, ImplementationInfo, InitializeParams, InitializeResult, JSON_RPC_VERSION,
    Notification, PROTOCOL_VERSION, RepositoryDescriptor, RepositoryPathParams, Request, Response,
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
}

impl Default for CoreState {
    fn default() -> Self {
        Self {
            lifecycle: Lifecycle::Uninitialized,
            active_repository: None,
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
        "repository/references" => references_request(request, state),
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
            repository_references: true,
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
        };
        let response = dispatch_request(request("unknown", json!({})), &mut state, &registry);
        let error = response.error.expect("error response");
        assert_eq!(error.code, ERROR_REQUEST_CANCELLED);
        assert_eq!(error.data.stable_code, "request.cancelled");
    }
}
