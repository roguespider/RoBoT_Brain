// src/tools/reflection/definitions.rs
//! Reflection tool definitions

/// Tool name definitions
pub const GET_INSIGHTS: &str = "get_insights";
pub const CREATE_REFLECTION: &str = "create_reflection";
pub const ANALYZE_PATTERNS: &str = "analyze_patterns";
pub const GET_PATTERNS: &str = "get_patterns";
pub const VALIDATE_REFLECTION: &str = "validate_reflection";
pub const LIST_REFLECTIONS_BY_STATUS: &str = "list_reflections_by_status";
pub const UPDATE_REFLECTION: &str = "update_reflection";

/// Get all reflection tools
pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
    vec![
        crate::bridge::mcp::McpTool {
            name: GET_INSIGHTS.to_string(),
            description: "Get actionable insights from reflections".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "min_confidence": {
                        "type": "number",
                        "description": "Minimum confidence threshold (0.0 - 1.0)",
                        "minimum": 0.0,
                        "maximum": 1.0
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of insights to return",
                        "default": 10
                    }
                }
            }),
        },
        crate::bridge::mcp::McpTool {
            name: CREATE_REFLECTION.to_string(),
            description: "Create a new reflection from experiences".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "Title for the reflection"
                    },
                    "description": {
                        "type": "string",
                        "description": "Detailed description and reasoning"
                    },
                    "reflection_type": {
                        "type": "string",
                        "description": "Type of reflection",
                        "enum": ["success", "failure", "improvement", "pattern", "anomaly", "strategy", "general", "analysis"]
                    },
                    "experience_ids": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "IDs of experiences to reflect on"
                    }
                }
            }),
        },
        crate::bridge::mcp::McpTool {
            name: ANALYZE_PATTERNS.to_string(),
            description: "Analyze experiences to detect patterns".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "experience_ids": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Experience IDs to analyze"
                    }
                }
            }),
        },
        crate::bridge::mcp::McpTool {
            name: GET_PATTERNS.to_string(),
            description: "Get detected patterns".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "min_confidence": {
                        "type": "number",
                        "description": "Minimum confidence threshold",
                        "minimum": 0.0,
                        "maximum": 1.0
                    },
                    "pattern_type": {
                        "type": "string",
                        "description": "Filter by pattern type"
                    }
                }
            }),
        },
        crate::bridge::mcp::McpTool {
            name: VALIDATE_REFLECTION.to_string(),
            description: "Validate a reflection for quality and consistency".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "reflection_id": {
                        "type": "string",
                        "description": "ID of the reflection to validate"
                    }
                },
                "required": ["reflection_id"]
            }),
        },
        crate::bridge::mcp::McpTool {
            name: LIST_REFLECTIONS_BY_STATUS.to_string(),
            description: "List reflections filtered by status".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "status": {
                        "type": "string",
                        "description": "Reflection status to filter by",
                        "enum": ["draft", "active", "validated", "archived"]
                    }
                },
                "required": ["status"]
            }),
        },
        crate::bridge::mcp::McpTool {
            name: UPDATE_REFLECTION.to_string(),
            description: "Update an existing reflection".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "reflection_id": {
                        "type": "string",
                        "description": "ID of the reflection to update"
                    },
                    "title": {
                        "type": "string",
                        "description": "New title"
                    },
                    "description": {
                        "type": "string",
                        "description": "New description"
                    },
                    "summary": {
                        "type": "string",
                        "description": "New summary"
                    }
                },
                "required": ["reflection_id"]
            }),
        },
    ]
}
