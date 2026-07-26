use crate::diagnostics::DiagnosticLog;
use gitnova_protocol::{
    ClientCapabilities, ImplementationInfo, InitializeParams, InitializeResult, Notification,
    PROTOCOL_VERSION, Request, RequestId, Response, ServerCapabilities,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::ffi::OsString;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(15);
const PROVIDER_RESPONSE_TIMEOUT: Duration = Duration::from_secs(45);
const AI_RESPONSE_TIMEOUT: Duration = Duration::from_secs(75);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2);
const HOST_CORE_METHODS: &[&str] = &[
    "repository/open",
    "repository/status",
    "repository/diff",
    "repository/graph",
    "repository/commitDiff",
    "repository/commitFiles",
    "repository/commitFileDiff",
    "repository/references",
    "repository/commit",
    "repository/createBranch",
    "repository/switchBranch",
    "repository/checkoutRemoteBranch",
    "repository/fetch",
    "repository/pull",
    "repository/push",
    "github/repository",
    "github/pullRequest",
    "github/pullRequestCommitDiff",
    "github/squashTrace",
    "github/commitSquashTrace",
    "github/pullRequestCommitFiles",
    "github/pullRequestCommitFileDiff",
    "ai/inputPreview",
    "ai/generateCommitDraft",
];

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

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CoreEnvironment {
    #[default]
    Local,
    Wsl,
    Ssh,
    DevContainer,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CoreLaunchTarget {
    Local,
    Wsl { distribution: String },
    Ssh { destination: String },
    DevContainer { workspace_folder: String },
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreStatus {
    pub connected: bool,
    pub protocol_version: Option<String>,
    pub capabilities: Option<ServerCapabilities>,
    pub environment: CoreEnvironment,
}

pub struct CoreSupervisor {
    command: Mutex<CoreCommand>,
    process: Mutex<Option<CoreProcess>>,
    status: Mutex<CoreStatus>,
    diagnostics: Option<Arc<DiagnosticLog>>,
}

#[derive(Clone)]
struct CoreCommand {
    program: PathBuf,
    arguments: Vec<OsString>,
    environment: CoreEnvironment,
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
    pub fn discover(diagnostics: Arc<DiagnosticLog>) -> Result<Self, DesktopError> {
        Ok(Self::with_diagnostics(
            command_for_target(CoreLaunchTarget::Local)?,
            Some(diagnostics),
        ))
    }

    #[cfg(test)]
    fn new(command: CoreCommand) -> Self {
        Self::with_diagnostics(command, None)
    }

    fn with_diagnostics(command: CoreCommand, diagnostics: Option<Arc<DiagnosticLog>>) -> Self {
        Self {
            command: Mutex::new(command),
            process: Mutex::new(None),
            status: Mutex::new(CoreStatus::default()),
            diagnostics,
        }
    }

    pub fn status(&self) -> CoreStatus {
        self.status
            .lock()
            .expect("Core status mutex poisoned")
            .clone()
    }

    pub fn configure(&self, target: CoreLaunchTarget) -> Result<CoreStatus, DesktopError> {
        let environment = launch_target_environment(&target);
        let result = self.configure_inner(target);
        if let Some(diagnostics) = &self.diagnostics {
            let error_code = result.as_ref().err().map(|error| error.code);
            diagnostics.core_configured(
                environment_label(environment),
                if error_code.is_some() {
                    "error"
                } else {
                    "success"
                },
                error_code,
            );
        }
        result
    }

    fn configure_inner(&self, target: CoreLaunchTarget) -> Result<CoreStatus, DesktopError> {
        if self
            .process
            .lock()
            .map_err(|_| DesktopError::transport())?
            .is_some()
        {
            return Err(DesktopError::new(
                "desktop.core_already_running",
                "Stop GitNova Core before changing its environment",
                false,
            ));
        }
        let command = command_for_target(target)?;
        let environment = command.environment;
        *self.command.lock().map_err(|_| DesktopError::transport())? = command;
        let status = CoreStatus {
            environment,
            ..CoreStatus::default()
        };
        *self.status.lock().map_err(|_| DesktopError::transport())? = status.clone();
        Ok(status)
    }

    pub fn start(&self) -> Result<CoreStatus, DesktopError> {
        let started = Instant::now();
        let environment = self.current_environment();
        let result = self.start_inner();
        if let Some(diagnostics) = &self.diagnostics {
            let error_code = result.as_ref().err().map(|error| error.code);
            let protocol_version = result
                .as_ref()
                .ok()
                .and_then(|status| status.protocol_version.as_deref())
                .and_then(safe_diagnostic_version);
            diagnostics.core_started(
                environment_label(environment),
                elapsed_ms(started),
                if error_code.is_some() {
                    "error"
                } else {
                    "success"
                },
                error_code,
                protocol_version,
            );
        }
        result
    }

    fn start_inner(&self) -> Result<CoreStatus, DesktopError> {
        let mut process = self.process.lock().map_err(|_| DesktopError::transport())?;
        if process.is_some() {
            return Ok(self.status());
        }
        let command = self
            .command
            .lock()
            .map_err(|_| DesktopError::transport())?
            .clone();
        let mut child = spawn_core(&command)?;
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
            environment: command.environment,
        };
        *self.status.lock().map_err(|_| DesktopError::transport())? = status.clone();
        *process = Some(candidate);
        Ok(status)
    }

    pub fn request(&self, method: &str, params: Value) -> Result<Value, DesktopError> {
        let started = Instant::now();
        let environment = self.current_environment();
        let safe_method = allowed_host_method(method).then_some(method);
        let result = self.request_inner(method, params);
        if let Some(diagnostics) = &self.diagnostics {
            let desktop_error = result.as_ref().err().map(|error| error.code);
            let has_core_error = result
                .as_ref()
                .ok()
                .is_some_and(|response| response.get("error").is_some());
            let core_error = result.as_ref().ok().and_then(|response| {
                response
                    .pointer("/error/data/stableCode")
                    .and_then(Value::as_str)
                    .and_then(safe_diagnostic_error_code)
            });
            let error_code = desktop_error.or(core_error);
            let outcome = if desktop_error.is_some() {
                "transport_error"
            } else if has_core_error {
                "core_error"
            } else {
                "success"
            };
            diagnostics.core_request(
                environment_label(environment),
                safe_method,
                elapsed_ms(started),
                outcome,
                error_code,
            );
        }
        result
    }

    fn request_inner(&self, method: &str, params: Value) -> Result<Value, DesktopError> {
        if !allowed_host_method(method) {
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
                let environment = self
                    .command
                    .lock()
                    .map_err(|_| DesktopError::transport())?
                    .environment;
                *self.status.lock().map_err(|_| DesktopError::transport())? = CoreStatus {
                    environment,
                    ..CoreStatus::default()
                };
                Err(error)
            }
        }
    }

    pub fn shutdown(&self) -> Result<CoreStatus, DesktopError> {
        let started = Instant::now();
        let environment = self.current_environment();
        let result = self.shutdown_inner();
        if let Some(diagnostics) = &self.diagnostics {
            let error_code = result.as_ref().err().map(|error| error.code);
            diagnostics.core_shutdown(
                environment_label(environment),
                elapsed_ms(started),
                if error_code.is_some() {
                    "error"
                } else {
                    "success"
                },
                error_code,
            );
        }
        result
    }

    fn shutdown_inner(&self) -> Result<CoreStatus, DesktopError> {
        let mut process = self.process.lock().map_err(|_| DesktopError::transport())?;
        let result = process.take().map_or(Ok(()), |mut child| child.shutdown());
        let environment = self
            .command
            .lock()
            .map_err(|_| DesktopError::transport())?
            .environment;
        let status = CoreStatus {
            environment,
            ..CoreStatus::default()
        };
        *self.status.lock().map_err(|_| DesktopError::transport())? = status.clone();
        result.map(|()| status)
    }

    fn current_environment(&self) -> CoreEnvironment {
        self.command
            .lock()
            .map_or(CoreEnvironment::Local, |command| command.environment)
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
        let timeout = if method == "ai/generateCommitDraft" {
            AI_RESPONSE_TIMEOUT
        } else if method.starts_with("github/")
            || method.starts_with("gitlab/")
            || matches!(
                method,
                "repository/fetch" | "repository/pull" | "repository/push"
            )
        {
            PROVIDER_RESPONSE_TIMEOUT
        } else {
            RESPONSE_TIMEOUT
        };
        let received = self
            .responses
            .recv_timeout(timeout)
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
    let mut child = Command::new(&command.program);
    child
        .args(&command.arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(path) =
        projected_local_core_path(command.environment, env::var_os("PATH").as_deref())
    {
        child.env("PATH", path);
    }
    child.spawn().map_err(|error| match error.kind() {
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

const fn launch_target_environment(target: &CoreLaunchTarget) -> CoreEnvironment {
    match target {
        CoreLaunchTarget::Local => CoreEnvironment::Local,
        CoreLaunchTarget::Wsl { .. } => CoreEnvironment::Wsl,
        CoreLaunchTarget::Ssh { .. } => CoreEnvironment::Ssh,
        CoreLaunchTarget::DevContainer { .. } => CoreEnvironment::DevContainer,
    }
}

const fn environment_label(environment: CoreEnvironment) -> &'static str {
    match environment {
        CoreEnvironment::Local => "local",
        CoreEnvironment::Wsl => "wsl",
        CoreEnvironment::Ssh => "ssh",
        CoreEnvironment::DevContainer => "devContainer",
    }
}

fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn safe_diagnostic_error_code(value: &str) -> Option<&str> {
    (!value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_')
        }))
    .then_some(value)
}

fn safe_diagnostic_version(value: &str) -> Option<&str> {
    (!value.is_empty()
        && value.len() <= 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+')))
    .then_some(value)
}

fn projected_local_core_path(
    environment: CoreEnvironment,
    inherited: Option<&std::ffi::OsStr>,
) -> Option<OsString> {
    #[cfg(target_os = "macos")]
    {
        if environment != CoreEnvironment::Local {
            return None;
        }
        let mut entries: Vec<PathBuf> = inherited
            .map(env::split_paths)
            .into_iter()
            .flatten()
            .collect();
        for path in [
            "/opt/homebrew/bin",
            "/usr/local/bin",
            "/opt/local/bin",
            "/usr/bin",
            "/bin",
            "/usr/sbin",
            "/sbin",
        ] {
            let path = PathBuf::from(path);
            if !entries.contains(&path) {
                entries.push(path);
            }
        }
        env::join_paths(entries).ok()
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (environment, inherited);
        None
    }
}

fn command_for_target(target: CoreLaunchTarget) -> Result<CoreCommand, DesktopError> {
    let invalid = || {
        DesktopError::new(
            "desktop.invalid_core_environment",
            "Core environment configuration is invalid",
            false,
        )
    };
    match target {
        CoreLaunchTarget::Local => Ok(CoreCommand {
            program: resolve_core_binary()?,
            arguments: Vec::new(),
            environment: CoreEnvironment::Local,
        }),
        CoreLaunchTarget::Wsl { distribution } => {
            if !valid_identifier(&distribution, 64) {
                return Err(invalid());
            }
            Ok(CoreCommand {
                program: PathBuf::from("wsl.exe"),
                arguments: vec![
                    "--distribution".into(),
                    distribution.into(),
                    "--exec".into(),
                    "gitnova-core".into(),
                ],
                environment: CoreEnvironment::Wsl,
            })
        }
        CoreLaunchTarget::Ssh { destination } => {
            if !valid_ssh_destination(&destination) {
                return Err(invalid());
            }
            Ok(CoreCommand {
                program: PathBuf::from("ssh"),
                arguments: vec![
                    "-T".into(),
                    "-o".into(),
                    "BatchMode=yes".into(),
                    "-o".into(),
                    "ConnectTimeout=10".into(),
                    "--".into(),
                    destination.into(),
                    "gitnova-core".into(),
                ],
                environment: CoreEnvironment::Ssh,
            })
        }
        CoreLaunchTarget::DevContainer { workspace_folder } => {
            if !valid_workspace_folder(&workspace_folder) {
                return Err(invalid());
            }
            Ok(CoreCommand {
                program: PathBuf::from("devcontainer"),
                arguments: vec![
                    "exec".into(),
                    "--workspace-folder".into(),
                    workspace_folder.into(),
                    "gitnova-core".into(),
                ],
                environment: CoreEnvironment::DevContainer,
            })
        }
    }
}

fn valid_identifier(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && !value.starts_with('-')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn valid_ssh_destination(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 255
        && !value.starts_with('-')
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(byte, b'.' | b'_' | b'-' | b'@' | b':' | b'[' | b']')
        })
}

fn valid_workspace_folder(value: &str) -> bool {
    if value.is_empty() || value.len() > 4096 || value.chars().any(char::is_control) {
        return false;
    }
    let bytes = value.as_bytes();
    value.starts_with('/')
        || (bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && matches!(bytes[2], b'/' | b'\\'))
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

fn allowed_host_method(method: &str) -> bool {
    valid_method(method) && HOST_CORE_METHODS.contains(&method)
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
    use std::fs;
    use std::io::Cursor;
    use std::time::{SystemTime, UNIX_EPOCH};

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
        coreInfo:{name:'fake-core',version:'0.1.0'}, protocolVersion:'1.19', capabilities:{
          cancellation:true, repositoryDiscovery:true, workingTreeStatus:true,
          structuredFileDiff:true, paginatedCommitHistory:true, structuredCommitDiff:true,
          repositoryReferences:true, commitGraphProjection:true, githubRepository:true,
          githubPullRequest:true, githubPullRequestCommitDiff:true, githubSquashTrace:true,
          gitlabProject:true, gitlabMergeRequest:true, gitlabMergeRequestCommitDiff:true, gitlabSquashTrace:true, aiAssist:true,
          repositoryMutations:true, repositorySync:true, remoteBranchCheckout:true
        }
      }});
    } else if (request.method === 'gitnova/shutdown') {
      send({jsonrpc:'2.0', id:request.id, result:null});
    } else if (request.method === 'github/repository') {
      send({jsonrpc:'2.0', id:request.id, error:{code:-32021, message:'redacted by diagnostic boundary', data:{stableCode:'github.authentication_required', retryable:true}}});
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
        assert!(allowed_host_method("ai/inputPreview"));
        assert!(allowed_host_method("ai/generateCommitDraft"));
        assert!(allowed_host_method("repository/fetch"));
        assert!(allowed_host_method("repository/pull"));
        assert!(allowed_host_method("repository/push"));
        assert!(allowed_host_method("repository/checkoutRemoteBranch"));
        assert!(!allowed_host_method("test/echo"));
        assert_eq!(major_version("1.11"), Some("1"));
        assert_eq!(major_version("invalid"), None);
        assert_eq!(
            safe_diagnostic_error_code("github.authentication_required"),
            Some("github.authentication_required")
        );
        assert_eq!(safe_diagnostic_error_code("secret\nvalue"), None);
        assert_eq!(safe_diagnostic_version("1.19-beta+1"), Some("1.19-beta+1"));
        assert_eq!(safe_diagnostic_version("1.19\nsecret"), None);
    }

    #[test]
    fn supervises_initialize_request_and_graceful_shutdown() {
        let supervisor = CoreSupervisor::new(CoreCommand {
            program: PathBuf::from("node"),
            arguments: vec![OsString::from("-e"), OsString::from(FAKE_CORE)],
            environment: CoreEnvironment::Local,
        });
        let status = supervisor.start().unwrap();
        assert!(status.connected);
        assert_eq!(status.protocol_version.as_deref(), Some("1.19"));
        assert!(status.capabilities.unwrap().github_squash_trace);

        let response = supervisor
            .request("repository/open", serde_json::json!({"safe": true}))
            .unwrap();
        assert_eq!(response["result"]["method"], "repository/open");
        assert!(!supervisor.shutdown().unwrap().connected);
    }

    #[test]
    fn records_allowlisted_request_outcomes_without_payloads_or_messages() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock")
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "gitnova-transport-diagnostics-{}-{unique}",
            std::process::id()
        ));
        let diagnostics = Arc::new(DiagnosticLog::new(directory.clone()));
        let supervisor = CoreSupervisor::with_diagnostics(
            CoreCommand {
                program: PathBuf::from("node"),
                arguments: vec![OsString::from("-e"), OsString::from(FAKE_CORE)],
                environment: CoreEnvironment::Local,
            },
            Some(diagnostics.clone()),
        );

        supervisor.start().expect("start fake Core");
        let response = supervisor
            .request(
                "github/repository",
                serde_json::json!({"secret": "must-not-be-logged"}),
            )
            .expect("Core domain errors remain JSON-RPC responses");
        assert_eq!(
            response["error"]["data"]["stableCode"],
            "github.authentication_required"
        );
        supervisor.shutdown().expect("shutdown fake Core");

        let contents = fs::read_to_string(diagnostics.info().path).expect("diagnostic log");
        assert!(contents.contains("\"method\":\"github/repository\""));
        assert!(contents.contains("\"outcome\":\"core_error\""));
        assert!(contents.contains("\"errorCode\":\"github.authentication_required\""));
        assert!(!contents.contains("must-not-be-logged"));
        assert!(!contents.contains("redacted by diagnostic boundary"));
        fs::remove_dir_all(directory).expect("remove diagnostic test directory");
    }

    #[test]
    fn projects_structured_remote_launchers_without_a_shell() {
        let wsl = command_for_target(CoreLaunchTarget::Wsl {
            distribution: "Ubuntu-24.04".into(),
        })
        .unwrap();
        assert_eq!(wsl.program, PathBuf::from("wsl.exe"));
        assert_eq!(wsl.environment, CoreEnvironment::Wsl);
        assert_eq!(wsl.arguments[3], "gitnova-core");

        let ssh = command_for_target(CoreLaunchTarget::Ssh {
            destination: "git@example.com".into(),
        })
        .unwrap();
        assert_eq!(ssh.program, PathBuf::from("ssh"));
        assert!(ssh.arguments.contains(&OsString::from("BatchMode=yes")));
        assert_eq!(ssh.arguments.last(), Some(&OsString::from("gitnova-core")));

        let container = command_for_target(CoreLaunchTarget::DevContainer {
            workspace_folder: "/workspaces/gitnova".into(),
        })
        .unwrap();
        assert_eq!(container.program, PathBuf::from("devcontainer"));
        assert_eq!(container.environment, CoreEnvironment::DevContainer);

        assert!(
            command_for_target(CoreLaunchTarget::DevContainer {
                workspace_folder: r"D:\workspaces\gitnova".into(),
            })
            .is_ok()
        );
    }

    #[test]
    fn augments_only_the_macos_local_core_path_without_reordering_or_duplicates() {
        let inherited = env::join_paths([
            PathBuf::from("/custom/bin"),
            PathBuf::from("/usr/local/bin"),
        ])
        .unwrap();
        for environment in [
            CoreEnvironment::Wsl,
            CoreEnvironment::Ssh,
            CoreEnvironment::DevContainer,
        ] {
            assert_eq!(
                projected_local_core_path(environment, Some(&inherited)),
                None
            );
        }

        #[cfg(target_os = "macos")]
        {
            let projected = projected_local_core_path(CoreEnvironment::Local, Some(&inherited))
                .expect("macOS local Core must receive a projected PATH");
            let entries: Vec<_> = env::split_paths(&projected).collect();
            assert_eq!(entries[0], PathBuf::from("/custom/bin"));
            assert_eq!(entries[1], PathBuf::from("/usr/local/bin"));
            assert_eq!(
                entries
                    .iter()
                    .filter(|path| path.as_path() == std::path::Path::new("/usr/local/bin"))
                    .count(),
                1
            );
            for expected in [
                "/opt/homebrew/bin",
                "/opt/local/bin",
                "/usr/bin",
                "/bin",
                "/usr/sbin",
                "/sbin",
            ] {
                assert!(entries.contains(&PathBuf::from(expected)));
            }
        }
        #[cfg(not(target_os = "macos"))]
        assert_eq!(
            projected_local_core_path(CoreEnvironment::Local, Some(&inherited)),
            None
        );
    }

    #[test]
    fn rejects_launcher_argument_injection() {
        for target in [
            CoreLaunchTarget::Wsl {
                distribution: "--exec".into(),
            },
            CoreLaunchTarget::Ssh {
                destination: "host; touch /tmp/pwned".into(),
            },
            CoreLaunchTarget::DevContainer {
                workspace_folder: "relative/workspace".into(),
            },
            CoreLaunchTarget::DevContainer {
                workspace_folder: r"C:relative\workspace".into(),
            },
        ] {
            assert!(command_for_target(target).is_err());
        }
    }
}
