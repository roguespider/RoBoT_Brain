// src/bridge/tools/handlers/exploration_handler.rs
// Exploration tools handler - handles exploration tools

use crate::bridge::tools::exploration;
use crate::bridge::mcp::handlers::{HandlerError, HandlerInitResult, ToolHandler};

/// Handler for exploration-related tools
#[derive(Clone)]
pub struct ExplorationToolsHandler;

impl ExplorationToolsHandler {
    /// Create a new exploration tools handler
    pub fn new() -> HandlerInitResult<Self> {
        Ok(Self)
    }

    /// Start exploration
    pub fn execute_start_exploration(
        &self,
        input: exploration::StartExplorationInput,
    ) -> crate::bridge::tools::ToolOutput {
        exploration::execute_start_exploration(input)
    }

    /// Get exploration status
    pub fn execute_get_exploration_status(
        &self,
        input: exploration::GetExplorationStatusInput,
    ) -> crate::bridge::tools::ToolOutput {
        exploration::execute_get_exploration_status(input)
    }

    /// Pause exploration
    pub fn execute_pause_exploration(
        &self,
        input: exploration::GetExplorationStatusInput,
    ) -> crate::bridge::tools::ToolOutput {
        exploration::execute_pause_exploration(input)
    }

    /// Resume exploration
    pub fn execute_resume_exploration(
        &self,
        input: exploration::GetExplorationStatusInput,
    ) -> crate::bridge::tools::ToolOutput {
        exploration::execute_resume_exploration(input)
    }

    /// Complete exploration
    pub fn execute_complete_exploration(
        &self,
        input: exploration::CompleteExplorationInput,
    ) -> crate::bridge::tools::ToolOutput {
        exploration::execute_complete_exploration(input)
    }

    /// Abandon exploration
    pub fn execute_abandon_exploration(
        &self,
        input: exploration::GetExplorationStatusInput,
    ) -> crate::bridge::tools::ToolOutput {
        exploration::execute_abandon_exploration(input)
    }

    /// Record attempt
    pub fn execute_record_attempt(
        &self,
        input: exploration::RecordAttemptInput,
    ) -> crate::bridge::tools::ToolOutput {
        exploration::execute_record_attempt(input)
    }

    /// Add hypothesis
    pub fn execute_add_hypothesis(
        &self,
        input: exploration::AddHypothesisInput,
    ) -> crate::bridge::tools::ToolOutput {
        exploration::execute_add_hypothesis(input)
    }

    /// Evaluate hypothesis
    pub fn execute_evaluate_hypothesis(
        &self,
        input: exploration::EvaluateHypothesisInput,
    ) -> crate::bridge::tools::ToolOutput {
        exploration::execute_evaluate_hypothesis(input)
    }

    /// Promote finding
    pub fn execute_promote_finding(
        &self,
        input: exploration::PromoteFindingInput,
    ) -> crate::bridge::tools::ToolOutput {
        exploration::execute_promote_finding(input)
    }
}

impl ToolHandler for ExplorationToolsHandler {
    fn category(&self) -> &str {
        "exploration"
    }

