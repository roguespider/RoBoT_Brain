// exploration_tools.rs - Exploration and discovery tools

use crate::bridge::rmcp::generated::tool_traits::{
    ExplorationToolsHandlerTrait, ToolContext,
};
use crate::tools;
use crate::tools::ToolOutput;

/// Handler for exploration tools - implements ExplorationToolsHandlerTrait
pub struct ExplorationToolsHandler;

impl ExplorationToolsHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExplorationToolsHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ExplorationToolsHandlerTrait for ExplorationToolsHandler {
    async fn execute_start_exploration(
        &self,
        context: &ToolContext,
        input: tools::exploration::StartExplorationInput,
    ) -> ToolOutput {
        match tools::exploration::execute_start_exploration(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_exploration(
        &self,
        context: &ToolContext,
        input: tools::exploration::GetExplorationInput,
    ) -> ToolOutput {
        match tools::exploration::execute_get_exploration(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_list_explorations(
        &self,
        context: &ToolContext,
        input: tools::exploration::ListExplorationsInput,
    ) -> ToolOutput {
        match tools::exploration::execute_list_explorations(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_pause_exploration(
        &self,
        context: &ToolContext,
        input: tools::exploration::PauseExplorationInput,
    ) -> ToolOutput {
        match tools::exploration::execute_pause_exploration(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_resume_exploration(
        &self,
        context: &ToolContext,
        input: tools::exploration::ResumeExplorationInput,
    ) -> ToolOutput {
        match tools::exploration::execute_resume_exploration(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_complete_exploration(
        &self,
        context: &ToolContext,
        input: tools::exploration::CompleteExplorationInput,
    ) -> ToolOutput {
        match tools::exploration::execute_complete_exploration(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_cancel_exploration(
        &self,
        context: &ToolContext,
        input: tools::exploration::CancelExplorationInput,
    ) -> ToolOutput {
        match tools::exploration::execute_cancel_exploration(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_exploration_status(
        &self,
        context: &ToolContext,
        input: tools::exploration::GetExplorationStatusInput,
    ) -> ToolOutput {
        match tools::exploration::execute_get_exploration_status(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_record_exploration_finding(
        &self,
        context: &ToolContext,
        input: tools::exploration::RecordExplorationFindingInput,
    ) -> ToolOutput {
        match tools::exploration::execute_record_exploration_finding(
            input,
            &context.context.database,
        )
        .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_exploration_findings(
        &self,
        context: &ToolContext,
        input: tools::exploration::GetExplorationFindingsInput,
    ) -> ToolOutput {
        match tools::exploration::execute_get_exploration_findings(
            input,
            &context.context.database,
        )
        .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_list_exploration_findings(
        &self,
        context: &ToolContext,
        input: tools::exploration::ListExplorationFindingsInput,
    ) -> ToolOutput {
        match tools::exploration::execute_list_exploration_findings(
            input,
            &context.context.database,
        )
        .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    fn list_tools(&self) -> Vec<rmcp::tool::Tool> {
        vec![
            tools::exploration::start_exploration_tool(),
            tools::exploration::get_exploration_tool(),
            tools::exploration::list_explorations_tool(),
            tools::exploration::pause_exploration_tool(),
            tools::exploration::resume_exploration_tool(),
            tools::exploration::complete_exploration_tool(),
            tools::exploration::cancel_exploration_tool(),
            tools::exploration::get_exploration_status_tool(),
            tools::exploration::record_exploration_finding_tool(),
            tools::exploration::get_exploration_findings_tool(),
            tools::exploration::list_exploration_findings_tool(),
        ]
    }
}
