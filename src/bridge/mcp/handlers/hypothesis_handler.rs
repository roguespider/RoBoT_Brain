// src/bridge/tools/handlers/hypothesis_handler.rs
// Hypothesis tools handler - handles hypothesis engine operations

use std::sync::Arc;
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::hypothesis;
use crate::bridge::mcp::handlers::{HandlerError, HandlerInitError, HandlerInitResult, ToolHandler};

/// Handler for hypothesis-related tools
#[derive(Clone)]
pub struct HypothesisToolsHandler {
    context: Arc<McpContext>,
}

impl HypothesisToolsHandler {
    /// Create a new hypothesis tools handler
    pub fn new(
        context: Arc<McpContext>,
    ) -> HandlerInitResult<Self> {
        // Validate that required dependencies exist
        if context.database.connection().is_err() {
            return Err(HandlerInitError::new(
                "hypothesis",
                "Database connection not available",
            ));
        }

        Ok(Self { context })
    }

    /// Record an observation
    pub async fn execute_record_observation(
        &self,
        input: hypothesis::RecordObservationInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        hypothesis::execute_record_observation(input, &self.context.database).await
    }

    /// Create a hypothesis
    pub async fn execute_create_hypothesis(
        &self,
        input: hypothesis::CreateHypothesisInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        hypothesis::execute_create_hypothesis(input, &self.context.database).await
    }

    /// Add evidence to a hypothesis
    pub async fn execute_add_evidence(
        &self,
        input: hypothesis::AddEvidenceInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        hypothesis::execute_add_evidence(input, &self.context.database).await
    }

    /// Get hypothesis details
    pub async fn execute_get_hypothesis(
        &self,
        input: hypothesis::GetHypothesisInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        hypothesis::execute_get_hypothesis(input, &self.context.database).await
    }

    /// List hypotheses
    pub async fn execute_list_hypotheses(
        &self,
        input: hypothesis::ListHypothesesInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        hypothesis::execute_list_hypotheses(input, &self.context.database).await
    }

    /// List observations
    pub async fn execute_list_observations(
        &self,
        input: hypothesis::ListObservationsInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        hypothesis::execute_list_observations(input, &self.context.database).await
    }

    /// Evaluate a hypothesis
    pub async fn execute_evaluate_hypothesis(
        &self,
        input: hypothesis::EvaluateHypothesisInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        hypothesis::execute_evaluate_hypothesis(input, &self.context.database).await
    }

    /// Get learned knowledge
    pub async fn execute_get_knowledge(
        &self,
        input: hypothesis::GetKnowledgeInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        hypothesis::execute_get_knowledge(input, &self.context.database).await
    }

    /// Extract knowledge from hypothesis
    pub async fn execute_extract_knowledge(
        &self,
        input: hypothesis::ExtractKnowledgeInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        hypothesis::execute_extract_knowledge(input, &self.context.database).await
    }

    /// Get evidence by ID
    pub async fn execute_get_evidence(
        &self,
        input: hypothesis::GetEvidenceInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        hypothesis::execute_get_evidence(input, &self.context.database).await
    }

    /// List all evidence
    pub async fn execute_list_evidence(
        &self,
        input: hypothesis::ListEvidenceInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        hypothesis::execute_list_evidence(input, &self.context.database).await
    }
}

impl ToolHandler for HypothesisToolsHandler {
    fn category(&self) -> &str {
        "hypothesis"
    }

