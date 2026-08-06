// search_tools.rs - Cross-domain search tools

use crate::bridge::rmcp::generated::tool_traits::{
    SearchToolsHandlerTrait, ToolContext,
};
use crate::tools;
use crate::tools::ToolOutput;

/// Handler for search tools - implements SearchToolsHandlerTrait
pub struct SearchToolsHandler;

impl SearchToolsHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SearchToolsHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchToolsHandlerTrait for SearchToolsHandler {
    async fn execute_search_knowledge(
        &self,
        context: &ToolContext,
        input: tools::search::SearchKnowledgeInput,
    ) -> ToolOutput {
        match tools::search::execute_search_knowledge(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_search_experiences(
        &self,
        context: &ToolContext,
        input: tools::search::SearchExperiencesInput,
    ) -> ToolOutput {
        match tools::search::execute_search_experiences(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_search_memories(
        &self,
        context: &ToolContext,
        input: tools::search::SearchMemoriesInput,
    ) -> ToolOutput {
        match tools::search::execute_search_memories(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_advanced_search(
        &self,
        context: &ToolContext,
        input: tools::search::AdvancedSearchInput,
    ) -> ToolOutput {
        match tools::search::execute_advanced_search(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    fn list_tools(&self) -> Vec<rmcp::tool::Tool> {
        vec![
            tools::search::search_knowledge_tool(),
            tools::search::search_experiences_tool(),
            tools::search::search_memories_tool(),
            tools::search::advanced_search_tool(),
        ]
    }
}
