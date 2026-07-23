mod framing;
mod repository;

use gitnova_protocol::{
    CancelParams, CancellationRegistry, ERROR_ALREADY_INITIALIZED, ERROR_DIFFERENT_REPOSITORY_OPEN,
    ERROR_GIT_COMMAND_FAILED, ERROR_GIT_UNAVAILABLE, ERROR_INCOMPATIBLE_PROTOCOL,
    ERROR_INVALID_PARAMS, ERROR_INVALID_PATH, ERROR_INVALID_REQUEST, ERROR_METHOD_NOT_FOUND,
    ERROR_NOT_INITIALIZED, ERROR_PARSE, ERROR_REPOSITORY_NOT_FOUND, ERROR_REQUEST_CANCELLED,
    ERROR_UNSAFE_REPOSITORY, ImplementationInfo, InitializeParams, InitializeResult,
    JSON_RPC_VERSION, Notification, PROTOCOL_VERSION, RepositoryDescriptor, RepositoryPathParams,
    Request, Response, ResponseError, ServerCapabilities,
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
