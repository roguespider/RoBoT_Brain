//! F5-TTS adapter — per Architecture §13 (Tool Engine).
//! Provides text-to-speech synthesis via an F5-TTS CLI or library.

use serde_json;
use std::process::Command;

/// F5-TTS adapter for text-to-speech synthesis.
#[derive(Default)]
pub struct F5TtsAdapter;

impl F5TtsAdapter {
    /// Create a new F5-TTS adapter.
    pub fn new() -> Self {
        Self
    }

    /// Adapt an external tool call into the tool engine format.
    pub fn adapt(&self, external_id: &str, params: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "external_id": external_id,
            "params": params,
            "adapter": "f5tts",
            "adapted": true
        })
    }

    /// Synthesize `text` to speech and save to `output_path`.
    pub fn synthesize(&self, text: &str, output_path: &str) -> Result<(), String> {
        let output = Command::new("f5tts")
            .args(["--text", text, "--output", output_path])
            .output()
            .map_err(|e| format!("Failed to execute f5tts: {e}"))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "f5tts synthesis failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Check if the f5tts CLI is available.
    pub fn is_available(&self) -> bool {
        Command::new("f5tts").arg("--help").output().is_ok()
    }
}
