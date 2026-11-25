//! Git adapter — per Architecture §13 (Tool Engine).
//! Provides Git version control operations using std::process::Command.

use serde_json;
use std::process::Command;

/// Git adapter for version control operations.
#[derive(Default)]
pub struct GitAdapter;

impl GitAdapter {
    /// Create a new git adapter.
    pub fn new() -> Self {
        Self
    }

    /// Adapt an external tool call into the tool engine format.
    pub fn adapt(&self, external_id: &str, params: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "external_id": external_id,
            "params": params,
            "adapter": "git",
            "adapted": true
        })
    }

    /// Clone a repository from `url` into `path`.
    pub fn clone(&self, url: &str, path: &str) -> Result<String, String> {
        let output = Command::new("git")
            .args(["clone", url, path])
            .output()
            .map_err(|e| format!("Failed to execute git clone: {e}"))?;
        if output.status.success() {
            Ok(format!("Cloned {url} into {path}"))
        } else {
            Err(format!(
                "git clone failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Get the status of a repository at `path`.
    pub fn status(&self, path: &str) -> Result<String, String> {
        let output = Command::new("git")
            .args(["-C", path, "status", "--porcelain"])
            .output()
            .map_err(|e| format!("Failed to execute git status: {e}"))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Get the current branch of a repository at `path`.
    pub fn branch(&self, path: &str) -> Result<String, String> {
        let output = Command::new("git")
            .args(["-C", path, "rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .map_err(|e| format!("Failed to execute git branch: {e}"))?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}
