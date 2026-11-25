//! Plugin manifest — per Architecture §13 (Tool Engine).
//! Describes plugin metadata for the tool engine plugin system.

use serde_json;

/// Plugin manifest describing a plugin's metadata.
#[derive(Debug, Clone)]
pub struct PluginManifest {
    /// Plugin name.
    pub name: String,
    /// Plugin version.
    pub version: String,
    /// Plugin description.
    pub description: String,
    /// Optional plugin author.
    pub author: Option<String>,
}

impl PluginManifest {
    /// Create a new plugin manifest.
    pub fn new(name: String, version: String, description: String) -> Self {
        Self {
            name,
            version,
            description,
            author: None,
        }
    }

    /// Set the author of the plugin.
    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }

    /// Convert the manifest to a JSON value.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "version": self.version,
            "description": self.description,
            "author": self.author
        })
    }
}

impl Default for PluginManifest {
    fn default() -> Self {
        Self::new(
            "unknown".to_string(),
            "0.1.0".to_string(),
            "No description".to_string(),
        )
    }
}
