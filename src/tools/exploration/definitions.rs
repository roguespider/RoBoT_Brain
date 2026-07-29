
// src/tools/exploration_tools/definitions.rs
//! Tool definitions for exploration tools

pub const START_EXPLORATION: &str = "start_exploration";
pub const PAUSE_EXPLORATION: &str = "pause_exploration";
pub const RESUME_EXPLORATION: &str = "resume_exploration";
pub const GET_EXPLORATION_STATUS: &str = "get_exploration_status";
pub const COMPLETE_EXPLORATION: &str = "complete_exploration";
pub const RECORD_ATTEMPT: &str = "record_attempt";
pub const ADD_HYPOTHESIS: &str = "add_hypothesis";
pub const EVALUATE_HYPOTHESIS: &str = "evaluate_exploration_hypothesis";
pub const PROMOTE_FINDING: &str = "promote_finding";
pub const ABANDON_EXPLORATION: &str = "abandon_exploration";

pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
    vec![
        crate::bridge::mcp::McpTool {
            name: START_EXPLORATION.to_string(),
            description: "Start a new exploration session. Explorations allow RoBoT to actively investigate topics and test hypotheses.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Human-readable title for this exploration" },
                    "purpose": { "type": "string", "description": "Why this exploration is being conducted" }
                },
                "required": ["title", "purpose"]
            }),
        },
        crate::bridge::mcp::McpTool {
            name: PAUSE_EXPLORATION.to_string(),
            description: "Pause an active exploration. The exploration can be resumed later.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "exploration_id": { "type": "string", "description": "The exploration ID to pause" }
                },
                "required": ["exploration_id"]
            }),
        },
        crate::bridge::mcp::McpTool {
            name: RESUME_EXPLORATION.to_string(),
            description: "Resume a paused exploration.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "exploration_id": { "type": "string", "description": "The exploration ID to resume" }
                },
                "required": ["exploration_id"]
            }),
        },
        crate::bridge::mcp::McpTool {
            name: GET_EXPLORATION_STATUS.to_string(),
            description: "Get the current status of an exploration including hypotheses, attempts, and findings.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "exploration_id": { "type": "string", "description": "The exploration ID to check" }
                },
                "required": ["exploration_id"]
            }),
        },
        crate::bridge::mcp::McpTool {
            name: COMPLETE_EXPLORATION.to_string(),
            description: "Mark an exploration as completed with findings.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "exploration_id": { "type": "string", "description": "The exploration ID to complete" },
                    "findings": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "description": { "type": "string", "description": "Description of what was discovered" },
                                "confidence": { "type": "number", "description": "Confidence in this finding (0.0-1.0)" }
                            },
                            "required": ["description"]
                        },
                        "description": "Findings from this exploration"
                    }
                },
                "required": ["exploration_id", "findings"]
            }),
        },
        crate::bridge::mcp::McpTool {
            name: RECORD_ATTEMPT.to_string(),
            description: "Record an attempt made during exploration.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "exploration_id": { "type": "string", "description": "The exploration ID" },
                    "action": { "type": "string", "description": "What action was taken" },
                    "expected_result": { "type": "string", "description": "What result was expected (optional)" },
                    "actual_result": { "type": "string", "description": "What actually happened (optional)" }
                },
                "required": ["exploration_id", "action"]
            }),
        },
        crate::bridge::mcp::McpTool {
            name: ADD_HYPOTHESIS.to_string(),
            description: "Add a hypothesis to an exploration for testing.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "exploration_id": { "type": "string", "description": "The exploration ID" },
                    "statement": { "type": "string", "description": "The hypothesis statement" },
                    "initial_confidence": { "type": "number", "description": "Initial confidence (0.0-1.0, default 0.5)" }
                },
                "required": ["exploration_id", "statement"]
            }),
        },
        crate::bridge::mcp::McpTool {
            name: EVALUATE_HYPOTHESIS.to_string(),
            description: "Evaluate a hypothesis with a result.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "exploration_id": { "type": "string", "description": "The exploration ID" },
                    "hypothesis_id": { "type": "string", "description": "The hypothesis ID to evaluate" },
                    "result": { "type": "string", "description": "Result: supported, partially_supported, rejected, unknown" }
                },
                "required": ["exploration_id", "hypothesis_id", "result"]
            }),
        },
        crate::bridge::mcp::McpTool {
            name: PROMOTE_FINDING.to_string(),
            description: "Promote a finding to reusable knowledge.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "exploration_id": { "type": "string", "description": "The exploration ID" },
                    "finding_id": { "type": "string", "description": "The finding ID to promote" }
                },
                "required": ["exploration_id", "finding_id"]
            }),
        },
        crate::bridge::mcp::McpTool {
            name: ABANDON_EXPLORATION.to_string(),
            description: "Abandon an exploration without findings.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "exploration_id": { "type": "string", "description": "The exploration ID to abandon" }
                },
                "required": ["exploration_id"]
            }),
        },
    ]
}
