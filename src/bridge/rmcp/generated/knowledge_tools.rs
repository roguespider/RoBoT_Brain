// knowledge_tools.rs - Knowledge management tools

use crate::bridge::rmcp::generated::tool_traits::{
    KnowledgeToolsHandlerTrait, ToolContext,
};
use crate::tools;
use crate::tools::ToolOutput;

/// Handler for knowledge tools - implements KnowledgeToolsHandlerTrait
pub struct KnowledgeToolsHandler;

impl KnowledgeToolsHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for KnowledgeToolsHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeToolsHandlerTrait for KnowledgeToolsHandler {
    async fn execute_create_knowledge(
        &self,
        context: &ToolContext,
        input: tools::knowledge::CreateKnowledgeInput,
    ) -> ToolOutput {
        match tools::knowledge::execute_create_knowledge(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_knowledge(
        &self,
        context: &ToolContext,
        input: tools::knowledge::GetKnowledgeInput,
    ) -> ToolOutput {
        match tools::knowledge::execute_get_knowledge(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_list_knowledge(
        &self,
        context: &ToolContext,
        input: tools::knowledge::ListKnowledgeInput,
    ) -> ToolOutput {
        match tools::knowledge::execute_list_knowledge(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_search_knowledge(
        &self,
        context: &ToolContext,
        input: tools::knowledge::SearchKnowledgeInput,
    ) -> ToolOutput {
        match tools::knowledge::execute_search_knowledge(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_update_knowledge(
        &self,
        context: &ToolContext,
        input: tools::knowledge::UpdateKnowledgeInput,
    ) -> ToolOutput {
        match tools::knowledge::execute_update_knowledge(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_delete_knowledge(
        &self,
        context: &ToolContext,
        input: tools::knowledge::DeleteKnowledgeInput,
    ) -> ToolOutput {
        match tools::knowledge::execute_delete_knowledge(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_knowledge_stats(
        &self,
        context: &ToolContext,
        input: tools::knowledge::GetKnowledgeStatsInput,
    ) -> ToolOutput {
        match tools::knowledge::execute_get_knowledge_stats(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_create_knowledge_base(
        &self,
        context: &ToolContext,
        input: tools::knowledge::CreateKnowledgeBaseInput,
    ) -> ToolOutput {
        match tools::knowledge::execute_create_knowledge_base(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_knowledge_base(
        &self,
        context: &ToolContext,
        input: tools::knowledge::GetKnowledgeBaseInput,
    ) -> ToolOutput {
        match tools::knowledge::execute_get_knowledge_base(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_list_knowledge_bases(
        &self,
        context: &ToolContext,
        input: tools::knowledge::ListKnowledgeBasesInput,
    ) -> ToolOutput {
        match tools::knowledge::execute_list_knowledge_bases(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_delete_knowledge_base(
        &self,
        context: &ToolContext,
        input: tools::knowledge::DeleteKnowledgeBaseInput,
    ) -> ToolOutput {
        match tools::knowledge::execute_delete_knowledge_base(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_add_source(
        &self,
        context: &ToolContext,
        input: tools::knowledge::AddSourceInput,
    ) -> ToolOutput {
        match tools::knowledge::execute_add_source(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_source(
        &self,
        context: &ToolContext,
        input: tools::knowledge::GetSourceInput,
    ) -> ToolOutput {
        match tools::knowledge::execute_get_source(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_list_sources(
        &self,
        context: &ToolContext,
        input: tools::knowledge::ListSourcesInput,
    ) -> ToolOutput {
        match tools::knowledge::execute_list_sources(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_delete_source(
        &self,
        context: &ToolContext,
        input: tools::knowledge::DeleteSourceInput,
    ) -> ToolOutput {
        match tools::knowledge::execute_delete_source(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    fn list_tools(&self) -> Vec<rmcp::tool::Tool> {
        vec![
            tools::knowledge::create_knowledge_tool(),
            tools::knowledge::get_knowledge_tool(),
            tools::knowledge::list_knowledge_tool(),
            tools::knowledge::search_knowledge_tool(),
            tools::knowledge::update_knowledge_tool(),
            tools::knowledge::delete_knowledge_tool(),
            tools::knowledge::get_knowledge_stats_tool(),
            tools::knowledge::create_knowledge_base_tool(),
            tools::knowledge::get_knowledge_base_tool(),
            tools::knowledge::list_knowledge_bases_tool(),
            tools::knowledge::delete_knowledge_base_tool(),
            tools::knowledge::add_source_tool(),
            tools::knowledge::get_source_tool(),
            tools::knowledge::list_sources_tool(),
            tools::knowledge::delete_source_tool(),
        ]
    }
}
