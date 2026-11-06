//! What the real binary puts on its two output streams.
#![allow(clippy::expect_used)]

use std::io::Write as _;
use std::process::{Command, Stdio};

use serde_json::Value;

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"0"}}}"#;
const LIST_TOOLS: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;

fn session_file(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "bloppen-mcp-transport-{}-{name}.json",
        std::process::id()
    ))
}

#[test]
fn the_log_never_lands_on_the_protocol_stream() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_bloppen-mcp"))
        .arg("--session-file")
        .arg(session_file("log"))
        .env("BORNELOPPEN_MCP_LOG", "trace")
        .env_remove("BORNELOPPEN_EMAIL")
        .env_remove("BORNELOPPEN_PASSWORD")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary starts");

    let mut stdin = child.stdin.take().expect("stdin is piped");
    writeln!(stdin, "{INITIALIZE}").expect("the handshake is accepted");
    writeln!(
        stdin,
        r#"{{"jsonrpc":"2.0","method":"notifications/initialized"}}"#
    )
    .expect("the notification is accepted");
    writeln!(stdin, "{LIST_TOOLS}").expect("the listing is accepted");
    drop(stdin);

    let output = child.wait_with_output().expect("the server exits");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert_eq!(lines.len(), 2, "expected two responses, got:\n{stdout}");
    for line in lines {
        serde_json::from_str::<Value>(line)
            .unwrap_or_else(|error| panic!("stdout carried a non-frame: {error}\n{line}"));
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("TRACE") || stderr.contains("DEBUG"),
        "{stderr}"
    );
}

#[test]
fn tool_schemas_print_without_credentials() {
    let output = Command::new(env!("CARGO_BIN_EXE_bloppen-mcp"))
        .arg("--list-tools")
        .env_remove("BORNELOPPEN_EMAIL")
        .env_remove("BORNELOPPEN_PASSWORD")
        .output()
        .expect("the binary runs");

    assert!(output.status.success(), "{:?}", output.stderr);
    let tools: Vec<Value> =
        serde_json::from_slice(&output.stdout).expect("--list-tools prints JSON");
    assert_eq!(tools.len(), 3);
}

#[test]
fn a_refused_check_explains_the_missing_credentials_on_stderr() {
    let output = Command::new(env!("CARGO_BIN_EXE_bloppen-mcp"))
        .arg("--check")
        .arg("--session-file")
        .arg(session_file("check"))
        .env_remove("BORNELOPPEN_EMAIL")
        .env_remove("BORNELOPPEN_PASSWORD")
        .output()
        .expect("the binary runs");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty(), "{:?}", output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("BORNELOPPEN_EMAIL"), "{stderr}");
}
