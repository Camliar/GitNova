mod framing;

use gitnova_protocol::{
    CancelParams, CancellationRegistry, ERROR_ALREADY_INITIALIZED, ERROR_INCOMPATIBLE_PROTOCOL,
    ERROR_INVALID_PARAMS, ERROR_INVALID_REQUEST, ERROR_METHOD_NOT_FOUND, ERROR_NOT_INITIALIZED,
    ERROR_PARSE, ERROR_REQUEST_CANCELLED, ImplementationInfo, InitializeParams, InitializeResult,
    JSON_RPC_VERSION, Notification, PROTOCOL_VERSION, Request, Response, ResponseError,
    ServerCapabilities,
};
use serde_json::Value;
use std::io::{self, BufRead, Write};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Lifecycle {
    Uninitialized,
    Initialized,
    Shutdown,
}

pub fn run(reader: &mut impl BufRead, writer: &mut impl Write) -> io::Result<i32> {
    let mut lifecycle = Lifecycle::Uninitialized;
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
            let response = dispatch_request(request, &mut lifecycle, &cancellations);
            write_response(writer, &response)?;
        } else {
            let notification: Notification = match serde_json::from_value::<Notification>(value) {
                Ok(notification) if notification.jsonrpc == JSON_RPC_VERSION => notification,
                _ => continue,
            };
            if dispatch_notification(notification, lifecycle, &cancellations) {
                return Ok(if lifecycle == Lifecycle::Shutdown {
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
    lifecycle: &mut Lifecycle,
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
        "gitnova/initialize" => initialize(request, lifecycle),
        "gitnova/shutdown" => shutdown(request, lifecycle),
        _ if *lifecycle == Lifecycle::Uninitialized => Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_NOT_INITIALIZED,
                "core.not_initialized",
                "Core must be initialized before handling this request",
                true,
            ),
        ),
        _ if *lifecycle == Lifecycle::Shutdown => Response::error(
            Some(request.id),
            ResponseError::new(
                ERROR_INVALID_REQUEST,
                "core.already_shutdown",
                "Core has already shut down",
                false,
            ),
        ),
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

fn initialize(request: Request, lifecycle: &mut Lifecycle) -> Response {
    if *lifecycle != Lifecycle::Uninitialized {
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

    *lifecycle = Lifecycle::Initialized;
    let result = InitializeResult {
        core_info: ImplementationInfo {
            name: "gitnova-core".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        },
        protocol_version: PROTOCOL_VERSION.into(),
        capabilities: ServerCapabilities { cancellation: true },
    };
    Response::success(
        request.id,
        serde_json::to_value(result).expect("serializable result"),
    )
}

fn shutdown(request: Request, lifecycle: &mut Lifecycle) -> Response {
    if *lifecycle == Lifecycle::Uninitialized {
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
    if *lifecycle == Lifecycle::Shutdown {
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
    *lifecycle = Lifecycle::Shutdown;
    Response::success(request.id, Value::Null)
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
        let mut lifecycle = Lifecycle::Uninitialized;
        let response = dispatch_request(
            request(
                "gitnova/initialize",
                json!({
                    "clientInfo": {"name": "test", "version": "1"},
                    "protocolVersion": "2.0",
                    "capabilities": {}
                }),
            ),
            &mut lifecycle,
            &CancellationRegistry::default(),
        );
        assert_eq!(
            response.error.expect("error response").code,
            ERROR_INCOMPATIBLE_PROTOCOL
        );
        assert_eq!(lifecycle, Lifecycle::Uninitialized);
    }

    #[test]
    fn cancelled_request_returns_stable_error() {
        let registry = CancellationRegistry::default();
        registry.cancel(RequestId::Number(1));
        let mut lifecycle = Lifecycle::Initialized;
        let response = dispatch_request(request("unknown", json!({})), &mut lifecycle, &registry);
        let error = response.error.expect("error response");
        assert_eq!(error.code, ERROR_REQUEST_CANCELLED);
        assert_eq!(error.data.stable_code, "request.cancelled");
    }
}
