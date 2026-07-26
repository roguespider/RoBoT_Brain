// src/bridge/rmcp/helpers.rs
// Helper functions for RMCP

use rmcp::model::{ContentBlock, TextContent};

use crate::tools::ToolOutput;
use crate::workflows::enforcement::WorkflowEnforcementError;

/// Convert ToolOutput to MCP-compliant ContentBlock
pub fn tool_output_to_content(output: ToolOutput) -> ContentBlock {
    let text = if output.success {
        serde_json::to_string_pretty(&output.data)
            .unwrap_or_else(|_| r#"{"success": true}"#.to_string())
    } else {
        serde_json::to_string_pretty(&serde_json::json!({
            "success": false,
            "error": output.error.unwrap_or_else(|| "Unknown error".to_string())
        }))
        .unwrap_or_else(|_| r#"{"success": false, "error": "Failed to serialize error"}"#.to_string())
    };
    ContentBlock::Text(TextContent::new(text))
}

/// Helper function to convert enforcement error to ContentBlock
pub fn enforcement_error_to_content(error: WorkflowEnforcementError) -> ContentBlock {
    let text = serde_json::to_string_pretty(&serde_json::json!({
        "success": false,
        "error": {
            "code": error.error_code,
            "message": error.message,
            "required_action": error.required_action,
            "blocked_tools": error.tools_blocked
        },
        "hint": "Call get_workflow first, then search_memory before using other tools."
    }))
    .unwrap_or_else(|_| r#"{"success": false, "error": "Enforcement error"}"#.to_string());
    ContentBlock::Text(TextContent::new(text))
}
