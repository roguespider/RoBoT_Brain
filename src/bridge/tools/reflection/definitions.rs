// src/tools/reflection/definitions.rs
//! Reflection tool definitions

/// Tool name definitions
pub const GET_INSIGHTS: &str = "get_insights";
pub const CREATE_REFLECTION: &str = "create_reflection";
pub const ANALYZE_PATTERNS: &str = "analyze_patterns";
pub const GET_PATTERNS: &str = "get_patterns";

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
    ]
}
