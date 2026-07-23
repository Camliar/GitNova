use gitnova_protocol::{
    ClientCapabilities, ImplementationInfo, InitializeParams, InitializeResult, Notification,
    PROTOCOL_VERSION, Request, RequestId, Response, ServerCapabilities,
};
use serde::Serialize;
use serde_json::Value;
use std::env;
use std::ffi::OsString;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(15);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopError {
    pub code: &'static str,
    pub message: &'static str,
    pub retryable: bool,
}

impl DesktopError {
    fn new(code: &'static str, message: &'static str, retryable: bool) -> Self {
        Self {
            code,
            message,
            retryable,
        }
    }

    fn not_running() -> Self {
        Self::new(
            "desktop.core_not_running",
            "GitNova Core is not running",
            true,
        )
    }

    fn transport() -> Self {
        Self::new(
            "desktop.core_transport_failed",
            "GitNova Core transport failed",
            true,
        )
    }

    fn protocol() -> Self {
        Self::new(
            "desktop.core_protocol_failed",
            "GitNova Core returned an invalid protocol response",
            false,
        )
    }

    pub fn host_task_failed() -> Self {
        Self::new("desktop.host_task_failed", "Desktop Host task failed", true)
    }
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreStatus {
    pub connected: bool,
    pub protocol_version: Option<String>,
    pub capabilities: Option<ServerCapabilities>,
}

pub struct CoreSupervisor {
    command: CoreCommand,
    process: Mutex<Option<CoreProcess>>,
    status: Mutex<CoreStatus>,
}

#[derive(Clone)]
struct CoreCommand {
    program: PathBuf,
    arguments: Vec<OsString>,
}

struct CoreProcess {
    child: Child,
    stdin: BufWriter<ChildStdin>,
    responses: Receiver<Result<ReceivedResponse, DesktopError>>,
    next_id: i64,
}

struct ReceivedResponse {
    response: Response,
    raw: Value,
    has_result: bool,
    has_error: bool,
}

impl CoreSupervisor {
    pub fn discover() -> Result<Self, DesktopError> {
        Ok(Self::new(CoreCommand {
            program: resolve_core_binary()?,
            arguments: Vec::new(),
        }))
    }

    fn new(command: CoreCommand) -> Self {
        Self {
            command,
            process: Mutex::new(None),
            status: Mutex::new(CoreStatus::default()),
        }
    }

    pub fn status(&self) -> CoreStatus {
        self.status
            .lock()
            .expect("Core status mutex poisoned")
            .clone()
    }

