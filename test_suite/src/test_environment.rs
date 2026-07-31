



//! Test Environment Setup Module
//! 
//! Sets up the test environment with database and files_to_import folder
//! for comprehensive MCP tool testing.

use std::path::PathBuf;
use std::fs;

/// Test environment configuration
pub struct TestEnvironment {
    /// Root directory for test environment
    pub root_dir: PathBuf,
    /// Path to test server executable
    pub server_path: PathBuf,
    /// Path to files_to_import folder
    pub files_folder: PathBuf,
}

impl TestEnvironment {
    /// Create a new test environment
    pub fn new(root_dir: PathBuf, server_path: PathBuf) -> Self {
        let files_folder = root_dir.join("files_to_import");
        Self {
            root_dir,
            server_path,
            files_folder,
        }
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        // Clean up test environment on drop
        if self.root_dir.exists() {
            let _ = fs::remove_dir_all(&self.root_dir);
        }
    }
}
