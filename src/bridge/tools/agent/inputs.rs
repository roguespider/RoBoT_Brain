
// src/tools/agent/inputs.rs
// Input structures for agent tools

use serde::{Deserialize, Serialize};

/// Tool input for getting workflow rules (MUST be called first)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetWorkflowInput {
    pub purpose: Option<String>,
}

/// Tool input for listing available tools
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListToolsInput {
    pub filter: Option<String>,
}

/// Tool input for getting tool details
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetToolInput {
    pub name: String,
}

/// Tool input for connecting to an MCP server
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConnectMcpServerInput {
    pub name: String,
    pub command: String,
    pub args: Option<Vec<String>>,
}

/// Tool input for calling an MCP tool
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CallMcpToolInput {
    pub tool_name: String,
    /// JSON-encoded arguments as a string (e.g., "{\"key\": \"value\"}")
    pub arguments: Option<String>,
}

/// Tool input for running the goal-driven agent loop (Architecture §5.7).
/// Given a goal, the agent plans, retrieves memory/knowledge/experiences,
/// evaluates action confidence, checks the safety gate, and records the
/// outcome as a new experience — closing the cognitive loop.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RunAgentGoalInput {
    /// The goal to pursue.
    pub goal: String,
    /// Minimum confidence required to act (0.0–1.0). If no plan step
    /// meets this threshold, the agent abstains and records why.
    pub confidence_threshold: Option<f32>,
}
