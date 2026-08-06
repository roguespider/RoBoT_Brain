// experience_tools.rs - Experience recording and management tools

use crate::bridge::rmcp::generated::tool_traits::{
    ExperienceToolsHandlerTrait, ToolContext,
};
use crate::tools;
use crate::tools::ToolOutput;

/// Handler for experience tools - implements ExperienceToolsHandlerTrait
pub struct ExperienceToolsHandler;

impl ExperienceToolsHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExperienceToolsHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ExperienceToolsHandlerTrait for ExperienceToolsHandler {
    async fn execute_record_experience(
        &self,
        context: &ToolContext,
        input: tools::experience::RecordExperienceInput,
    ) -> ToolOutput {
        match tools::experience::execute_record_experience(
            input,
            &context.context.coordinator,
            &context.context.database,
        )
        .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_experience_stats(
        &self,
        context: &ToolContext,
        input: tools::experience::GetExperienceStatsInput,
    ) -> ToolOutput {
        match tools::experience::execute_get_experience_stats(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_list_experiences(
        &self,
        context: &ToolContext,
        input: tools::experience::ListExperiencesInput,
    ) -> ToolOutput {
        match tools::experience::execute_list_experiences(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_experience(
        &self,
        context: &ToolContext,
        input: tools::experience::GetExperienceInput,
    ) -> ToolOutput {
        match tools::experience::execute_get_experience(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_search_experiences(
        &self,
        context: &ToolContext,
        input: tools::experience::SearchExperiencesInput,
    ) -> ToolOutput {
        match tools::experience::execute_search_experiences(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_delete_experience(
        &self,
        context: &ToolContext,
        input: tools::experience::DeleteExperienceInput,
    ) -> ToolOutput {
        match tools::experience::execute_delete_experience(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    fn list_tools(&self) -> Vec<rmcp::tool::Tool> {
        vec![
            tools::experience::record_experience_tool(),
            tools::experience::get_experience_stats_tool(),
            tools::experience::list_experiences_tool(),
            tools::experience::get_experience_tool(),
            tools::experience::search_experiences_tool(),
            tools::experience::delete_experience_tool(),
        ]
    }
}