    pub fn start(&self) -> Result<CoreStatus, DesktopError> {
        let mut process = self.process.lock().map_err(|_| DesktopError::transport())?;
        if process.is_some() {
            return Ok(self.status());
        }
        let mut child = spawn_core(&self.command)?;
        let stdin = child.stdin.take().ok_or_else(DesktopError::transport)?;
        let stdout = child.stdout.take().ok_or_else(DesktopError::transport)?;
        let stderr = child.stderr.take().ok_or_else(DesktopError::transport)?;
        thread::spawn(move || {
            let _ = io::copy(&mut BufReader::new(stderr), &mut io::sink());
        });
        let (sender, responses) = mpsc::channel();
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let response = read_frame(&mut reader)
                    .and_then(|frame| {
                        frame.ok_or_else(|| {
                            io::Error::new(io::ErrorKind::UnexpectedEof, "Core stdout closed")
                        })
                    })
                    .and_then(|frame| {
                        serde_json::from_slice::<Value>(&frame).map_err(io::Error::other)
                    })
                    .and_then(|value| {
                        let has_result = value.get("result").is_some();
                        let has_error = value.get("error").is_some();
                        serde_json::from_value::<Response>(value.clone())
                            .map(|response| ReceivedResponse {
                                response,
                                raw: value,
                                has_result,
                                has_error,
                            })
                            .map_err(io::Error::other)
                    })
                    .map_err(|_| DesktopError::transport());
                let finished = response.is_err();
                if sender.send(response).is_err() || finished {
                    break;
                }
            }
        });
        let mut candidate = CoreProcess {
            child,
            stdin: BufWriter::new(stdin),
            responses,
            next_id: 1,
        };
        let initialize = InitializeParams {
            client_info: ImplementationInfo {
                name: "gitnova-desktop".into(),
                version: env!("CARGO_PKG_VERSION").into(),
            },
            protocol_version: PROTOCOL_VERSION.into(),
            capabilities: ClientCapabilities { cancellation: true },
        };
        let response = candidate.request(
            "gitnova/initialize",
            serde_json::to_value(initialize).map_err(|_| DesktopError::protocol())?,
        )?;
        if let Some(error) = response.response.error {
            let _ = error;
            return Err(DesktopError::new(
                "desktop.core_initialize_failed",
                "GitNova Core initialization failed",
                false,
            ));
        }
        let result: InitializeResult = serde_json::from_value(
            response
                .response
                .result
                .ok_or_else(DesktopError::protocol)?,
        )
        .map_err(|_| DesktopError::protocol())?;
        validate_initialize(&result)?;
        let status = CoreStatus {
            connected: true,
            protocol_version: Some(result.protocol_version),
            capabilities: Some(result.capabilities),
        };
        *self.status.lock().map_err(|_| DesktopError::transport())? = status.clone();
        *process = Some(candidate);
        Ok(status)
    }

    pub fn request(&self, method: &str, params: Value) -> Result<Value, DesktopError> {
        if !valid_method(method)
            || matches!(method, "gitnova/initialize" | "gitnova/shutdown" | "exit")
        {
            return Err(DesktopError::new(
                "desktop.invalid_core_method",
                "Core method is invalid",
                false,
            ));
        }
        let mut guard = self.process.lock().map_err(|_| DesktopError::transport())?;
        let Some(process) = guard.as_mut() else {
            return Err(DesktopError::not_running());
        };
        match process.request(method, params) {
            Ok(response) => Ok(response.raw),
            Err(error) => {
                process.terminate();
                *guard = None;
                *self.status.lock().map_err(|_| DesktopError::transport())? = CoreStatus::default();
                Err(error)
            }
        }
    }

    pub fn shutdown(&self) -> Result<CoreStatus, DesktopError> {
        let mut process = self.process.lock().map_err(|_| DesktopError::transport())?;
        let result = process.take().map_or(Ok(()), |mut child| child.shutdown());
        let status = CoreStatus::default();
        *self.status.lock().map_err(|_| DesktopError::transport())? = status.clone();
        result.map(|()| status)
    }
}

impl Drop for CoreSupervisor {
    fn drop(&mut self) {
        if let Ok(process) = self.process.get_mut()
            && let Some(process) = process.as_mut()
        {
            process.terminate();
        }
    }
}

impl CoreProcess {
    fn request(&mut self, method: &str, params: Value) -> Result<ReceivedResponse, DesktopError> {
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or_else(DesktopError::protocol)?;
        let request = Request {
            jsonrpc: "2.0".into(),
            id: RequestId::Number(id),
            method: method.into(),
            params,
        };
        write_frame(
            &mut self.stdin,
            &serde_json::to_vec(&request).map_err(|_| DesktopError::protocol())?,
        )
        .map_err(|_| DesktopError::transport())?;
        let received = self
            .responses
            .recv_timeout(RESPONSE_TIMEOUT)
            .map_err(|_| DesktopError::transport())??;
        if received.response.jsonrpc != "2.0"
            || received.response.id != Some(RequestId::Number(id))
            || received.has_result == received.has_error
        {
            return Err(DesktopError::protocol());
        }
        Ok(received)
    }

