//! Core traits and types for RoBoT Brain tools
//! 
//! Each tool crate implements ToolPlugin and is loaded at runtime.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Result type for tool execution
pub type ToolResult = Result<Value, String>;

/// Trait that all tool plugins must implement
pub trait ToolPlugin: Send + Sync {
    /// Get the tool category name (e.g., "memory", "skills")
    fn name(&self) -> &str;
    
    /// Get all tools provided by this plugin
    fn tools(&self) -> Vec<ToolDefinition>;
    
    /// Execute a tool by name
    fn execute(&self, tool_name: &str, input: Value) -> ToolResult;
    
    /// Initialize the plugin (called after loading)
    fn init(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Tool definition for MCP registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Plugin handle returned by the plugin loader
pub struct PluginHandle {
    pub name: String,
    pub tools: Vec<ToolDefinition>,
    pub execute_fn: Box<dyn Fn(&str, Value) -> ToolResult + Send + Sync>,
}

/// Macro to export a plugin from a crate
#[macro_export]
macro_rules! export_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn get_plugin() -> *mut dyn $crate::ToolPlugin {
            Box::into_raw(Box::new(<$plugin_type>::new()))
        }
    };
}
