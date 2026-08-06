// hypothesis_tools.rs - Hypothesis and observation tools

use crate::bridge::rmcp::types::McpServerHandler;
use crate::tools;
use crate::tools::ToolOutput;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ContentBlock;
use rmcp::tool_router;
use rmcp::tool;
use crate::bridge::rmcp::helpers::{tool_output_to_content, enforcement_error_to_content};

#[tool_router]
impl McpServerHandler {
#[tool(
    name = "record_observation",
    description = "Record an observation. Observations are the starting point for learning."
)]
async fn record_observation(
    &self,
    Parameters(input): Parameters<tools::hypothesis::RecordObservationInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("record_observation").await {
        tracing::warn!("Workflow enforcement blocked record_observation: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::hypothesis::execute_record_observation(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("record_observation", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "create_hypothesis", description = "Create a testable hypothesis from observations.")]
async fn create_hypothesis(
    &self,
    Parameters(input): Parameters<tools::hypothesis::CreateHypothesisInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("create_hypothesis").await {
        tracing::warn!("Workflow enforcement blocked create_hypothesis: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::hypothesis::execute_create_hypothesis(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("create_hypothesis", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "add_evidence", description = "Add evidence to a hypothesis.")]
async fn add_evidence(
    &self,
    Parameters(input): Parameters<tools::hypothesis::AddEvidenceInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("add_evidence").await {
        tracing::warn!("Workflow enforcement blocked add_evidence: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::hypothesis::execute_add_evidence(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("add_evidence", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "get_hypothesis", description = "Get details of a specific hypothesis.")]
async fn get_hypothesis(
    &self,
    Parameters(input): Parameters<tools::hypothesis::GetHypothesisInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_hypothesis").await {
        tracing::warn!("Workflow enforcement blocked get_hypothesis: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::hypothesis::execute_get_hypothesis(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("get_hypothesis", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "list_hypotheses", description = "List all hypotheses with optional filters.")]
async fn list_hypotheses(
    &self,
    Parameters(input): Parameters<tools::hypothesis::ListHypothesesInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("list_hypotheses").await {
        tracing::warn!("Workflow enforcement blocked list_hypotheses: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::hypothesis::execute_list_hypotheses(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("list_hypotheses", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "list_observations", description = "List recorded observations.")]
async fn list_observations(
    &self,
    Parameters(input): Parameters<tools::hypothesis::ListObservationsInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("list_observations").await {
        tracing::warn!("Workflow enforcement blocked list_observations: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::hypothesis::execute_list_observations(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("list_observations", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "evaluate_hypothesis", description = "Evaluate a hypothesis based on its evidence.")]
async fn evaluate_hypothesis(
    &self,
    Parameters(input): Parameters<tools::hypothesis::EvaluateHypothesisInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("evaluate_hypothesis").await {
        tracing::warn!("Workflow enforcement blocked evaluate_hypothesis: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::hypothesis::execute_evaluate_hypothesis(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("evaluate_hypothesis", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "get_evidence", description = "Get a specific evidence record by its ID.")]
async fn get_evidence(
    &self,
    Parameters(input): Parameters<tools::hypothesis::GetEvidenceInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_evidence").await {
        tracing::warn!("Workflow enforcement blocked get_evidence: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::hypothesis::execute_get_evidence(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("get_evidence", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "list_evidence", description = "List all evidence records across hypotheses.")]
async fn list_evidence(
    &self,
    Parameters(input): Parameters<tools::hypothesis::ListEvidenceInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("list_evidence").await {
        tracing::warn!("Workflow enforcement blocked list_evidence: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::hypothesis::execute_list_evidence(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("list_evidence", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}
}
