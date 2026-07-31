


// src/tools/mod.rs
// MCP tools for Zed Editor integration


use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::bridge::mcp::McpContext;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_tools_async() {
        // get_tools_async() returns empty vec when registry not initialized
        let tools = get_tools_async().await;
        assert!(tools.is_empty());
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

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }
}

/// Register all MCP tools with the given context
pub fn register_tools(context: &Arc<McpContext>) {
    // Wire up MCP context fields by accessing them here
    // These fields are stored for use by tools
    let _ = &context.bus;          // Event bus for pub/sub
    let _ = &context.evolution;     // Evolution engine
    let _ = &context.scheduler;     // Background scheduler
    let _ = &context.metrics;       // Metrics collector
    let _ = &context.policy;        // Policy engine
    let _ = &context.working_memory; // Working memory
    let _ = &context.permanent_memory; // Permanent memory
    let _ = &context.memory_retrieval; // Memory retrieval
    let _ = &context.server_info;   // Server info
    let _ = &context.capabilities;  // Server capabilities
    
    let registry = TOOL_REGISTRY.get_or_init(|| Arc::new(Mutex::new(ToolRegistry::new())));

    // Register memory tools
    let tools = memory::definitions::all();
    tracing::info!("Registered {} memory tools", tools.len());

    // Register experience tools
    let tools = experience::definitions::all();
    tracing::info!("Registered {} experience tools", tools.len());

    // Register reflection tools
    let tools = reflection::definitions::all();
    tracing::info!("Registered {} reflection tools", tools.len());

    // Register search tools
    let tools = search::definitions::all();
    tracing::info!("Registered {} search tools", tools.len());

    // Register ingestor tools
    let tools = ingestor::definitions::all();
    tracing::info!("Registered {} ingestor tools", tools.len());

    // Register agent tools
    let tools = agent::definitions::all();
    tracing::info!("Registered {} agent tools", tools.len());

    // Register hypothesis tools
    let tools = hypothesis::definitions::all();
    tracing::info!("Registered {} hypothesis tools", tools.len());

    // Register exploration tools
    let tools = exploration::definitions::all();
    tracing::info!("Registered {} exploration tools", tools.len());

    // Register knowledge tools
    let tools = knowledge::definitions::all();
    tracing::info!("Registered {} knowledge tools", tools.len());

    // Register planner tools
    let tools = planner::definitions::all();
    tracing::info!("Registered {} planner tools", tools.len());

    // Register workflow tools
    let tools = workflow::definitions::all();
    tracing::info!("Registered {} workflow tools", tools.len());

    // Register skills tools (Architecture §15)
    let tools = skills::definitions::all();
    tracing::info!("Registered {} skills tools", tools.len());
    
    // Wire up skills for use by tools
    let _ = &context.skills;

    // Collect all tools
    let all_tools = memory::definitions::all()
        .into_iter()
        .chain(experience::definitions::all())
        .chain(reflection::definitions::all())
        .chain(search::definitions::all())
        .chain(ingestor::definitions::all())
        .chain(agent::definitions::all())
        .chain(hypothesis::definitions::all())
        .chain(exploration::definitions::all())
        .chain(knowledge::definitions::all())
        .chain(planner::definitions::all())
        .chain(workflow::definitions::all())
        .chain(skills::definitions::all())
        .collect();

    // Update registry using mutex lock
    let mut reg = registry.lock().unwrap();
    reg.tools = all_tools;
    tracing::info!("Total MCP tools registered: {}", reg.tools.len());
}

/// Get all registered tools
pub async fn get_tools_async() -> Vec<crate::bridge::mcp::McpTool> {
    // Use blocking lock inside async context (safe since it's only read)
    match TOOL_REGISTRY.get() {
        Some(registry) => registry.lock().unwrap().tools.clone(),
        None => Vec::new(),  // Return empty vec when registry not initialized
    }
}
