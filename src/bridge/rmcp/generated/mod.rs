// src/bridge/rmcp/generated/mod.rs
// Tools module - each tool category is a separate, isolated module
//
// Architecture:
// - server_handler_impl.rs provides ServerHandler trait impl for McpServerHandler
// - Each *_tools.rs has its own impl McpServerHandler with #[tool_router] and #[tool] macros
// - Tools are registered via the #[tool] attribute and handled by #[tool_router]
// - If a tool has compile errors, disable via Cargo.toml feature
// - MCP loads first (via ServerHandler impl), tools load independently

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
