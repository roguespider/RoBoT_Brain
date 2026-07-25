//! MCP Integration Test Library for RoBoT Brain
//!
//! Comprehensive test suite that tests every MCP tool available in the compiled
//! RoBoT Brain server with pass/fail indicators.
//!
//! ## Running Tests
//! ```bash
//! cd robot_brain_test
//! cargo test
//! ```
//!
//! ## Running Specific Test Module
//! ```bash
//! cargo test connection
//! cargo test memory
//! cargo test knowledge
//! cargo test planner
//! cargo test workflow
//! cargo test agent
//! cargo test hypothesis
//! cargo test reflection
//! cargo test search
//! cargo test ingestor
//! cargo test e2e
//! cargo test error_handling
//! ```

use std::path::PathBuf;

// ============================================================================
// Public exports
// ============================================================================

pub mod common;
pub mod client;
pub mod tools;
pub mod tests;

pub use common::{ToolTestResult, TestSuiteSummary};
pub use client::McpTestClient;
pub use tools::{get_all_tool_names, get_tools_by_category};

// ============================================================================
// Server Path Detection
// ============================================================================

/// Get the path to the compiled MCP server executable
pub fn get_server_path() -> PathBuf {
    std::env::var("MCP_SERVER_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let release_path = PathBuf::from("../RoBoT_Brain/target/release/robot_brain");
            let debug_path = PathBuf::from("../RoBoT_Brain/target/debug/robot_brain");
            
            #[cfg(windows)]
            let release_path = release_path.with_extension("exe");
            #[cfg(windows)]
            let debug_path = debug_path.with_extension("exe");
            
            if release_path.exists() {
                release_path
            } else if debug_path.exists() {
                debug_path
            } else {
                panic!("MCP server not found! Build with `cargo build --release` in RoBoT_Brain first.");
            }
        })
}