    fn shutdown(&mut self) -> Result<(), DesktopError> {
        self.request("gitnova/shutdown", Value::Null)?;
        let notification = Notification {
            jsonrpc: "2.0".into(),
            method: "exit".into(),
            params: Value::Null,
        };
        write_frame(
            &mut self.stdin,
            &serde_json::to_vec(&notification).map_err(|_| DesktopError::protocol())?,
        )
        .map_err(|_| DesktopError::transport())?;
        let deadline = Instant::now() + SHUTDOWN_TIMEOUT;
        loop {
            match self.child.try_wait() {
                Ok(Some(status)) if status.success() => return Ok(()),
                Ok(Some(_)) => {
                    return Err(DesktopError::new(
                        "desktop.core_shutdown_failed",
                        "GitNova Core exited unsuccessfully",
                        true,
                    ));
                }
                Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
                _ => {
                    self.terminate();
                    return Err(DesktopError::new(
                        "desktop.core_shutdown_failed",
                        "GitNova Core did not shut down",
                        true,
                    ));
                }
            }
        }
    }

    fn terminate(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for CoreProcess {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            self.terminate();
        }
    }
}

fn spawn_core(command: &CoreCommand) -> Result<Child, DesktopError> {
    Command::new(&command.program)
        .args(&command.arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| match error.kind() {
            io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied => DesktopError::new(
                "desktop.core_unavailable",
                "GitNova Core executable is unavailable",
                true,
            ),
            _ => DesktopError::new(
                "desktop.core_spawn_failed",
                "GitNova Core could not be started",
                true,
            ),
        })
}

fn resolve_core_binary() -> Result<PathBuf, DesktopError> {
    if cfg!(debug_assertions)
        && let Some(path) = env::var_os("GITNOVA_CORE_BINARY")
    {
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            return Err(DesktopError::new(
                "desktop.invalid_core_path",
                "GitNova Core path must be absolute",
                false,
            ));
        }
        return Ok(path);
    }
    let executable = env::current_exe().map_err(|_| {
        DesktopError::new(
            "desktop.core_path_failed",
            "GitNova Core path could not be resolved",
            false,
        )
    })?;
    let parent = executable.parent().ok_or_else(|| {
        DesktopError::new(
            "desktop.core_path_failed",
            "GitNova Core path could not be resolved",
            false,
        )
    })?;
    Ok(parent.join(format!("gitnova-core{}", env::consts::EXE_SUFFIX)))
}

fn validate_initialize(result: &InitializeResult) -> Result<(), DesktopError> {
    if major_version(&result.protocol_version) != major_version(PROTOCOL_VERSION) {
        return Err(DesktopError::new(
            "desktop.core_incompatible",
            "GitNova Core protocol is incompatible",
            false,
        ));
    }
    let capabilities = &result.capabilities;
    if !capabilities.repository_discovery
        || !capabilities.github_pull_request_commit_diff
        || !capabilities.github_squash_trace
    {
        return Err(DesktopError::new(
            "desktop.core_capability_missing",
            "GitNova Core is missing required capabilities",
            false,
        ));
    }
    Ok(())
}

fn major_version(version: &str) -> Option<&str> {
    version.split_once('.').map(|(major, _)| major)
}

fn valid_method(method: &str) -> bool {
    !method.is_empty()
        && method.len() <= 128
        && method
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'_' | b'$'))
}

fn write_frame(writer: &mut impl Write, body: &[u8]) -> io::Result<()> {
    if body.len() > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame too large",
        ));
    }
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(body)?;
    writer.flush()
}

