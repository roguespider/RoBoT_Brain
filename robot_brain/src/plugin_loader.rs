//! Plugin loader for dynamically loading tool crates at runtime
//! 
//! Uses dlopen to load .so files as plugins. If a plugin fails to load,
//! the MCP server continues with the remaining plugins.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use libloading::{Library, Symbol};

use tools_core::{ToolDefinition, ToolPlugin, ToolResult};

/// Loaded plugin handle
pub struct LoadedPlugin {
    pub name: String,
    pub tools: Vec<ToolDefinition>,
    pub library: Library,
}

type GetPluginFn = unsafe extern "C" fn() -> *mut dyn ToolPlugin;

/// Plugin manager - loads and manages tool plugins
pub struct PluginManager {
    plugins: HashMap<String, LoadedPlugin>,
}

impl PluginManager {
    pub fn new() -> Self {
        PluginManager {
            plugins: HashMap::new(),
        }
    }

    /// Load all plugins from a directory
    pub fn load_from_directory(&mut self, plugins_dir: &Path) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if !plugins_dir.exists() {
            return Err(format!("Plugins directory does not exist: {:?}", plugins_dir).into());
        }

        let entries = fs::read_dir(plugins_dir)
            .map_err(|e| format!("Failed to read plugins directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("so") {
                match self.load_plugin(&path) {
                    Ok(name) => {
                        tracing::info!("Loaded plugin: {}", name);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load plugin {:?}: {}", path, e);
                        // Continue loading other plugins - this is the key feature!
                    }
                }
            }
        }

        if self.plugins.is_empty() {
            return Err("No plugins loaded".into());
        }

        Ok(())
    }

    /// Load a single plugin from a .so file
    fn load_plugin(&mut self, path: &Path) -> std::result::Result<String, Box<dyn std::error::Error>> {
        // Library::new is unsafe and requires an unsafe block
        let library = unsafe {
            Library::new(path)
                .map_err(|e| format!("Failed to load library: {}", e))?
        };

        unsafe {
            let get_plugin: Symbol<GetPluginFn> = library
                .get(b"get_plugin")
                .map_err(|e| format!("Failed to get get_plugin symbol: {}", e))?;

            let plugin_ptr = get_plugin();
            if plugin_ptr.is_null() {
                return Err("get_plugin returned null".into());
            }

            // Create a Box to take ownership of the plugin
            let plugin = Box::from_raw(plugin_ptr);
            let name = plugin.name().to_string();

            // Initialize the plugin
            if let Err(e) = plugin.init() {
                tracing::warn!("Plugin {} init failed: {}", name, e);
            }

            // Get tools - this returns owned data so we don't need the plugin after this
            let tools = plugin.tools();

            // Drop the plugin Box now - we have all the data we need.
            // The library is kept alive by being stored in LoadedPlugin.
            drop(plugin);

            self.plugins.insert(name.clone(), LoadedPlugin {
                name: name.clone(),
                tools,
                library,
            });

            Ok(name)
        }
    }

    /// Get all tools from all plugins
    pub fn all_tools(&self) -> Vec<ToolDefinition> {
        self.plugins.values()
            .flat_map(|p| p.tools.clone())
            .collect()
    }

    /// Execute a tool by fully qualified name (plugin::tool)
    pub fn execute(&self, full_name: &str, input: serde_json::Value) -> ToolResult {
        let parts: Vec<&str> = full_name.splitn(2, "::").collect();
        if parts.len() != 2 {
            return Err(format!("Invalid tool name format: {}. Expected 'plugin::tool'", full_name));
        }

        let plugin_name = parts[0];
        let tool_name = parts[1];

        // Look up the plugin and find the tool
        let plugin = self.plugins.get(plugin_name)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_name))?;

        // Find the tool in this plugin
        let tool = plugin.tools.iter()
            .find(|t| t.name == tool_name)
            .ok_or_else(|| format!("Tool '{}' not found in plugin '{}'", tool_name, plugin_name))?;

        // Return placeholder response with actual tool metadata
        Ok(serde_json::json!({
            "status": "placeholder",
            "plugin": plugin_name,
            "tool": tool_name,
            "tool_description": tool.description,
            "input": input
        }))
    }

    /// Get the number of loaded plugins
    pub fn count(&self) -> usize {
        self.plugins.len()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
