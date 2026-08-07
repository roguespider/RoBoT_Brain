//! MCP Message Types
//!
//! Core message types for MCP protocol communication.

use serde::{Deserialize, Serialize};

/// MCP message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpMessage {
    Request(McpRequest),
    Response(McpResponse),
    Notification(McpNotification),
}

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
pub struct McpRequest {
    pub method: String,
    pub params: Option<serde_json::Value>,
    pub id: String,
}

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
pub struct McpResponse {
    pub result: Option<serde_json::Value>,
    pub error: Option<super::error::McpError>,
    pub id: String,
}

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
    pub fn error(id: &str, error: super::error::McpError) -> Self {
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

/// MCP notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpNotification {
    pub method: String,
    pub params: Option<serde_json::Value>,
}

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
