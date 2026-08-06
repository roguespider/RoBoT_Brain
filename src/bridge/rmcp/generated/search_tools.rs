    // search_tools.rs - Global search and recommendation tools

use crate::bridge::rmcp::types::McpServerHandler;
use crate::bridge::tools;
use crate::bridge::tools::ToolOutput;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ContentBlock;
use rmcp::tool_router;
use rmcp::tool;
use crate::bridge::rmcp::helpers::{tool_output_to_content, enforcement_error_to_content};

#[tool_router]
impl McpServerHandler {
    #[tool(name = "global_search", description = "Search across all memories and experiences")]
    async fn global_search(
        &self,
        Parameters(input): Parameters<tools::search::GlobalSearchInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("global_search").await {
            tracing::warn!("Workflow enforcement blocked global_search: {}", e.message);
            return enforcement_error_to_content(e);
        }
        let query = Some(input.query.clone());
        match tools::search::execute_global_search(input, &self.context.database).await {
            Ok(result) => {
                self.record_tool_execution("global_search", query).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "get_recommendations", description = "Get recommendations based on learned patterns")]
    async fn get_recommendations(
        &self,
        Parameters(input): Parameters<tools::search::GetRecommendationsInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("get_recommendations").await {
            tracing::warn!("Workflow enforcement blocked get_recommendations: {}", e.message);
            return enforcement_error_to_content(e);
        }
        match tools::search::execute_get_recommendations(input, &self.context.database).await {
            Ok(result) => {
                self.record_tool_execution("get_recommendations", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "get_reputation", description = "Get reputation score for a target")]
    async fn get_reputation(
        &self,
        Parameters(input): Parameters<tools::search::GetReputationInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("get_reputation").await {
            tracing::warn!("Workflow enforcement blocked get_reputation: {}", e.message);
            return enforcement_error_to_content(e);
        }
        match tools::search::execute_get_reputation(input, &self.context.database).await {
            Ok(result) => {
                self.record_tool_execution("get_reputation", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }
}
