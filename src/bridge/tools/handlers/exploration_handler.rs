// src/bridge/tools/handlers/exploration_handler.rs
// Exploration tools handler - handles exploration tools

use crate::bridge::tools::exploration;
use crate::bridge::tools::handlers::{HandlerInitResult, ToolHandler};
use crate::workflows::enforcement::WorkflowEnforcer;
use std::sync::Arc;
use crate::bridge::mcp::McpContext;

/// Handler for exploration-related tools
#[derive(Clone)]
pub struct ExplorationToolsHandler {
    context: Arc<McpContext>,
    enforcer: Arc<WorkflowEnforcer>,
}

impl ExplorationToolsHandler {
    /// Create a new exploration tools handler
    pub fn new(
        context: Arc<McpContext>,
        enforcer: Arc<WorkflowEnforcer>,
    ) -> HandlerInitResult<Self> {
        Ok(Self { context, enforcer })
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
            "evaluate_hypothesis".to_string(),
            "promote_finding".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        true
    }
}
