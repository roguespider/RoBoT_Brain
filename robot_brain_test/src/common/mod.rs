//! Common utilities shared across the test suite

use std::path::PathBuf;

pub mod types;

pub use types::{ToolTestResult, TestSuiteSummary};

/// Get the path to the compiled MCP server executable
pub fn get_server_path() -> PathBuf {
    std::env::var("MCP_SERVER_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            // Try multiple possible paths relative to the workspace root
            let possible_paths = vec![
                // From workspace root
                PathBuf::from("target/release/robot_brain"),
                PathBuf::from("target/debug/robot_brain"),
                // From robot_brain_test directory
                PathBuf::from("../target/release/robot_brain"),
                PathBuf::from("../target/debug/robot_brain"),
                // Absolute path fallback
                PathBuf::from("/workspace/project/RoBoT_Brain/target/release/robot_brain"),
                PathBuf::from("/workspace/project/RoBoT_Brain/target/debug/robot_brain"),
            ];
            
            #[cfg(windows)]
            let windows_possible: Vec<PathBuf> = possible_paths.iter()
                .map(|p| p.with_extension("exe"))
                .collect();
            
            #[cfg(windows)]
            let all_paths: Vec<PathBuf> = possible_paths.into_iter()
                .chain(windows_possible)
                .collect();
            
            #[cfg(not(windows))]
            let all_paths = possible_paths;
            
            for path in &all_paths {
                if path.exists() {
                    return path.clone();
                }
            }
            
            panic!("MCP server not found! Build with `cargo build --release` in RoBoT_Brain first. Searched paths: {:?}", all_paths);
        })
}
