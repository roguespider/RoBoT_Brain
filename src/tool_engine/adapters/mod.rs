//! Tool adapters — Per Architecture §13 (Tool Engine).
//! Adapters for integrating external tool systems.

/// Tool adapter for external system integration.
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
