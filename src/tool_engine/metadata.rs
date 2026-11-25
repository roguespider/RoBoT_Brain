//! Tool metadata — per Architecture §13.3 (Tool Registry) and §13.4 (Tool Contracts).
//!
//! Describes a tool's identity, category, parameters, and capabilities for
//! registration, discovery, and validation.

use serde::{Deserialize, Serialize};

/// Tool category for routing and capability matching.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolCategory {
    Memory,
    Knowledge,
    Experience,
    Planning,
    Execution,
    Skill,
    System,
    Other,
}

impl Default for ToolCategory {
    fn default() -> Self {
        Self::Other
    }
}

/// Tool metadata describing registration, parameters, and capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// Unique tool identifier.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Tool category for routing.
    pub category: ToolCategory,
    /// JSON Schema for input parameters.
    pub input_schema: serde_json::Value,
    /// Whether the tool requires authentication.
    pub requires_auth: bool,
    /// Maximum invocations per minute per caller.
    pub max_invocations_per_minute: u32,
    /// Tags for discovery and filtering.
    pub tags: Vec<String>,
}

impl ToolMetadata {
    /// Create new tool metadata.
    pub fn new(
        name: String,
        description: String,
        category: ToolCategory,
        input_schema: serde_json::Value,
    ) -> Self {
        Self {
            name,
            description,
            category,
            input_schema,
            requires_auth: false,
            max_invocations_per_minute: 60,
            tags: Vec::new(),
        }
    }

    /// Set authentication requirement.
    pub fn with_auth(mut self, required: bool) -> Self {
        self.requires_auth = required;
        self
    }

    /// Set rate limit.
    pub fn with_rate_limit(mut self, max_per_minute: u32) -> Self {
        self.max_invocations_per_minute = max_per_minute;
        self
    }

    /// Add a tag.
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }
}
