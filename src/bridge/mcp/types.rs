// src/bridge/mcp/types.rs

//! MCP (Model Context Protocol) core types
//!
//! These types define the MCP protocol for communication between
//! clients and servers.

use serde::{Deserialize, Serialize};

/// MCP protocol version
#[allow(dead_code)]
pub const MCP_VERSION: &str = "2024-11-05";

/// MCP message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
pub enum McpMessage {
    Request(McpRequest),
    Response(McpResponse),
    Notification(McpNotification),
}

#[allow(dead_code)]
impl McpMessage {
    /// Check if this is a request message
    pub fn is_request(&self) -> bool {
        matches!(self, McpMessage::Request(_))
    }

    /// Check if this is a response message
    pub fn is_response(&self) -> bool {
        matches!(self, McpMessage::Response(_))
    }

    /// Check if this is a notification message
    pub fn is_notification(&self) -> bool {
        matches!(self, McpMessage::Notification(_))
    }
}

/// MCP request message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct McpRequest {
    pub method: String,
    pub params: Option<serde_json::Value>,
    pub id: String,
}

#[allow(dead_code)]
impl McpRequest {
    /// Create a new request
    pub fn new(method: &str, id: &str) -> Self {
        Self {
            method: method.to_string(),
            params: None,
            id: id.to_string(),
        }
    }

    /// Create a request with parameters
    pub fn with_params(mut self, params: serde_json::Value) -> Self {
        self.params = Some(params);
        self
    }
}

/// MCP response message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct McpResponse {
    pub result: Option<serde_json::Value>,
    pub error: Option<McpError>,
    pub id: String,
}

#[allow(dead_code)]
impl McpResponse {
    /// Create a successful response
    pub fn success(id: &str, result: serde_json::Value) -> Self {
        Self {
            result: Some(result),
            error: None,
            id: id.to_string(),
        }
    }

    /// Create an error response
    pub fn error(id: &str, error: McpError) -> Self {
        Self {
            result: None,
            error: Some(error),
            id: id.to_string(),
        }
    }

    /// Check if response is successful
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }
}

/// MCP error
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[allow(dead_code)]
impl McpError {
    /// Create a new error
    pub fn new(code: i32, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
            data: None,
        }
    }

    /// Create an error with data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

/// MCP notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct McpNotification {
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[allow(dead_code)]
impl McpNotification {
    /// Create a new notification
    pub fn new(method: &str) -> Self {
        Self {
            method: method.to_string(),
            params: None,
        }
    }

    /// Create a notification with parameters
    pub fn with_params(mut self, params: serde_json::Value) -> Self {
        self.params = Some(params);
        self
    }
}

/// Tool definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[allow(dead_code)]
impl McpTool {
    /// Create a new tool definition
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            input_schema: serde_json::json!({}),
        }
    }

    /// Create a tool with an input schema
    pub fn with_schema(mut self, schema: serde_json::Value) -> Self {
        self.input_schema = schema;
        self
    }
}

/// Resource definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

#[allow(dead_code)]
impl McpResource {
    /// Create a new resource
    pub fn new(uri: &str, name: &str) -> Self {
        Self {
            uri: uri.to_string(),
            name: name.to_string(),
            description: None,
            mime_type: None,
        }
    }
}

/// Prompt definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct McpPrompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<McpPromptArgument>,
}

/// Argument for a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct McpPromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

#[allow(dead_code)]
impl McpPromptArgument {
    /// Create a new prompt argument
    pub fn new(name: &str, required: bool) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            required,
        }
    }
}

/// Initialize request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct InitializeParams {
    pub protocol_version: String,
    pub capabilities: McpCapabilities,
    pub client_info: McpClientInfo,
}

#[allow(dead_code)]
impl InitializeParams {
    /// Create new initialize params
    pub fn new(client_info: McpClientInfo) -> Self {
        Self {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: McpCapabilities::default(),
            client_info,
        }
    }
}

/// Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct McpCapabilities {
    #[serde(default)]
    pub tools: Option<McpEmpty>,
    #[serde(default)]
    pub resources: Option<McpResourcesCapability>,
    #[serde(default)]
    pub prompts: Option<McpEmpty>,
    #[serde(default)]
    pub logging: Option<McpEmpty>,
}

#[allow(dead_code)]
impl McpCapabilities {
    /// Create capabilities with tools enabled
    pub fn with_tools() -> Self {
        Self {
            tools: Some(McpEmpty),
            resources: None,
            prompts: None,
            logging: None,
        }
    }

    /// Create capabilities with all features enabled
    pub fn all() -> Self {
        Self {
            tools: Some(McpEmpty),
            resources: Some(McpResourcesCapability::default()),
            prompts: Some(McpEmpty),
            logging: Some(McpEmpty),
        }
    }
}

impl Default for McpCapabilities {
    fn default() -> Self {
        Self::all()
    }
}

/// Empty capability marker
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct McpEmpty;

/// Resources capability
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct McpResourcesCapability {
    pub subscribe: Option<bool>,
    pub list_changed: Option<bool>,
}

impl Default for McpResourcesCapability {
    fn default() -> Self {
        Self {
            subscribe: Some(true),
            list_changed: Some(true),
        }
    }
}

/// Client information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct McpClientInfo {
    pub name: String,
    pub version: String,
}

#[allow(dead_code)]
impl McpClientInfo {
    /// Create new client info
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
        }
    }
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
}

impl McpServerInfo {
    /// Create new server info
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
        }
    }
}
