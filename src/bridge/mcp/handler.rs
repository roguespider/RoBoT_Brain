// src/bridge/mcp/handler.rs

//! MCP protocol handler implementation using rmcp crate
//!
//! This module provides a complete MCP server handler implementation that
//! integrates with the rmcp crate for actual MCP protocol handling.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use rmcp::{
    model::{
        CallToolRequestParams, CallToolResult, CompleteRequestParams, CompleteResult,
        GetPromptRequestParams, GetPromptResult, Implementation,
        InitializeRequestParams, InitializeResult, ListPromptsResult,
        ListResourceTemplatesResult, ListResourcesResult, ListToolsResult,
        PaginatedRequestParams, ReadResourceRequestParams, ReadResourceResult, Resource,
        SetLevelRequestParams, SubscribeRequestParams, Tool,
    },
    service::{NotificationContext, RequestContext, RoleServer},
    ErrorData as McpError, ServerHandler,
};

/// Trait for MCP protocol handlers (simplified version for non-async use)
#[allow(dead_code)]
pub trait McpHandler: Send + Sync {
    /// Handle an MCP request (simplified synchronous version)
    fn handle_request_sync(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value>;
    /// Get server capabilities
    fn get_capabilities(&self) -> super::types::McpCapabilities;
    /// Get server info
    fn get_server_info(&self) -> super::types::McpServerInfo;
}

/// Tool executor trait for handling tool calls
#[allow(dead_code)]
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool with the given name and arguments
    fn execute(&self, tool_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value>;
}

/// Default tool executor that returns an error (to be extended)
#[allow(dead_code)]
pub struct DefaultToolExecutor;

impl ToolExecutor for DefaultToolExecutor {
    fn execute(&self, tool_name: &str, _arguments: serde_json::Value) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": format!("Tool '{}' not found. No tools are registered.", tool_name)
            }]
        }))
    }
}

/// MCP Server handler that implements the rmcp ServerHandler trait
#[allow(dead_code)]
pub struct McpServerHandler {
    name: String,
    version: String,
    tools: Vec<Tool>,
    resources: Vec<Resource>,
    /// Actual content for each resource, keyed by URI
    resource_contents: HashMap<String, String>,
    tool_executor: Arc<dyn ToolExecutor>,
}

impl McpServerHandler {
    /// Create a new MCP server handler
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            tools: Vec::new(),
            resources: Vec::new(),
            resource_contents: HashMap::new(),
            tool_executor: Arc::new(DefaultToolExecutor),
        }
    }

    /// Create a handler with a custom tool executor
    pub fn with_executor(name: &str, version: &str, executor: Arc<dyn ToolExecutor>) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            tools: Vec::new(),
            resources: Vec::new(),
            resource_contents: HashMap::new(),
            tool_executor: executor,
        }
    }

    /// Add a tool to the handler
    pub fn add_tool(&mut self, tool: Tool) {
        self.tools.push(tool);
    }

    /// Add a tool from a simple definition
    pub fn add_simple_tool(&mut self, name: &str, description: &str, input_schema: serde_json::Value) {
        let schema: serde_json::Map<String, serde_json::Value> = 
            input_schema.as_object().cloned().unwrap_or_default();
        self.tools.push(Tool::new(
            name.to_string(),
            description.to_string(),
            Arc::new(schema),
        ));
    }

    /// Set the tools list
    pub fn set_tools(&mut self, tools: Vec<Tool>) {
        self.tools = tools;
    }

    /// Get the list of tools
    pub fn get_tools(&self) -> &[Tool] {
        &self.tools
    }

    /// Add a resource to the handler
    pub fn add_resource(&mut self, resource: Resource) {
        self.resources.push(resource);
    }

    /// Add a resource with its content to the handler
    pub fn add_resource_with_content(&mut self, resource: Resource, content: String) {
        let uri = resource.uri.clone();
        self.resources.push(resource);
        self.resource_contents.insert(uri, content);
    }

    /// Set the resources list (clears any associated content)
    pub fn set_resources(&mut self, resources: Vec<Resource>) {
        self.resources = resources;
        // Clear contents that are no longer associated with resources
        let uris: std::collections::HashSet<_> = self.resources.iter().map(|r| r.uri.clone()).collect();
        self.resource_contents.retain(|uri, _| uris.contains(uri));
    }

    /// Set resources with their contents
    pub fn set_resources_with_contents(&mut self, resources: Vec<Resource>, contents: HashMap<String, String>) {
        self.resources = resources;
        self.resource_contents = contents;
    }

    /// Get the list of resources
    pub fn get_resources(&self) -> &[Resource] {
        &self.resources
    }

    /// Get resource content by URI
    pub fn get_resource_content(&self, uri: &str) -> Option<&str> {
        self.resource_contents.get(uri).map(|s| s.as_str())
    }

    /// Get server info as Implementation
    fn implementation(&self) -> Implementation {
        Implementation::new(&self.name, &self.version)
    }
}

impl ServerHandler for McpServerHandler {
    /// Handle initialize request
    async fn initialize(
        &self,
        _request: InitializeRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        Ok(InitializeResult::new(rmcp::model::ServerCapabilities::default())
            .with_server_info(self.implementation()))
    }