fn read_frame(reader: &mut impl BufRead) -> io::Result<Option<Vec<u8>>> {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            return if content_length.is_none() {
                Ok(None)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "incomplete headers",
                ))
            };
        }
        if line == "\r\n" {
            break;
        }
        let (name, value) = line
            .trim_end_matches(['\r', '\n'])
            .split_once(':')
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid header"))?;
        if name.eq_ignore_ascii_case("Content-Length") {
            if content_length.is_some() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "duplicate length",
                ));
            }
            let length = value
                .trim()
                .parse::<usize>()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid length"))?;
            if length > MAX_FRAME_BYTES {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "frame too large",
                ));
            }
            content_length = Some(length);
        }
    }
    let length = content_length
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing length"))?;
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    Ok(Some(body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const FAKE_CORE: &str = r#"
let buffer = Buffer.alloc(0);
process.stdin.on('data', chunk => { buffer = Buffer.concat([buffer, chunk]); drain(); });
function send(value) {
  const body = Buffer.from(JSON.stringify(value));
  process.stdout.write(`Content-Length: ${body.length}\r\n\r\n`);
  process.stdout.write(body);
}
function drain() {
  for (;;) {
    const marker = buffer.indexOf('\r\n\r\n');
    if (marker < 0) return;
    const header = buffer.subarray(0, marker).toString();
    const match = /^Content-Length:\s*(\d+)$/i.exec(header);
    if (!match) process.exit(2);
    const length = Number(match[1]);
    if (buffer.length < marker + 4 + length) return;
    const request = JSON.parse(buffer.subarray(marker + 4, marker + 4 + length));
    buffer = buffer.subarray(marker + 4 + length);
    if (request.method === 'exit') process.exit(0);
    if (request.method === 'gitnova/initialize') {
      send({jsonrpc:'2.0', id:request.id, result:{
        coreInfo:{name:'fake-core',version:'0.1.0'}, protocolVersion:'1.11', capabilities:{
          cancellation:true, repositoryDiscovery:true, workingTreeStatus:true,
          structuredFileDiff:true, paginatedCommitHistory:true, structuredCommitDiff:true,
          repositoryReferences:true, commitGraphProjection:true, githubRepository:true,
          githubPullRequest:true, githubPullRequestCommitDiff:true, githubSquashTrace:true
        }
      }});
    } else if (request.method === 'gitnova/shutdown') {
      send({jsonrpc:'2.0', id:request.id, result:null});
    } else {
      send({jsonrpc:'2.0', id:request.id, result:{method:request.method, params:request.params}});
    }
  }
}
"#;

    #[test]
    fn framing_round_trips_and_rejects_invalid_lengths() {
        let mut bytes = Vec::new();
        write_frame(&mut bytes, br#"{"jsonrpc":"2.0"}"#).unwrap();
        assert_eq!(
            read_frame(&mut Cursor::new(bytes)).unwrap().unwrap(),
            br#"{"jsonrpc":"2.0"}"#
        );
        assert!(
            read_frame(&mut Cursor::new(
                b"Content-Length: 1\r\nContent-Length: 1\r\n\r\nx"
            ))
            .is_err()
        );
        assert!(
            read_frame(&mut Cursor::new(format!(
                "Content-Length: {}\r\n\r\n",
                MAX_FRAME_BYTES + 1
            )))
            .is_err()
        );
        assert!(read_frame(&mut Cursor::new(b"Content-Length: 4\r\n\r\n{}".as_slice())).is_err());
    }

    #[test]
    fn validates_method_and_protocol_requirements() {
        assert!(valid_method("repository/open"));
        assert!(!valid_method("repository/open?path=secret"));
        assert_eq!(major_version("1.11"), Some("1"));
        assert_eq!(major_version("invalid"), None);
    }

    #[test]
    fn supervises_initialize_request_and_graceful_shutdown() {
        let supervisor = CoreSupervisor::new(CoreCommand {
            program: PathBuf::from("node"),
            arguments: vec![OsString::from("-e"), OsString::from(FAKE_CORE)],
        });
        let status = supervisor.start().unwrap();
        assert!(status.connected);
        assert_eq!(status.protocol_version.as_deref(), Some("1.11"));
        assert!(status.capabilities.unwrap().github_squash_trace);

        let response = supervisor
            .request("test/echo", serde_json::json!({"safe": true}))
            .unwrap();
        assert_eq!(response["result"]["method"], "test/echo");
        assert!(!supervisor.shutdown().unwrap().connected);
    }
}
