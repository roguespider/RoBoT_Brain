//! Tool plugins — Per Architecture §13 (Tool Engine).
//! Plugin system for extensible tool capabilities.

use serde_json::Value;

/// Tool plugin trait for extensible capabilities.
/// Per Architecture §13: plugins extend the tool engine with custom logic.
pub trait ToolPlugin {
    /// Plugin name.
    fn name(&self) -> &str;

    /// Execute the plugin with given parameters.
    fn execute(&self, params: Value) -> Value;
}

/// A simple plugin implementation.
pub struct SimplePlugin {
    name: String,
}

impl SimplePlugin {
    /// Create a new simple plugin.
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl ToolPlugin for SimplePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, params: Value) -> Value {
        serde_json::json!({
            "plugin": self.name,
            "params": params,
            "status": "executed"
        })
    }
}
