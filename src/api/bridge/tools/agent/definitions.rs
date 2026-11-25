// src/tools/agent/definitions.rs
// Agent tool definitions

use crate::api::mcp::McpTool;

pub const GET_WORKFLOW: &str = "get_workflow";
pub const LIST_TOOLS: &str = "list_tools";
pub const GET_TOOL: &str = "get_tool";
pub const CONNECT_MCP_SERVER: &str = "connect_mcp_server";
pub const CALL_MCP_TOOL: &str = "call_tool";
pub const RUN_AGENT_GOAL: &str = "run_agent_goal";

/// Get all agent tool definitions
pub fn all() -> Vec<McpTool> {
    macro_rules! desc {
        ($s:expr) => {
            format!("[WORKFLOW: get_workflow + search_memory first] {}", $s)
        };
    }
    vec![
        McpTool {
            name: GET_WORKFLOW.to_string(),
            description: "MANDATORY: Get workflow rules. MUST be called before any other tool. Returns the required workflow for this MCP server.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "purpose": {
                        "type": "string",
                        "description": "Context for why you need the workflow (e.g., 'file_ingestion', 'memory_search', 'general')"
                    }
                }
            }),
        },
        McpTool {
            name: LIST_TOOLS.to_string(),
            description: desc!("List all available MCP tools with optional filter"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "filter": {
                        "type": "string",
                        "description": "Optional filter to match tool names or descriptions"
                    }
                }
            }),
        },
        McpTool {
            name: GET_TOOL.to_string(),
            description: desc!("Get detailed information about a specific tool"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "The name of the tool to get details for"
                    }
                },
                "required": ["name"]
            }),
        },
        McpTool {
            name: CONNECT_MCP_SERVER.to_string(),
            description: desc!("Connect to an external MCP server via child process. For workers module: {\"name\": \"workers\", \"command\": \"python\", \"args\": [\"-u\", \".agents/.workers/workers.py\"]}. After connecting, call list_tools to discover external tools (delegate_to_workers, evaluate_worker_output, list_worker_models, get_usage_guide), then use call_tool to invoke them."),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name to identify this server (e.g., 'workers')"
                    },
                    "command": {
                        "type": "string",
                        "description": "Path to the MCP server executable (e.g., 'python' for Python servers, NOT 'python workers.py')"
                    },
                    "args": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Command line arguments for the server (e.g., [\"-u\", \".agents/.workers/workers.py\"] for workers)"
                    }
                },
                "required": ["name", "command"]
            }),
        },
        McpTool {
            name: CALL_MCP_TOOL.to_string(),
            description: desc!("Call a tool on a connected MCP server"),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "tool_name": {
                        "type": "string",
                        "description": "Name of the tool to call"
                    },
                    "arguments": {
                        "type": "string",
                        "description": "JSON-encoded arguments to pass to the tool (e.g., '{\"key\": \"value\"}')"
                    }
                },
                "required": ["tool_name"]
            }),
        },
        McpTool {
            name: RUN_AGENT_GOAL.to_string(),
            description: desc!("Run the goal-driven agent loop (Architecture §5.7). Given a goal, the agent decomposes it into a plan, retrieves supporting memory/knowledge/experiences, evaluates action confidence, checks the safety gate, and records the outcome as a new experience. This closes the cognitive loop: Goal → Plan → Retrieve → Decide → Act → Record → Learn."),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "goal": {
                        "type": "string",
                        "description": "The goal for the agent to pursue"
                    },
                    "confidence_threshold": {
                        "type": "number",
                        "description": "Minimum confidence (0.0–1.0) required to act. Default 0.5."
                    }
                },
                "required": ["goal"]
            }),
        },
    ]
}
