//! Developer Interface tool definitions (constants and schemas)

/// Tool name constants
pub const DEVELOPER_EXECUTE_COMMAND: &str = "developer_execute_command";
pub const DEVELOPER_EXPLORE_COGNITIVE_SYSTEM: &str = "developer_explore_cognitive_system";
pub const DEVELOPER_INSPECT_MEMORY_BY_LAYER: &str = "developer_inspect_memory_by_layer";
pub const DEVELOPER_INSPECT_STRATEGIC_LEARNING: &str = "developer_inspect_strategic_learning";
pub const DEVELOPER_INSPECT_BACKGROUND_WORKERS: &str = "developer_inspect_background_workers";
pub const DEVELOPER_MANAGE_MEMORY_LAYER: &str = "developer_manage_memory_layer";
pub const DEVELOPER_INSPECT_LEARNING_EVOLUTION: &str = "developer_inspect_learning_evolution";

pub fn all() -> Vec<crate::api::mcp::McpTool> {
    vec![
        crate::api::mcp::McpTool {
            name: DEVELOPER_EXECUTE_COMMAND.to_string(),
            description: "[WORKFLOW: get_workflow + search_memory first] Execute a control plane command on the developer interface."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Command name to execute"
                    }
                },
                "required": ["name"]
            }),
        },
        crate::api::mcp::McpTool {
            name: DEVELOPER_EXPLORE_COGNITIVE_SYSTEM.to_string(),
            description: "[WORKFLOW: get_workflow + search_memory first] Explore a cognitive subsystem (Memory, KnowledgeGraph, Experience, etc.)."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "target": {
                        "type": "string",
                        "description": "Subsystem target to explore"
                    }
                },
                "required": ["target"]
            }),
        },
        crate::api::mcp::McpTool {
            name: DEVELOPER_INSPECT_MEMORY_BY_LAYER.to_string(),
            description: "[WORKFLOW: get_workflow + search_memory first] Inspect memory records in a specific layer."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "layer": {
                        "type": "string",
                        "description": "Memory layer to inspect"
                    }
                },
                "required": ["layer"]
            }),
        },
        crate::api::mcp::McpTool {
            name: DEVELOPER_INSPECT_STRATEGIC_LEARNING.to_string(),
            description: "[WORKFLOW: get_workflow + search_memory first] Inspect strategic learning state from knowledge and experience tables."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        crate::api::mcp::McpTool {
            name: DEVELOPER_INSPECT_BACKGROUND_WORKERS.to_string(),
            description: "[WORKFLOW: get_workflow + search_memory first] Inspect background worker status from scheduled tasks."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        crate::api::mcp::McpTool {
            name: DEVELOPER_MANAGE_MEMORY_LAYER.to_string(),
            description: "[WORKFLOW: get_workflow + search_memory first] Manage a memory layer: inspect, promote, or archive records."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "layer": {
                        "type": "string",
                        "description": "Memory layer to manage"
                    },
                    "action": {
                        "type": "string",
                        "description": "Action: inspect, promote, or archive"
                    }
                },
                "required": ["layer", "action"]
            }),
        },
        crate::api::mcp::McpTool {
            name: DEVELOPER_INSPECT_LEARNING_EVOLUTION.to_string(),
            description: "[WORKFLOW: get_workflow + search_memory first] Inspect learning evolution state (delegates to strategic learning inspection)."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
    ]
}
