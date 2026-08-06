// exploration_tools.rs - Exploration session tools

use crate::bridge::rmcp::types::McpServerHandler;
use crate::tools;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ContentBlock;
use rmcp::tool_router;
use rmcp::tool;
use crate::bridge::rmcp::helpers::{tool_output_to_content, enforcement_error_to_content};

#[tool_router]
impl McpServerHandler {
#[tool(
    name = "start_exploration",
    description = "Start a new exploration session. Explorations allow RoBoT to actively investigate topics and test hypotheses."
)]
async fn start_exploration(
    &self,
    Parameters(input): Parameters<tools::exploration::StartExplorationInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("start_exploration").await {
        tracing::warn!("Workflow enforcement blocked start_exploration: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::exploration::execute_start_exploration(input);
    if result.success {
        self.record_tool_execution("start_exploration", None).await;
    }
    tool_output_to_content(result)
}

#[tool(
    name = "get_exploration_status",
    description = "Get the current status of an exploration including hypotheses, attempts, and findings."
)]
async fn get_exploration_status(
    &self,
    Parameters(input): Parameters<tools::exploration::GetExplorationStatusInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_exploration_status").await {
        tracing::warn!("Workflow enforcement blocked get_exploration_status: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::exploration::execute_get_exploration_status(input);
    if result.success {
        self.record_tool_execution("get_exploration_status", None).await;
    }
    tool_output_to_content(result)
}

#[tool(
    name = "complete_exploration",
    description = "Mark an exploration as completed with findings."
)]
async fn complete_exploration(
    &self,
    Parameters(input): Parameters<tools::exploration::CompleteExplorationInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("complete_exploration").await {
        tracing::warn!("Workflow enforcement blocked complete_exploration: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::exploration::execute_complete_exploration(input);
    if result.success {
        self.record_tool_execution("complete_exploration", None).await;
    }
    tool_output_to_content(result)
}

#[tool(
    name = "abandon_exploration",
    description = "Abandon an exploration without completing it."
)]
async fn abandon_exploration(
    &self,
    Parameters(input): Parameters<tools::exploration::GetExplorationStatusInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("abandon_exploration").await {
        tracing::warn!("Workflow enforcement blocked abandon_exploration: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::exploration::execute_abandon_exploration(input);
    if result.success {
        self.record_tool_execution("abandon_exploration", None).await;
    }
    tool_output_to_content(result)
}

#[tool(
    name = "record_attempt",
    description = "Record an attempt made during exploration."
)]
async fn record_attempt(
    &self,
    Parameters(input): Parameters<tools::exploration::RecordAttemptInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("record_attempt").await {
        tracing::warn!("Workflow enforcement blocked record_attempt: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::exploration::execute_record_attempt(input);
    if result.success {
        self.record_tool_execution("record_attempt", None).await;
    }
    tool_output_to_content(result)
}

#[tool(
    name = "add_exploration_hypothesis",
    description = "Add a testable hypothesis to an exploration."
)]
async fn add_exploration_hypothesis(
    &self,
    Parameters(input): Parameters<tools::exploration::AddHypothesisInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("add_exploration_hypothesis").await {
        tracing::warn!("Workflow enforcement blocked add_exploration_hypothesis: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::exploration::execute_add_hypothesis(input);
    if result.success {
        self.record_tool_execution("add_exploration_hypothesis", None).await;
    }
    tool_output_to_content(result)
}

#[tool(
    name = "evaluate_exploration_hypothesis",
    description = "Set the result for a hypothesis based on evidence."
)]
async fn evaluate_exploration_hypothesis(
    &self,
    Parameters(input): Parameters<tools::exploration::EvaluateHypothesisInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("evaluate_exploration_hypothesis").await {
        tracing::warn!("Workflow enforcement blocked evaluate_exploration_hypothesis: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::exploration::execute_evaluate_hypothesis(input);
    if result.success {
        self.record_tool_execution("evaluate_exploration_hypothesis", None).await;
    }
    tool_output_to_content(result)
}

#[tool(
    name = "promote_finding",
    description = "Promote a finding from an exploration to reusable knowledge."
)]
async fn promote_finding(
    &self,
    Parameters(input): Parameters<tools::exploration::PromoteFindingInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("promote_finding").await {
        tracing::warn!("Workflow enforcement blocked promote_finding: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::exploration::execute_promote_finding(input);
    if result.success {
        self.record_tool_execution("promote_finding", None).await;
    }
    tool_output_to_content(result)
}

#[tool(
    name = "pause_exploration",
    description = "Pause an active exploration."
)]
async fn pause_exploration(
    &self,
    Parameters(input): Parameters<tools::exploration::GetExplorationStatusInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("pause_exploration").await {
        tracing::warn!("Workflow enforcement blocked pause_exploration: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::exploration::execute_pause_exploration(input);
    if result.success {
        self.record_tool_execution("pause_exploration", None).await;
    }
    tool_output_to_content(result)
}

#[tool(
    name = "resume_exploration",
    description = "Resume a paused exploration."
)]
async fn resume_exploration(
    &self,
    Parameters(input): Parameters<tools::exploration::GetExplorationStatusInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("resume_exploration").await {
        tracing::warn!("Workflow enforcement blocked resume_exploration: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::exploration::execute_resume_exploration(input);
    if result.success {
        self.record_tool_execution("resume_exploration", None).await;
    }
    tool_output_to_content(result)
}
}
