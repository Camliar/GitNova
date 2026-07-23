//! Versioned JSON-RPC types shared by GitNova Core and contract tests.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use std::sync::{Arc, Mutex};

pub const JSON_RPC_VERSION: &str = "2.0";
pub const PROTOCOL_VERSION: &str = "1.3";

pub const ERROR_PARSE: i64 = -32700;
pub const ERROR_INVALID_REQUEST: i64 = -32600;
pub const ERROR_METHOD_NOT_FOUND: i64 = -32601;
pub const ERROR_INVALID_PARAMS: i64 = -32602;
pub const ERROR_INTERNAL: i64 = -32603;
pub const ERROR_INCOMPATIBLE_PROTOCOL: i64 = -32001;
pub const ERROR_NOT_INITIALIZED: i64 = -32002;
pub const ERROR_ALREADY_INITIALIZED: i64 = -32003;
pub const ERROR_INVALID_PATH: i64 = -32100;
pub const ERROR_REPOSITORY_NOT_FOUND: i64 = -32101;
pub const ERROR_GIT_UNAVAILABLE: i64 = -32102;
pub const ERROR_GIT_COMMAND_FAILED: i64 = -32103;
pub const ERROR_UNSAFE_REPOSITORY: i64 = -32104;
pub const ERROR_DIFFERENT_REPOSITORY_OPEN: i64 = -32105;
pub const ERROR_REPOSITORY_NOT_OPEN: i64 = -32106;
pub const ERROR_WORKTREE_REQUIRED: i64 = -32107;
pub const ERROR_STATUS_PARSE: i64 = -32108;
pub const ERROR_DIFF_PARSE: i64 = -32109;
pub const ERROR_INVALID_REPOSITORY_PATH: i64 = -32110;
pub const ERROR_REQUEST_CANCELLED: i64 = -32800;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(untagged)]
pub enum RequestId {
    Number(i64),
    String(String),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub jsonrpc: String,
    pub id: RequestId,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub jsonrpc: String,
    pub id: Option<RequestId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

impl Response {
    #[must_use]
    pub fn success(id: RequestId, result: Value) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.into(),
            id: Some(id),
            result: Some(result),
            error: None,
        }
    }

    #[must_use]
    pub fn error(id: Option<RequestId>, error: ResponseError) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.into(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseError {
    pub code: i64,
    pub message: String,
    pub data: ErrorData,
}

impl ResponseError {
    #[must_use]
    pub fn new(code: i64, stable_code: &str, message: &str, retryable: bool) -> Self {
        Self {
            code,
            message: message.into(),
            data: ErrorData {
                stable_code: stable_code.into(),
                retryable,
                details: BTreeMap::new(),
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorData {
    pub stable_code: String,
    pub retryable: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub details: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImplementationInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    #[serde(default)]
    pub cancellation: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub client_info: ImplementationInfo,
    pub protocol_version: String,
    #[serde(default)]
    pub capabilities: ClientCapabilities,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    pub cancellation: bool,
    pub repository_discovery: bool,
    pub working_tree_status: bool,
    pub structured_file_diff: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub core_info: ImplementationInfo,
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CancelParams {
    pub id: RequestId,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RepositoryPathParams {
    pub path: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RepositoryKind {
    Worktree,
    LinkedWorktree,
    Bare,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryDescriptor {
    pub worktree_root: Option<String>,
    pub git_directory: String,
    pub common_git_directory: String,
    pub kind: RepositoryKind,
    pub git_version: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StatusEntryKind {
    Ordinary,
    RenameOrCopy,
    Unmerged,
    Untracked,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FileStatus {
    Unmodified,
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Unmerged,
    Untracked,
    TypeChanged,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusEntry {
    pub path: String,
    pub original_path: Option<String>,
    pub kind: StatusEntryKind,
    pub index_status: FileStatus,
    pub worktree_status: FileStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchStatus {
    pub head: Option<String>,
    pub oid: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u64,
    pub behind: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkingTreeStatus {
    pub branch: BranchStatus,
    pub entries: Vec<StatusEntry>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DiffScope {
    WorkingTree,
    Staged,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffParams {
    pub path: String,
    pub scope: DiffScope,
    #[serde(default)]
    pub context_lines: Option<u8>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DiffLineKind {
    Context,
    Addition,
    Deletion,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
    pub old_line: Option<u64>,
    pub new_line: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffHunk {
    pub old_start: u64,
    pub old_lines: u64,
    pub new_start: u64,
    pub new_lines: u64,
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDiff {
    pub old_path: String,
    pub new_path: String,
    pub is_binary: bool,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Clone, Default)]
pub struct CancellationRegistry {
    cancelled: Arc<Mutex<HashSet<RequestId>>>,
}

impl CancellationRegistry {
    pub fn cancel(&self, id: RequestId) {
        self.cancelled
            .lock()
            .expect("cancellation registry mutex poisoned")
            .insert(id);
    }

    #[must_use]
    pub fn take_cancelled(&self, id: &RequestId) -> bool {
        self.cancelled
            .lock()
            .expect("cancellation registry mutex poisoned")
            .remove(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_id_preserves_string_and_number_types() {
        let number: RequestId = serde_json::from_str("7").unwrap();
        let string: RequestId = serde_json::from_str("\"7\"").unwrap();
        assert_eq!(number, RequestId::Number(7));
        assert_eq!(string, RequestId::String("7".into()));
    }

    #[test]
    fn cancellation_is_consumed_once() {
        let registry = CancellationRegistry::default();
        let id = RequestId::String("request-1".into());
        registry.cancel(id.clone());
        assert!(registry.take_cancelled(&id));
        assert!(!registry.take_cancelled(&id));
    }

    #[test]
    fn rust_contract_matches_schema_version_and_field_names() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../sdk/protocol/gitnova-protocol.schema.json"
        ))
        .unwrap();
        assert_eq!(
            schema["properties"]["protocolVersion"]["const"],
            PROTOCOL_VERSION
        );
        for definition in [
            "RequestId",
            "ImplementationInfo",
            "ClientCapabilities",
            "ServerCapabilities",
            "InitializeParams",
            "InitializeResult",
            "CancelParams",
            "ErrorData",
            "RepositoryPathParams",
            "RepositoryKind",
            "RepositoryDescriptor",
            "StatusEntryKind",
            "FileStatus",
            "StatusEntry",
            "BranchStatus",
            "WorkingTreeStatus",
            "DiffScope",
            "DiffParams",
            "DiffLineKind",
            "DiffLine",
            "DiffHunk",
            "FileDiff",
        ] {
            assert!(schema["$defs"].get(definition).is_some());
        }

        let params = InitializeParams {
            client_info: ImplementationInfo {
                name: "test-host".into(),
                version: "1.0.0".into(),
            },
            protocol_version: PROTOCOL_VERSION.into(),
            capabilities: ClientCapabilities::default(),
        };
        let serialized = serde_json::to_value(params).unwrap();
        assert!(serialized.get("clientInfo").is_some());
        assert!(serialized.get("protocolVersion").is_some());
        assert!(serialized.get("client_info").is_none());
    }
}
