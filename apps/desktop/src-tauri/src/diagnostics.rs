use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_LOG_BYTES: u64 = 1024 * 1024;
const RETAINED_FILES: u8 = 2;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticInfo {
    pub path: String,
    pub max_bytes: u64,
    pub retained_files: u8,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticEvent<'a> {
    timestamp_ms: u64,
    level: &'static str,
    event: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    environment: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    outcome: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_code: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    protocol_version: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    app_version: Option<&'a str>,
}

/// Best-effort local diagnostics. The typed methods are the privacy boundary:
/// callers cannot pass RPC payloads, repository paths, stderr, or error messages.
pub struct DiagnosticLog {
    active_path: PathBuf,
    previous_path: PathBuf,
    writer: Mutex<()>,
}

impl DiagnosticLog {
    #[must_use]
    pub fn new(directory: PathBuf) -> Self {
        let active_path = directory.join("diagnostics.jsonl");
        let previous_path = directory.join("diagnostics.previous.jsonl");
        let log = Self {
            active_path,
            previous_path,
            writer: Mutex::new(()),
        };
        let _ = fs::create_dir_all(directory);
        log
    }

    #[must_use]
    pub fn info(&self) -> DiagnosticInfo {
        DiagnosticInfo {
            path: self.active_path.to_string_lossy().into_owned(),
            max_bytes: MAX_LOG_BYTES,
            retained_files: RETAINED_FILES,
        }
    }

    pub fn app_started(&self, version: &str) {
        self.record(DiagnosticEvent {
            timestamp_ms: timestamp_ms(),
            level: "info",
            event: "app.started",
            environment: None,
            method: None,
            duration_ms: None,
            outcome: Some("success"),
            error_code: None,
            protocol_version: None,
            app_version: Some(version),
        });
    }

    pub fn core_configured(
        &self,
        environment: &str,
        outcome: &'static str,
        error_code: Option<&str>,
    ) {
        self.lifecycle(
            "core.configured",
            environment,
            None,
            outcome,
            error_code,
            None,
        );
    }

    pub fn core_started(
        &self,
        environment: &str,
        duration_ms: u64,
        outcome: &'static str,
        error_code: Option<&str>,
        protocol_version: Option<&str>,
    ) {
        self.lifecycle(
            "core.started",
            environment,
            Some(duration_ms),
            outcome,
            error_code,
            protocol_version,
        );
    }

    pub fn core_shutdown(
        &self,
        environment: &str,
        duration_ms: u64,
        outcome: &'static str,
        error_code: Option<&str>,
    ) {
        self.lifecycle(
            "core.shutdown",
            environment,
            Some(duration_ms),
            outcome,
            error_code,
            None,
        );
    }

    pub fn core_request(
        &self,
        environment: &str,
        method: Option<&str>,
        duration_ms: u64,
        outcome: &'static str,
        error_code: Option<&str>,
    ) {
        self.record(DiagnosticEvent {
            timestamp_ms: timestamp_ms(),
            level: if outcome == "success" { "info" } else { "warn" },
            event: "core.request",
            environment: Some(environment),
            method,
            duration_ms: Some(duration_ms),
            outcome: Some(outcome),
            error_code,
            protocol_version: None,
            app_version: None,
        });
    }

    fn lifecycle(
        &self,
        event: &'static str,
        environment: &str,
        duration_ms: Option<u64>,
        outcome: &'static str,
        error_code: Option<&str>,
        protocol_version: Option<&str>,
    ) {
        self.record(DiagnosticEvent {
            timestamp_ms: timestamp_ms(),
            level: if outcome == "success" { "info" } else { "warn" },
            event,
            environment: Some(environment),
            method: None,
            duration_ms,
            outcome: Some(outcome),
            error_code,
            protocol_version,
            app_version: None,
        });
    }

    fn record(&self, event: DiagnosticEvent<'_>) {
        let Ok(mut line) = serde_json::to_vec(&event) else {
            return;
        };
        line.push(b'\n');
        self.append(&line);
    }

    fn append(&self, line: &[u8]) {
        let Ok(_guard) = self.writer.lock() else {
            return;
        };
        let current_size = fs::metadata(&self.active_path).map_or(0, |metadata| metadata.len());
        if current_size.saturating_add(line.len() as u64) > MAX_LOG_BYTES {
            self.rotate();
        }
        let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.active_path)
        else {
            return;
        };
        let _ = file.write_all(line);
    }

    fn rotate(&self) {
        if self.previous_path.exists() && fs::remove_file(&self.previous_path).is_err() {
            return;
        }
        if self.active_path.exists() {
            let _ = fs::rename(&self.active_path, &self.previous_path);
        }
    }
}

fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::path::Path;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(1);

    fn test_directory(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "gitnova-diagnostics-{name}-{}-{}",
            std::process::id(),
            NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn writes_only_structured_diagnostic_fields() {
        let directory = test_directory("fields");
        let log = DiagnosticLog::new(directory.clone());
        log.core_request(
            "local",
            Some("github/commitSquashTrace"),
            42,
            "core_error",
            Some("github.authentication_required"),
        );

        let contents = fs::read_to_string(log.info().path).expect("diagnostic log");
        let event: Value = serde_json::from_str(contents.trim()).expect("valid JSONL event");
        assert_eq!(event["event"], "core.request");
        assert_eq!(event["method"], "github/commitSquashTrace");
        assert_eq!(event["durationMs"], 42);
        assert_eq!(event["errorCode"], "github.authentication_required");
        assert!(event.get("params").is_none());
        assert!(event.get("result").is_none());
        assert!(event.get("message").is_none());
        assert!(event.get("path").is_none());
        fs::remove_dir_all(directory).expect("remove test directory");
    }

    #[test]
    fn rotates_active_log_and_keeps_one_previous_file() {
        let directory = test_directory("rotation");
        let log = DiagnosticLog::new(directory.clone());
        log.append(&vec![b'x'; MAX_LOG_BYTES as usize]);
        log.app_started("0.1.0");
        assert_eq!(
            fs::metadata(&log.previous_path)
                .expect("previous log")
                .len(),
            MAX_LOG_BYTES
        );
        let active = fs::read_to_string(&log.active_path).expect("active log");
        assert!(active.contains("\"event\":\"app.started\""));

        log.append(&vec![b'y'; MAX_LOG_BYTES as usize]);
        log.app_started("0.1.0");
        assert_eq!(
            fs::metadata(&log.previous_path)
                .expect("replaced previous log")
                .len(),
            MAX_LOG_BYTES
        );
        assert_eq!(fs::read_dir(&directory).expect("log directory").count(), 2);
        fs::remove_dir_all(directory).expect("remove test directory");
    }

    #[test]
    fn logging_io_failures_are_non_blocking() {
        let root = test_directory("failure");
        fs::write(&root, b"not a directory").expect("blocking file");
        let log = DiagnosticLog::new(root.join("logs"));
        log.app_started("0.1.0");
        assert!(!Path::new(&log.info().path).exists());
        fs::remove_file(root).expect("remove blocking file");
    }
}
