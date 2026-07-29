#![allow(dead_code)]

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
