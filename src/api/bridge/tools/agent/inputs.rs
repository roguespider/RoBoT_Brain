// src/tools/agent/inputs.rs
// Input structures for agent tools

use serde::{Deserialize, Serialize};

/// Tool input for getting workflow rules (MUST be called first)
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetWorkflowInput {
    pub purpose: Option<String>,
}
impl std::fmt::Debug for GetWorkflowInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GetWorkflowInput")
            .field("purpose", &self.purpose)
            .finish()
    }
}

/// Tool input for listing available tools
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListToolsInput {
    pub filter: Option<String>,
}
impl std::fmt::Debug for ListToolsInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ListToolsInput")
            .field("filter", &self.filter)
            .finish()
    }
}

/// Tool input for getting tool details
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetToolInput {
    pub name: String,
}
impl std::fmt::Debug for GetToolInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GetToolInput")
            .field("name", &self.name)
            .finish()
    }
}

/// Tool input for connecting to an MCP server
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConnectMcpServerInput {
    pub name: String,
    pub command: String,
    pub args: Option<Vec<String>>,
}
impl std::fmt::Debug for ConnectMcpServerInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectMcpServerInput")
            .field("name", &self.name)
            .field("command", &self.command)
            .field("args", &self.args)
            .finish()
    }
}

/// Tool input for calling an MCP tool
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CallMcpToolInput {
    pub tool_name: String,
    /// JSON-encoded arguments as a string (e.g., "{\"key\": \"value\"}")
    pub arguments: Option<String>,
}
impl std::fmt::Debug for CallMcpToolInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallMcpToolInput")
            .field("tool_name", &self.tool_name)
            .field("arguments", &self.arguments)
            .finish()
    }
}

/// Tool input for running the goal-driven agent loop (Architecture §5.7).
/// Given a goal, the agent plans, retrieves memory/knowledge/experiences,
/// evaluates action confidence, checks the safety gate, and records the
/// outcome as a new experience — closing the cognitive loop.
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RunAgentGoalInput {
    /// The goal to pursue.
    pub goal: String,
    /// Minimum confidence required to act (0.0–1.0). If no plan step
    /// meets this threshold, the agent abstains and records why.
    pub confidence_threshold: Option<f32>,
}
impl std::fmt::Debug for RunAgentGoalInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunAgentGoalInput")
            .field("goal", &self.goal)
            .field("confidence_threshold", &self.confidence_threshold)
            .finish()
    }
}
