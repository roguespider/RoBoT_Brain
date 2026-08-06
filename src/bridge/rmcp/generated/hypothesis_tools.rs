// hypothesis_tools.rs - Hypothesis testing tools

use crate::bridge::rmcp::generated::tool_traits::{
    HypothesisToolsHandlerTrait, ToolContext,
};
use crate::tools;
use crate::tools::ToolOutput;

/// Handler for hypothesis tools - implements HypothesisToolsHandlerTrait
pub struct HypothesisToolsHandler;

impl HypothesisToolsHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HypothesisToolsHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl HypothesisToolsHandlerTrait for HypothesisToolsHandler {
    async fn execute_create_hypothesis(
        &self,
        context: &ToolContext,
        input: tools::hypothesis::CreateHypothesisInput,
    ) -> ToolOutput {
        match tools::hypothesis::execute_create_hypothesis(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_hypothesis(
        &self,
        context: &ToolContext,
        input: tools::hypothesis::GetHypothesisInput,
    ) -> ToolOutput {
        match tools::hypothesis::execute_get_hypothesis(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_list_hypotheses(
        &self,
        context: &ToolContext,
        input: tools::hypothesis::ListHypothesesInput,
    ) -> ToolOutput {
        match tools::hypothesis::execute_list_hypotheses(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_evaluate_hypothesis(
        &self,
        context: &ToolContext,
        input: tools::hypothesis::EvaluateHypothesisInput,
    ) -> ToolOutput {
        match tools::hypothesis::execute_evaluate_hypothesis(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_record_observation(
        &self,
        context: &ToolContext,
        input: tools::hypothesis::RecordObservationInput,
    ) -> ToolOutput {
        match tools::hypothesis::execute_record_observation(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_list_observations(
        &self,
        context: &ToolContext,
        input: tools::hypothesis::ListObservationsInput,
    ) -> ToolOutput {
        match tools::hypothesis::execute_list_observations(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_add_evidence(
        &self,
        context: &ToolContext,
        input: tools::hypothesis::AddEvidenceInput,
    ) -> ToolOutput {
        match tools::hypothesis::execute_add_evidence(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_evidence(
        &self,
        context: &ToolContext,
        input: tools::hypothesis::GetEvidenceInput,
    ) -> ToolOutput {
        match tools::hypothesis::execute_get_evidence(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_list_evidence(
        &self,
        context: &ToolContext,
        input: tools::hypothesis::ListEvidenceInput,
    ) -> ToolOutput {
        match tools::hypothesis::execute_list_evidence(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_extract_knowledge(
        &self,
        context: &ToolContext,
        input: tools::hypothesis::ExtractKnowledgeInput,
    ) -> ToolOutput {
        match tools::hypothesis::execute_extract_knowledge(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_knowledge(
        &self,
        context: &ToolContext,
        input: tools::hypothesis::GetKnowledgeInput,
    ) -> ToolOutput {
        match tools::hypothesis::execute_get_knowledge(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    fn list_tools(&self) -> Vec<rmcp::tool::Tool> {
        vec![
            tools::hypothesis::create_hypothesis_tool(),
            tools::hypothesis::get_hypothesis_tool(),
            tools::hypothesis::list_hypotheses_tool(),
            tools::hypothesis::evaluate_hypothesis_tool(),
            tools::hypothesis::record_observation_tool(),
            tools::hypothesis::list_observations_tool(),
            tools::hypothesis::add_evidence_tool(),
            tools::hypothesis::get_evidence_tool(),
            tools::hypothesis::list_evidence_tool(),
            tools::hypothesis::extract_knowledge_tool(),
            tools::hypothesis::get_knowledge_tool(),
        ]
    }
}
