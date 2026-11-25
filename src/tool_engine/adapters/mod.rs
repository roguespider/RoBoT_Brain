//! Tool adapters — Per Architecture §13 (Tool Engine).
//! Adapters for integrating external tool systems.

pub mod cargo;
pub mod f5tts;
pub mod filesystem;
pub mod git;
pub mod sqlite;
pub mod web;
pub mod whisper;

pub use cargo::CargoAdapter;
pub use f5tts::F5TtsAdapter;
pub use filesystem::FileSystemAdapter;
pub use git::GitAdapter;
pub use sqlite::SqliteAdapter;
pub use web::WebAdapter;
pub use whisper::WhisperAdapter;

/// Tool adapter for external system integration.
#[derive(Default)]
pub struct ToolAdapter;

impl ToolAdapter {
    /// Create a new tool adapter.
    pub fn new() -> Self {
        Self
    }

    /// Adapt an external tool call into the tool engine format.
    pub fn adapt(&self, external_id: &str, params: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "external_id": external_id,
            "params": params,
            "adapted": true
        })
    }
}
