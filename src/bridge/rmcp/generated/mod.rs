// src/bridge/rmcp/generated/mod.rs
// Tools module - each tool category is a separate, isolated module
//
// Each *_tools.rs file is a standalone module that contains its own impl block.
// If one tool has compile errors, it can be disabled via Cargo.toml features.
// MCP loads successfully with the remaining tools.

mod server_handler_impl;

// Each tool module can be disabled via Cargo.toml features
// If a tool has compile errors, disable its feature in Cargo.toml
// This ensures one tool cannot block MCP from loading or other tools from working

#[cfg(feature = "tools-memory")]
mod memory_tools;

#[cfg(feature = "tools-experience")]
mod experience_tools;

#[cfg(feature = "tools-reflection")]
mod reflection_tools;

#[cfg(feature = "tools-search")]
mod search_tools;

#[cfg(feature = "tools-ingestor")]
mod ingestor_tools;

#[cfg(feature = "tools-agent")]
mod agent_tools;

#[cfg(feature = "tools-hypothesis")]
mod hypothesis_tools;

#[cfg(feature = "tools-knowledge")]
mod knowledge_tools;

#[cfg(feature = "tools-planner")]
mod planner_tools;

#[cfg(feature = "tools-workflow")]
mod workflow_tools;

#[cfg(feature = "tools-exploration")]
mod exploration_tools;

#[cfg(feature = "tools-skills")]
mod skills_tools;
