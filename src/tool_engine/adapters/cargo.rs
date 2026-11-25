//! Cargo adapter — per Architecture §13 (Tool Engine).
//! Provides cargo/Rust package operations using std::process::Command.

use serde_json;
use std::process::Command;

/// Cargo adapter for Rust package operations.
#[derive(Default)]
pub struct CargoAdapter;

impl CargoAdapter {
    /// Create a new cargo adapter.
    pub fn new() -> Self {
        Self
    }

    /// Adapt an external tool call into the tool engine format.
    pub fn adapt(&self, external_id: &str, params: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "external_id": external_id,
            "params": params,
            "adapter": "cargo",
            "adapted": true
        })
    }

    /// Run `cargo build` in the given directory.
    pub fn build(&self, path: &str) -> Result<String, String> {
        let output = Command::new("cargo")
            .arg("build")
            .current_dir(path)
            .output()
            .map_err(|e| format!("Failed to execute cargo build: {e}"))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(format!(
                "cargo build failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Run `cargo test` in the given directory.
    pub fn test(&self, path: &str) -> Result<String, String> {
        let output = Command::new("cargo")
            .arg("test")
            .current_dir(path)
            .output()
            .map_err(|e| format!("Failed to execute cargo test: {e}"))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(format!(
                "cargo test failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Run `cargo check` in the given directory.
    pub fn check(&self, path: &str) -> Result<String, String> {
        let output = Command::new("cargo")
            .arg("check")
            .current_dir(path)
            .output()
            .map_err(|e| format!("Failed to execute cargo check: {e}"))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(format!(
                "cargo check failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
}
