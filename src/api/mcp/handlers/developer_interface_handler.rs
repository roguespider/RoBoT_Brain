// src/api/mcp/handlers/developer_interface_handler.rs
// Developer Interface tools handler - handles MCP tools for the developer interface
// (Architecture Chapter 28).

use crate::api::developer_interface::full::{
    ExplorerTarget, explore_cognitive_system, inspect_background_workers,
    inspect_learning_evolution, inspect_memory_by_layer, inspect_strategic_learning,
};
use crate::api::mcp::McpContext;
use crate::api::mcp::handlers::{HandlerError, HandlerInitResult, ToolHandler};
use std::sync::Arc;

/// Handler for developer interface tools (inspection, exploration, management).
#[derive(Clone)]
pub struct DeveloperInterfaceToolsHandler {
    context: Arc<McpContext>,
}

impl DeveloperInterfaceToolsHandler {
    /// Create a new developer interface tools handler.
    pub fn new(context: Arc<McpContext>) -> HandlerInitResult<Self> {
        Ok(Self { context })
    }

    /// Execute a control command on the developer interface.
    pub async fn execute_control_command(
        &self,
        name: &str,
    ) -> Result<crate::api::bridge::tools::ToolOutput, HandlerError> {
        use crate::api::developer_interface::ControlCommand;

        let cmd = ControlCommand::new(name);
        let summary = format!("Executed command: {}", cmd.name);
        Ok(crate::api::bridge::tools::ToolOutput::success(
            serde_json::json!({
                "id": cmd.id,
                "name": cmd.name,
                "parameters": cmd.parameters,
                "timestamp": cmd.timestamp,
                "summary": summary,
            }),
        ))
    }

    /// Explore a cognitive subsystem.
    pub async fn explore_cognitive_system_tool(
        &self,
        target: &str,
    ) -> Result<crate::api::bridge::tools::ToolOutput, HandlerError> {
        let explorer_target = match target {
            "Memory" => ExplorerTarget::Memory,
            "KnowledgeGraph" => ExplorerTarget::KnowledgeGraph,
            "Experience" => ExplorerTarget::Experience,
            "Planning" => ExplorerTarget::Planning,
            "Execution" => ExplorerTarget::Execution,
            "Learning" => ExplorerTarget::Learning,
            "StrategicObjectives" => ExplorerTarget::StrategicObjectives,
            "BackgroundWorkers" => ExplorerTarget::BackgroundWorkers,
            "ConfidenceSystem" => ExplorerTarget::ConfidenceSystem,
            _ => {
                return Err(HandlerError::InvalidParams(format!(
                    "Unknown explorer target: {}",
                    target
                )));
            }
        };

        let result = explore_cognitive_system(explorer_target);
        Ok(crate::api::bridge::tools::ToolOutput::success(
            serde_json::json!({
                "target": result.target.label(),
                "summary": result.summary,
                "details": result.details,
                "timestamp": result.timestamp,
            }),
        ))
    }

    /// Inspect memory by layer.
    pub async fn inspect_memory_by_layer_tool(
        &self,
        layer: &str,
    ) -> Result<crate::api::bridge::tools::ToolOutput, HandlerError> {
        let result = inspect_memory_by_layer(layer);
        Ok(crate::api::bridge::tools::ToolOutput::success(
            serde_json::json!({
                "layer": result.layer,
                "record_count": result.record_count,
                "promotion_candidates": result.promotion_candidates,
                "archive_candidates": result.archive_candidates,
                "timestamp": result.timestamp,
            }),
        ))
    }

    /// Inspect strategic learning state.
    pub async fn inspect_strategic_learning_tool(
        &self,
    ) -> Result<crate::api::bridge::tools::ToolOutput, HandlerError> {
        let result = inspect_strategic_learning();
        Ok(crate::api::bridge::tools::ToolOutput::success(
            serde_json::json!({
                "active_objectives": result.active_objectives,
                "completed_objectives": result.completed_objectives,
                "pending_updates": result.pending_updates,
                "feedback_count": result.feedback_count,
                "timestamp": result.timestamp,
            }),
        ))
    }

    /// Inspect background workers.
    pub async fn inspect_background_workers_tool(
        &self,
    ) -> Result<crate::api::bridge::tools::ToolOutput, HandlerError> {
        let result = inspect_background_workers();
        Ok(crate::api::bridge::tools::ToolOutput::success(
            serde_json::json!({
                "running_workers": result.running_workers,
                "pending_tasks": result.pending_tasks,
                "completed_tasks": result.completed_tasks,
                "capacity_used_percent": result.capacity_used_percent,
                "timestamp": result.timestamp,
            }),
        ))
    }

