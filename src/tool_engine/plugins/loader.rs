//! Plugin loader — per Architecture §13 (Tool Engine).
//! Loads plugins into the tool engine.

use serde_json;
use std::collections::HashMap;

/// Plugin loader for discovering and loading plugins.
pub struct PluginLoader {
    plugins: HashMap<String, String>,
}

impl PluginLoader {
    /// Create a new plugin loader.
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a plugin by name and path.
    pub fn register(&mut self, name: String, path: String) {
        self.plugins.insert(name, path);
    }

    /// Load a plugin by name, returning its registered path.
    pub fn load(&self, name: &str) -> Result<String, String> {
        self.plugins
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Plugin {name} not found"))
    }

    /// List all registered plugin names.
    pub fn list_plugins(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Check if a plugin is registered.
    pub fn contains(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    /// Adapt an external tool call into the tool engine format.
    pub fn adapt(&self, external_id: &str, params: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "external_id": external_id,
            "params": params,
            "adapter": "plugin_loader",
            "adapted": true
        })
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}
