// src/bridge/rmcp/generated/mod.rs
// Tools module - each tool category is a separate, isolated module
//
// If one tool category has a compile error, disable it in Cargo.toml features.
// MCP loads with remaining tools.

use crate::bridge::rmcp::types::McpServerHandler;
use crate::bridge::rmcp::helpers::{tool_output_to_content, enforcement_error_to_content};

// Each tool module can be disabled via Cargo.toml features
// If a tool has compile errors, comment out its feature in Cargo.toml

#[cfg(feature = "tools-agent")]
pub mod agent_tools;

#[cfg(feature = "tools-experience")]
pub mod experience_tools;

#[cfg(feature = "tools-exploration")]
pub mod exploration_tools;

#[cfg(feature = "tools-hypothesis")]
pub mod hypothesis_tools;

#[cfg(feature = "tools-ingestor")]
pub mod ingestor_tools;

#[cfg(feature = "tools-knowledge")]
pub mod knowledge_tools;

#[cfg(feature = "tools-memory")]
pub mod memory_tools;

#[cfg(feature = "tools-planner")]
pub mod planner_tools;

#[cfg(feature = "tools-reflection")]
pub mod reflection_tools;

#[cfg(feature = "tools-search")]
pub mod search_tools;

#[cfg(feature = "tools-skills")]
pub mod skills_tools;

#[cfg(feature = "tools-workflow")]
pub mod workflow_tools;

// ServerHandler impl - separate file for clarity
include!("impl_tools.rs");