    /// Handle ping
    async fn ping(&self, _context: RequestContext<RoleServer>) -> Result<(), McpError> {
        Ok(())
    }

    /// List all tools
    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult::with_all_items(self.tools.clone()))
    }

    /// Call a tool
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_name = &request.name;
        let arguments: serde_json::Value = request.arguments.into();

        match self.tool_executor.execute(tool_name, arguments) {
            Ok(result) => Ok(CallToolResult::success(vec![
                rmcp::model::ContentBlock::Text(rmcp::model::TextContent::new(result.to_string())),
            ])),
            Err(e) => Ok(CallToolResult::error(vec![
                rmcp::model::ContentBlock::Text(rmcp::model::TextContent::new(format!(
                    "Error executing tool '{}': {}",
                    tool_name, e
                ))),
            ])),
        }
    }

    /// Get a tool by name (for tool name validation)
    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tools.iter().find(|t| &t.name == name).cloned()
    }

    /// List resources
    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult::with_all_items(self.resources.clone()))
    }

    /// List resource templates
    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult::with_all_items(vec![]))
    }

    /// Read a resource
    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let uri = &request.uri;
        if let Some(resource) = self.resources.iter().find(|r| &r.uri == uri) {
            // Try to get actual content from our storage
            let content = self.resource_contents.get(uri);
            
            // If no content stored, fall back to metadata description
            let text_content = if let Some(content) = content {
                content.clone()
            } else {
                // Try to read from file system if URI is a file path
                if uri.starts_with("file://") {
                    let path = &uri[7..]; // Remove "file://" prefix
                    std::fs::read_to_string(path)
                        .unwrap_or_else(|_| format!("Content for resource: {}", uri))
                } else {
                    format!("Content for resource: {}", uri)
                }
            };
            
            Ok(ReadResourceResult::new(vec![rmcp::model::ResourceContents::text(
                text_content,
                uri,
            )
            .with_mime_type(resource.mime_type.as_deref().unwrap_or("text/plain"))]))
        } else {
            Err(McpError::invalid_params(format!("Resource not found: {}", uri), None))
        }
    }

    /// Subscribe to resource updates
    async fn subscribe(
        &self,
        _request: SubscribeRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<(), McpError> {
        // No-op: subscriptions not supported without a notification system
        Ok(())
    }

    /// Unsubscribe from resource updates
    async fn unsubscribe(
        &self,
        _request: rmcp::model::UnsubscribeRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<(), McpError> {
        Ok(())
    }

    /// List prompts
    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult::with_all_items(vec![]))
    }

    /// Get a prompt
    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        Err(McpError::invalid_params(format!("Prompt '{}' not found", request.name), None))
    }

    /// Complete a request
    async fn complete(
        &self,
        _request: CompleteRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CompleteResult, McpError> {
        let completion = rmcp::model::CompletionInfo::with_all_values(vec![])
            .expect("empty values should be valid");
        Ok(CompleteResult::new(completion))
    }

    /// Set logging level (deprecated by MCP protocol)
    #[allow(deprecated)]
    async fn set_level(
        &self,
        _request: SetLevelRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<(), McpError> {
        Ok(())
    }

    /// Handle initialized notification
    async fn on_initialized(&self, _context: NotificationContext<RoleServer>) {
        tracing::info!("MCP server '{}' initialized successfully", self.name);
    }

    /// Handle roots list changed notification
    async fn on_roots_list_changed(&self, _context: NotificationContext<RoleServer>) {
        // No-op
    }
}

// Alias for backwards compatibility
#[allow(dead_code)]
pub type DefaultMcpHandler = McpServerHandler;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_creation() {
        let handler = McpServerHandler::new("test-server", "1.0.0");
        assert_eq!(handler.get_tools().len(), 0);
        assert_eq!(handler.get_resources().len(), 0);
    }

    #[test]
    fn test_add_tool() {
        let mut handler = McpServerHandler::new("test-server", "1.0.0");
        handler.add_simple_tool("test_tool", "A test tool", serde_json::json!({}));
        assert_eq!(handler.get_tools().len(), 1);
        assert_eq!(handler.get_tools()[0].name, "test_tool");
    }

    #[test]
    fn test_get_tool() {
        let mut handler = McpServerHandler::new("test-server", "1.0.0");
        handler.add_simple_tool("my_tool", "My tool", serde_json::json!({}));
        
        assert!(handler.get_tool("my_tool").is_some());
        assert!(handler.get_tool("nonexistent").is_none());
    }

    #[test]
    fn test_set_tools() {
        let mut handler = McpServerHandler::new("test-server", "1.0.0");
        handler.add_simple_tool("tool1", "First tool", serde_json::json!({}));
        handler.add_simple_tool("tool2", "Second tool", serde_json::json!({}));
        assert_eq!(handler.get_tools().len(), 2);
    }

    #[test]
    fn test_resources() {
        let mut handler = McpServerHandler::new("test-server", "1.0.0");
        assert_eq!(handler.get_resources().len(), 0);
        
        // Resources would need to be added via Resource type
        // For now just verify the field exists and works
        handler.set_resources(vec![]);
        assert_eq!(handler.get_resources().len(), 0);
    }
}
