//! Plugin lifecycle — per Architecture §13 (Tool Engine).
//! Manages plugin lifecycle (start, stop, enable, disable).

use serde_json;

/// Plugin lifecycle manager for tracking plugin states.
#[derive(Default)]
pub struct PluginLifecycle {
    enabled: bool,
}

impl PluginLifecycle {
    /// Create a new plugin lifecycle manager.
    pub fn new() -> Self {
        Self { enabled: false }
    }

    /// Start the plugin.
    pub fn start(&mut self) {
        self.enabled = true;
    }

    /// Stop the plugin.
    pub fn stop(&mut self) {
        self.enabled = false;
    }

    /// Check if the plugin is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Adapt an external tool call into the tool engine format.
    pub fn adapt(&self, external_id: &str, params: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "external_id": external_id,
            "params": params,
            "adapter": "plugin_lifecycle",
            "adapted": true
        })
    }
}
