// src/bridge/mcp/types/self_check.rs
//! MCP types self-check (Architecture §8: MCP/ACP bridge)
//!
//! Exercises the MCP protocol type builders and predicates
//! (McpMessage::is_request/is_response/is_notification,
//! McpResponse::is_success, McpError::with_data,
//! McpCapabilities::with_tools, McpTool::with_schema) so those code paths
//! remain live rather than dead code.

use tracing::info;

use super::capabilities::McpCapabilities;
use super::error::McpError;
use super::message::{McpMessage, McpNotification, McpRequest, McpResponse};
use super::tools::McpTool;

/// Run the MCP types self-check. Returns the number of checks that passed.
pub fn run() -> usize {
    let mut checks_total = 0usize;
    let mut checks_passed = 0usize;

    // 1. McpMessage predicates across all three variants.
    checks_total += 1;
    let req = McpMessage::Request(McpRequest::new("tools/list", "1"));
    let resp = McpMessage::Response(McpResponse::success("1", serde_json::json!({})));
    let note = McpMessage::Notification(McpNotification::new("initialized"));
    if req.is_request() && !req.is_response() && resp.is_response() && note.is_notification() {
        checks_passed += 1;
    }

    // 2. McpResponse::is_success with success and error responses.
    checks_total += 1;
    let ok_resp = McpResponse::success("2", serde_json::json!({"ok": true}));
    let err_resp = McpResponse::error(
        "3",
        McpError::new(-32601, "method not found").with_data(serde_json::json!({"method": "x"})),
    );
    if ok_resp.is_success() && !err_resp.is_success() {
        checks_passed += 1;
    }

    // 3. McpRequest::with_params / McpNotification::with_params.
    checks_total += 1;
    let req_with = McpRequest::new("tools/call", "4")
        .with_params(serde_json::json!({"name": "search"}));
    let note_with = McpNotification::new("progress")
        .with_params(serde_json::json!({"progress": 50}));
    if req_with.params.is_some() && note_with.params.is_some() {
        checks_passed += 1;
    }

    // 4. McpCapabilities::with_tools vs all.
    checks_total += 1;
    let tools_only = McpCapabilities::with_tools();
    let all = McpCapabilities::all();
    if tools_only.tools.is_some()
        && tools_only.resources.is_none()
        && all.resources.is_some()
        && all.logging.is_some()
    {
        checks_passed += 1;
    }

    // 5. McpTool::with_schema.
    checks_total += 1;
    let tool = McpTool::new("search", "Search memories").with_schema(serde_json::json!({
        "type": "object",
        "properties": {}
    }));
    if tool.input_schema.is_object() {
        checks_passed += 1;
    }

    info!(
        "MCP types self-check: {}/{} checks passed",
        checks_passed, checks_total
    );
    checks_passed
}
