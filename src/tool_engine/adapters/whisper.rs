//! Whisper adapter — per Architecture §13 (Tool Engine).
//! Provides speech-to-text operations via a Whisper CLI or library.

use serde_json;
use std::process::Command;

/// Whisper adapter for speech-to-text transcription.
#[derive(Default)]
pub struct WhisperAdapter;

impl WhisperAdapter {
    /// Create a new whisper adapter.
    pub fn new() -> Self {
        Self
    }

    /// Adapt an external tool call into the tool engine format.
    pub fn adapt(&self, external_id: &str, params: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "external_id": external_id,
            "params": params,
            "adapter": "whisper",
            "adapted": true
        })
    }

    /// Transcribe an audio file at `path` to text.
    pub fn transcribe(&self, path: &str) -> Result<String, String> {
        let output = Command::new("whisper")
            .arg(path)
            .output()
            .map_err(|e| format!("Failed to execute whisper: {e}"))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(format!(
                "whisper transcription failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Check if the whisper CLI is available.
    pub fn is_available(&self) -> bool {
        Command::new("whisper").arg("--help").output().is_ok()
    }
}
