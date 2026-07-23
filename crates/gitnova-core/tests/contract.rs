use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Output, Stdio};

fn frame(value: &Value) -> Vec<u8> {
    let body = serde_json::to_vec(value).unwrap();
    let mut framed = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    framed.extend(body);
    framed
}

fn run(messages: &[Value]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_gitnova-core"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let stdin = child.stdin.as_mut().unwrap();
        for message in messages {
            stdin.write_all(&frame(message)).unwrap();
        }
    }
    drop(child.stdin.take());
    child.wait_with_output().unwrap()
}

fn responses(bytes: &[u8]) -> Vec<Value> {
    let mut remaining = bytes;
    let mut values = Vec::new();
    while !remaining.is_empty() {
        let header_end = remaining
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .expect("stdout contains only framed messages");
        let header = std::str::from_utf8(&remaining[..header_end]).unwrap();
        let length: usize = header
            .strip_prefix("Content-Length: ")
            .expect("canonical Content-Length header")
            .parse()
            .unwrap();
        let body_start = header_end + 4;
        let body_end = body_start + length;
        values.push(serde_json::from_slice(&remaining[body_start..body_end]).unwrap());
        remaining = &remaining[body_end..];
    }
    values
}

fn initialize(id: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "gitnova/initialize",
        "params": {
            "clientInfo": {"name": "contract-test", "version": "1.0.0"},
            "protocolVersion": "1.0",
            "capabilities": {"cancellation": true}
        }
    })
}

#[test]
fn completes_lifecycle_and_keeps_stdout_protocol_clean() {
    let output = run(&[
        initialize(json!("init-1")),
        json!({"jsonrpc":"2.0","id":2,"method":"gitnova/shutdown"}),
        json!({"jsonrpc":"2.0","method":"exit"}),
    ]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let responses = responses(&output.stdout);
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0]["id"], "init-1");
    assert_eq!(responses[0]["result"]["protocolVersion"], "1.0");
    assert_eq!(responses[0]["result"]["capabilities"]["cancellation"], true);
    assert_eq!(responses[1]["result"], Value::Null);
}

#[test]
fn rejects_requests_before_initialization() {
    let output = run(&[json!({
        "jsonrpc":"2.0","id":9,"method":"gitnova/unknown","params":{}
    })]);
    assert!(output.status.success());
    let responses = responses(&output.stdout);
    assert_eq!(responses[0]["error"]["code"], -32002);
    assert_eq!(
        responses[0]["error"]["data"]["stableCode"],
        "core.not_initialized"
    );
}

#[test]
fn cancellation_notification_is_applied_to_matching_request_id() {
    let output = run(&[
        initialize(json!(1)),
        json!({"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":"work-1"}}),
        json!({"jsonrpc":"2.0","id":"work-1","method":"gitnova/unknown","params":{}}),
    ]);
    let responses = responses(&output.stdout);
    assert_eq!(responses[1]["error"]["code"], -32800);
    assert_eq!(
        responses[1]["error"]["data"]["stableCode"],
        "request.cancelled"
    );
}

#[test]
fn exit_without_shutdown_is_unsuccessful() {
    let output = run(&[json!({"jsonrpc":"2.0","method":"exit"})]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}
