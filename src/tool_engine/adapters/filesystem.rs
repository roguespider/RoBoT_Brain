//! Filesystem adapter — per Architecture §13 (Tool Engine).
//! Provides file system operations for the tool engine using std::fs.

use serde_json;
use std::fs;
use std::path::Path;

/// File system adapter for file and directory operations.
#[derive(Default)]
pub struct FileSystemAdapter;

impl FileSystemAdapter {
    /// Create a new file system adapter.
    pub fn new() -> Self {
        Self
    }

    /// Adapt an external tool call into the tool engine format.
    pub fn adapt(&self, external_id: &str, params: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "external_id": external_id,
            "params": params,
            "adapter": "filesystem",
            "adapted": true
        })
    }

    /// Read the contents of a file at `path`.
    pub fn read_file(&self, path: &str) -> Result<String, String> {
        fs::read_to_string(path).map_err(|e| format!("Failed to read {path}: {e}"))
    }

    /// Write `content` to a file at `path`.
    pub fn write_file(&self, path: &str, content: &str) -> Result<(), String> {
        fs::write(path, content).map_err(|e| format!("Failed to write {path}: {e}"))
    }

    /// List the entries in a directory at `path`.
    pub fn list_dir(&self, path: &str) -> Result<Vec<String>, String> {
        let entries = fs::read_dir(path).map_err(|e| format!("Failed to list {path}: {e}"))?;
        let mut names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {e}"))?;
            names.push(entry.file_name().to_string_lossy().to_string());
        }
        Ok(names)
    }

    /// Check if a path exists.
    pub fn exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }
}