    fn tool_names(&self) -> Vec<String> {
        vec![
            "record_observation".to_string(),
            "create_hypothesis".to_string(),
            "add_evidence".to_string(),
            "get_hypothesis".to_string(),
            "list_hypotheses".to_string(),
            "list_observations".to_string(),
            "evaluate_hypothesis".to_string(),
            "get_knowledge".to_string(),
            "extract_knowledge".to_string(),
            "get_evidence".to_string(),
            "list_evidence".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        self.context.database.connection().is_ok()
    }

    fn get_tools(&self) -> Vec<rmcp::model::Tool> {
        use crate::bridge::mcp::handlers::json_to_schema;
        vec![
            rmcp::model::Tool::new(
                "record_observation",
                "Record an observation as the starting point for learning",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": { "type": "string", "description": "What was observed" },
                        "observation_type": { "type": "string", "description": "Type: success, failure, pattern, anomaly" },
                        "context": { "type": "string", "description": "Context or circumstances" }
                    },
                    "required": ["content", "observation_type"]
                })),
            ).with_title("Record Observation"),
            rmcp::model::Tool::new(
                "create_hypothesis",
                "Create a testable hypothesis from observations",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "statement": { "type": "string", "description": "The hypothesis statement" },
                        "domain": { "type": "string", "description": "Domain/category" },
                        "source_observations": { "type": "array", "items": { "type": "string" }, "description": "Observation IDs" }
                    },
                    "required": ["statement", "domain"]
                })),
            ).with_title("Create Hypothesis"),
            rmcp::model::Tool::new(
                "add_evidence",
                "Add evidence to a hypothesis",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "hypothesis_id": { "type": "string", "description": "ID of the hypothesis" },
                        "content": { "type": "string", "description": "Description of the evidence" },
                        "direction": { "type": "string", "description": "support or contradict" },
                        "evidence_type": { "type": "string", "description": "Type: success, failure, correlation, anomaly" },
                        "strength": { "type": "number", "description": "Strength 0.0-1.0" }
                    },
                    "required": ["hypothesis_id", "content", "direction"]
                })),
            ).with_title("Add Evidence"),
            rmcp::model::Tool::new(
                "get_hypothesis",
                "Get details of a specific hypothesis",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "hypothesis_id": { "type": "string", "description": "ID of the hypothesis" }
                    },
                    "required": ["hypothesis_id"]
                })),
            ).with_title("Get Hypothesis"),
            rmcp::model::Tool::new(
                "list_hypotheses",
                "List all hypotheses with optional filters",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "number", "description": "Maximum results" },
                        "status": { "type": "string", "description": "Filter by status" },
                        "domain": { "type": "string", "description": "Filter by domain" }
                    }
                })),
            ).with_title("List Hypotheses"),
            rmcp::model::Tool::new(
                "list_observations",
                "List recorded observations",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "number", "description": "Maximum results" },
                        "observation_type": { "type": "string", "description": "Filter by type" }
                    }
                })),
            ).with_title("List Observations"),
            rmcp::model::Tool::new(
                "evaluate_hypothesis",
                "Evaluate a hypothesis based on accumulated evidence",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "hypothesis_id": { "type": "string", "description": "ID of the hypothesis to evaluate" }
                    },
                    "required": ["hypothesis_id"]
                })),
            ).with_title("Evaluate Hypothesis"),
            rmcp::model::Tool::new(
                "get_knowledge",
                "Get learned knowledge that can inform future decisions",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "domain": { "type": "string", "description": "Filter by domain" },
                        "limit": { "type": "number", "description": "Maximum results" }
                    }
                })),
            ).with_title("Get Knowledge"),
            rmcp::model::Tool::new(
                "extract_knowledge",
                "Extract supported hypothesis as reusable knowledge",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "hypothesis_id": { "type": "string", "description": "ID of the supported hypothesis" },
                        "knowledge_content": { "type": "string", "description": "The knowledge content to extract" }
                    },
                    "required": ["hypothesis_id", "knowledge_content"]
                })),
            ).with_title("Extract Knowledge"),
            rmcp::model::Tool::new(
                "get_evidence",
                "Get a specific evidence record by its ID",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "evidence_id": { "type": "string", "description": "ID of the evidence" }
                    },
                    "required": ["evidence_id"]
                })),
            ).with_title("Get Evidence"),
            rmcp::model::Tool::new(
                "list_evidence",
                "List all evidence records across hypotheses",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "number", "description": "Maximum results" },
                        "direction": { "type": "string", "description": "Filter by direction" },
                        "evidence_type": { "type": "string", "description": "Filter by type" }
                    }
                })),
            ).with_title("List Evidence"),
        ]
    }

    fn execute_tool(&self, name: &str, args: serde_json::Value) -> impl std::future::Future<Output = Result<crate::bridge::tools::ToolOutput, HandlerError>> + Send {
        async move {
            match name {
                "record_observation" => {
                    let input: hypothesis::RecordObservationInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_record_observation(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "create_hypothesis" => {
                    let input: hypothesis::CreateHypothesisInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_create_hypothesis(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "add_evidence" => {
                    let input: hypothesis::AddEvidenceInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_add_evidence(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_hypothesis" => {
                    let input: hypothesis::GetHypothesisInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_get_hypothesis(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "list_hypotheses" => {
                    let input: hypothesis::ListHypothesesInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_list_hypotheses(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "list_observations" => {
                    let input: hypothesis::ListObservationsInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_list_observations(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "evaluate_hypothesis" => {
                    let input: hypothesis::EvaluateHypothesisInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_evaluate_hypothesis(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_knowledge" => {
                    let input: hypothesis::GetKnowledgeInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_get_knowledge(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "extract_knowledge" => {
                    let input: hypothesis::ExtractKnowledgeInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_extract_knowledge(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_evidence" => {
                    let input: hypothesis::GetEvidenceInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_get_evidence(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "list_evidence" => {
                    let input: hypothesis::ListEvidenceInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_list_evidence(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                _ => Err(HandlerError::ToolNotFound(name.to_string()))
            }
        }
    }
}
