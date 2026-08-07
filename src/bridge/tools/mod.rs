


// src/tools/mod.rs
// MCP tools for Zed Editor integration


use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Standard output type for MCP tool executions
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ToolOutput {
    /// Whether the tool execution was successful
    pub success: bool,
    /// The result data from the tool
    pub data: serde_json::Value,
    /// Optional error message if execution failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ToolOutput {
    /// Create a successful tool output
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data,
            error: None,
        }
    }

    /// Create a failed tool output
    pub fn error<E: std::fmt::Display>(msg: E) -> Self {
        Self {
            success: false,
            data: serde_json::Value::Null,
            error: Some(msg.to_string()),
        }
    }
}

pub mod agent;
pub mod experience;
pub mod exploration;
pub mod hypothesis;
pub mod ingestor;
pub mod knowledge;
pub mod memory;
pub mod planner;
pub mod reflection;
pub mod search;
pub mod skills;
pub mod workflow;

/// Global tool registry (lazily initialized, using Mutex since only written once at startup)
static TOOL_REGISTRY: std::sync::OnceLock<Arc<Mutex<ToolRegistry>>> = std::sync::OnceLock::new();

/// Tool registry for MCP tools
pub struct ToolRegistry {
    pub tools: Vec<crate::bridge::mcp::McpTool>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }
}

/// Register all MCP tools
pub fn register_tools() {
    // Get or create the registry
    let registry = TOOL_REGISTRY.get_or_init(|| Arc::new(Mutex::new(ToolRegistry::new())));

    // Register each tool category and log the count
    let memory_tools = memory::definitions::all();
    tracing::info!("Registered {} memory tools", memory_tools.len());

    let experience_tools = experience::definitions::all();
    tracing::info!("Registered {} experience tools", experience_tools.len());

    let reflection_tools = reflection::definitions::all();
    tracing::info!("Registered {} reflection tools", reflection_tools.len());

    let search_tools = search::definitions::all();
    tracing::info!("Registered {} search tools", search_tools.len());

    let ingestor_tools = ingestor::definitions::all();
    tracing::info!("Registered {} ingestor tools", ingestor_tools.len());

    let agent_tools = agent::definitions::all();
    tracing::info!("Registered {} agent tools", agent_tools.len());

    let hypothesis_tools = hypothesis::definitions::all();
    tracing::info!("Registered {} hypothesis tools", hypothesis_tools.len());

    let exploration_tools = exploration::definitions::all();
    tracing::info!("Registered {} exploration tools", exploration_tools.len());

    let knowledge_tools = knowledge::definitions::all();
    tracing::info!("Registered {} knowledge tools", knowledge_tools.len());

    let planner_tools = planner::definitions::all();
    tracing::info!("Registered {} planner tools", planner_tools.len());

    let workflow_tools = workflow::definitions::all();
    tracing::info!("Registered {} workflow tools", workflow_tools.len());

    let skills_tools = skills::definitions::all();
    tracing::info!("Registered {} skills tools", skills_tools.len());

    // Collect all tools
    let all_tools = memory_tools
        .into_iter()
        .chain(experience_tools)
        .chain(reflection_tools)
        .chain(search_tools)
        .chain(ingestor_tools)
        .chain(agent_tools)
        .chain(hypothesis_tools)
        .chain(exploration_tools)
        .chain(knowledge_tools)
        .chain(planner_tools)
        .chain(workflow_tools)
        .chain(skills_tools)
        .collect();

    // Update registry using mutex lock with error handling
    match registry.lock() {
        Ok(mut reg) => {
            reg.tools = all_tools;
            tracing::info!("Total MCP tools registered: {}", reg.tools.len());
        }
        Err(poisoned) => {
            // Handle poisoned mutex gracefully
            let mut reg = poisoned.into_inner();
            reg.tools = all_tools;
            tracing::info!("Total MCP tools registered (recovered from poison): {}", reg.tools.len());
        }
    }
}

/// Get all registered tools
pub async fn get_tools_async() -> Vec<crate::bridge::mcp::McpTool> {
    // Use blocking lock inside async context (safe since it's only read)
    match TOOL_REGISTRY.get() {
        Some(registry) => {
            match registry.lock() {
                Ok(reg) => reg.tools.clone(),
                Err(poisoned) => {
                    // Return tools from poisoned state
                    let reg = poisoned.into_inner();
                    reg.tools.clone()
                }
            }
        }
        None => Vec::new(),  // Return empty vec when registry not initialized
    }
}