    fn tool_names(&self) -> Vec<String> {
        vec![
            "start_exploration".to_string(),
            "get_exploration_status".to_string(),
            "pause_exploration".to_string(),
            "resume_exploration".to_string(),
            "complete_exploration".to_string(),
            "abandon_exploration".to_string(),
            "record_attempt".to_string(),
            "add_hypothesis".to_string(),
            "evaluate_exploration_hypothesis".to_string(),
            "promote_finding".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        true
    }

    fn get_tools(&self) -> Vec<rmcp::model::Tool> {
        use crate::bridge::mcp::handlers::json_to_schema;
        vec![
            rmcp::model::Tool::new(
                "start_exploration",
                "Start a new exploration to investigate a problem or hypothesis",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "topic": { "type": "string", "description": "Topic to explore" },
                        "approach": { "type": "string", "description": "Exploration approach" }
                    },
                    "required": ["topic"]
                })),
            ).with_title("Start Exploration"),
            rmcp::model::Tool::new(
                "get_exploration_status",
                "Get the current status of an exploration",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "exploration_id": { "type": "string", "description": "Exploration ID" }
                    },
                    "required": ["exploration_id"]
                })),
            ).with_title("Get Exploration Status"),
            rmcp::model::Tool::new(
                "pause_exploration",
                "Pause an ongoing exploration",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "exploration_id": { "type": "string", "description": "Exploration ID" }
                    },
                    "required": ["exploration_id"]
                })),
            ).with_title("Pause Exploration"),
            rmcp::model::Tool::new(
                "resume_exploration",
                "Resume a paused exploration",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "exploration_id": { "type": "string", "description": "Exploration ID" }
                    },
                    "required": ["exploration_id"]
                })),
            ).with_title("Resume Exploration"),
            rmcp::model::Tool::new(
                "complete_exploration",
                "Complete an exploration with findings",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "exploration_id": { "type": "string", "description": "Exploration ID" },
                        "findings": { "type": "string", "description": "Key findings" }
                    },
                    "required": ["exploration_id", "findings"]
                })),
            ).with_title("Complete Exploration"),
            rmcp::model::Tool::new(
                "abandon_exploration",
                "Abandon an exploration without findings",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "exploration_id": { "type": "string", "description": "Exploration ID" }
                    },
                    "required": ["exploration_id"]
                })),
            ).with_title("Abandon Exploration"),
            rmcp::model::Tool::new(
                "record_attempt",
                "Record an attempt during exploration",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "exploration_id": { "type": "string", "description": "Exploration ID" },
                        "description": { "type": "string", "description": "What was attempted" },
                        "result": { "type": "string", "description": "Result of the attempt" }
                    },
                    "required": ["exploration_id", "description", "result"]
                })),
            ).with_title("Record Attempt"),
            rmcp::model::Tool::new(
                "add_hypothesis",
                "Add a hypothesis to an exploration",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "exploration_id": { "type": "string", "description": "Exploration ID" },
                        "hypothesis": { "type": "string", "description": "Hypothesis statement" }
                    },
                    "required": ["exploration_id", "hypothesis"]
                })),
            ).with_title("Add Hypothesis"),
            rmcp::model::Tool::new(
                "evaluate_exploration_hypothesis",
                "Evaluate a hypothesis during exploration",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "exploration_id": { "type": "string", "description": "Exploration ID" },
                        "hypothesis_id": { "type": "string", "description": "Hypothesis ID" },
                        "result": { "type": "string", "description": "supported, partially_supported, rejected, or unknown" }
                    },
                    "required": ["exploration_id", "hypothesis_id", "result"]
                })),
            ).with_title("Evaluate Exploration Hypothesis"),
            rmcp::model::Tool::new(
                "promote_finding",
                "Promote a finding to reusable knowledge",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "exploration_id": { "type": "string", "description": "Exploration ID" },
                        "finding_id": { "type": "string", "description": "Finding ID to promote" }
                    },
                    "required": ["exploration_id", "finding_id"]
                })),
            ).with_title("Promote Finding"),
        ]
    }

    fn execute_tool(&self, name: &str, args: serde_json::Value) -> impl std::future::Future<Output = Result<crate::bridge::tools::ToolOutput, HandlerError>> + Send {
        async move {
            match name {
                "start_exploration" => {
                    let input: exploration::StartExplorationInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_start_exploration(input))
                }
                "get_exploration_status" => {
                    let input: exploration::GetExplorationStatusInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_get_exploration_status(input))
                }
                "pause_exploration" => {
                    let input: exploration::GetExplorationStatusInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_pause_exploration(input))
                }
                "resume_exploration" => {
                    let input: exploration::GetExplorationStatusInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_resume_exploration(input))
                }
                "complete_exploration" => {
                    let input: exploration::CompleteExplorationInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_complete_exploration(input))
                }
                "abandon_exploration" => {
                    let input: exploration::GetExplorationStatusInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_abandon_exploration(input))
                }
                "record_attempt" => {
                    let input: exploration::RecordAttemptInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_record_attempt(input))
                }
                "add_hypothesis" => {
                    let input: exploration::AddHypothesisInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_add_hypothesis(input))
                }
                "evaluate_exploration_hypothesis" => {
                    let input: exploration::EvaluateHypothesisInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_evaluate_hypothesis(input))
                }
                "promote_finding" => {
                    let input: exploration::PromoteFindingInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_promote_finding(input))
                }
                _ => Err(HandlerError::ToolNotFound(name.to_string()))
            }
        }
    }
}