    /// Manage memory layer (inspect/promote/archive).
    pub async fn manage_memory_layer_tool(
        &self,
        layer: &str,
        action: &str,
    ) -> Result<crate::api::bridge::tools::ToolOutput, HandlerError> {
        use crate::api::developer_interface::full::manage_memory_layer;

        let result = manage_memory_layer(layer, action);
        Ok(crate::api::bridge::tools::ToolOutput::success(
            serde_json::json!({
                "layer": result.layer,
                "record_count": result.record_count,
                "promotion_candidates": result.promotion_candidates,
                "archive_candidates": result.archive_candidates,
                "timestamp": result.timestamp,
            }),
        ))
    }

    /// Inspect learning evolution state.
    pub async fn inspect_learning_evolution_tool(
        &self,
    ) -> Result<crate::api::bridge::tools::ToolOutput, HandlerError> {
        let result = inspect_learning_evolution();
        Ok(crate::api::bridge::tools::ToolOutput::success(
            serde_json::json!({
                "active_objectives": result.active_objectives,
                "completed_objectives": result.completed_objectives,
                "pending_updates": result.pending_updates,
                "feedback_count": result.feedback_count,
                "timestamp": result.timestamp,
            }),
        ))
    }
}

impl ToolHandler for DeveloperInterfaceToolsHandler {
    fn category(&self) -> &str {
        "developer_interface"
    }

    fn tool_names(&self) -> Vec<String> {
        vec![
            "developer_execute_command".to_string(),
            "developer_explore_cognitive_system".to_string(),
            "developer_inspect_memory_by_layer".to_string(),
            "developer_inspect_strategic_learning".to_string(),
            "developer_inspect_background_workers".to_string(),
            "developer_manage_memory_layer".to_string(),
            "developer_inspect_learning_evolution".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        self.context.database.connection().is_ok()
    }

    fn get_tools(&self) -> Vec<rmcp::model::Tool> {
        use crate::api::mcp::handlers::json_to_schema;
        vec![
            rmcp::model::Tool::new(
                "developer_execute_command",
                "Execute a control plane command on the developer interface.",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Command name to execute" }
                    },
                    "required": ["name"]
                })),
            ).with_title("Developer Execute Command"),
            rmcp::model::Tool::new(
                "developer_explore_cognitive_system",
                "Explore a cognitive subsystem (Memory, KnowledgeGraph, Experience, etc.).",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Subsystem target to explore" }
                    },
                    "required": ["target"]
                })),
            ).with_title("Developer Explore Cognitive System"),
            rmcp::model::Tool::new(
                "developer_inspect_memory_by_layer",
                "Inspect memory records in a specific layer.",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "layer": { "type": "string", "description": "Memory layer to inspect" }
                    },
                    "required": ["layer"]
                })),
            ).with_title("Developer Inspect Memory By Layer"),
            rmcp::model::Tool::new(
                "developer_inspect_strategic_learning",
                "Inspect strategic learning state from knowledge and experience tables.",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {}
                })),
            ).with_title("Developer Inspect Strategic Learning"),
            rmcp::model::Tool::new(
                "developer_inspect_background_workers",
                "Inspect background worker status from scheduled tasks.",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {}
                })),
            ).with_title("Developer Inspect Background Workers"),
            rmcp::model::Tool::new(
                "developer_manage_memory_layer",
                "Manage a memory layer: inspect, promote, or archive records.",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "layer": { "type": "string", "description": "Memory layer to manage" },
                        "action": { "type": "string", "description": "Action: inspect, promote, or archive" }
                    },
                    "required": ["layer", "action"]
                })),
            ).with_title("Developer Manage Memory Layer"),
            rmcp::model::Tool::new(
                "developer_inspect_learning_evolution",
                "Inspect learning evolution state (delegates to strategic learning inspection).",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {}
                })),
            ).with_title("Developer Inspect Learning Evolution"),
        ]
    }

    async fn execute_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<crate::api::bridge::tools::ToolOutput, HandlerError> {
        match name {
            "developer_execute_command" => {
                let name_val = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HandlerError::InvalidParams("missing 'name'".to_string()))?;
                self.execute_control_command(name_val).await
            }
            "developer_explore_cognitive_system" => {
                let target = args
                    .get("target")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HandlerError::InvalidParams("missing 'target'".to_string()))?;
                self.explore_cognitive_system_tool(target).await
            }
            "developer_inspect_memory_by_layer" => {
                let layer = args
                    .get("layer")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HandlerError::InvalidParams("missing 'layer'".to_string()))?;
                self.inspect_memory_by_layer_tool(layer).await
            }
            "developer_inspect_strategic_learning" => self.inspect_strategic_learning_tool().await,
            "developer_inspect_background_workers" => self.inspect_background_workers_tool().await,
            "developer_manage_memory_layer" => {
                let layer = args
                    .get("layer")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HandlerError::InvalidParams("missing 'layer'".to_string()))?;
                let action = args
                    .get("action")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HandlerError::InvalidParams("missing 'action'".to_string()))?;
                self.manage_memory_layer_tool(layer, action).await
            }
            "developer_inspect_learning_evolution" => self.inspect_learning_evolution_tool().await,
            _ => Err(HandlerError::ToolNotFound(name.to_string())),
        }
    }
}
