    // knowledge_tools.rs - Knowledge base tools

use crate::bridge::rmcp::types::McpServerHandler;
use crate::tools;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ContentBlock;
use rmcp::tool_router;
use rmcp::tool;
use crate::bridge::rmcp::helpers::{tool_output_to_content, enforcement_error_to_content};

#[tool_router]
impl McpServerHandler {
    #[tool(name = "get_knowledge", description = "Get learned knowledge extracted from validated hypotheses.")]
    async fn get_knowledge(
        &self,
        Parameters(input): Parameters<tools::hypothesis::GetKnowledgeInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("get_knowledge").await {
            tracing::warn!("Workflow enforcement blocked get_knowledge: {}", e.message);
            return enforcement_error_to_content(e);
        }
        match tools::hypothesis::execute_get_knowledge(input, &self.context.database).await {
            Ok(result) => {
                self.record_tool_execution("get_knowledge", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "extract_knowledge", description = "Extract knowledge from a validated hypothesis.")]
    async fn extract_knowledge(
        &self,
        Parameters(input): Parameters<tools::hypothesis::ExtractKnowledgeInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("extract_knowledge").await {
            tracing::warn!("Workflow enforcement blocked extract_knowledge: {}", e.message);
            return enforcement_error_to_content(e);
        }
        match tools::hypothesis::execute_extract_knowledge(input, &self.context.database).await {
            Ok(result) => {
                self.record_tool_execution("extract_knowledge", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "add_knowledge", description = "Add new validated knowledge to the knowledge base")]
    async fn add_knowledge(
        &self,
        Parameters(input): Parameters<tools::knowledge::AddKnowledgeInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("add_knowledge").await {
            tracing::warn!("Workflow enforcement blocked add_knowledge: {}", e.message);
            return enforcement_error_to_content(e);
        }
        let result = tools::knowledge::execute_add_knowledge(input, &self.context.knowledge).await;
        if result.success {
            self.record_tool_execution("add_knowledge", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(name = "query_knowledge", description = "Query the knowledge base for relevant knowledge")]
    async fn query_knowledge(
        &self,
        Parameters(input): Parameters<tools::knowledge::QueryKnowledgeInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("query_knowledge").await {
            tracing::warn!("Workflow enforcement blocked query_knowledge: {}", e.message);
            return enforcement_error_to_content(e);
        }
        let result = tools::knowledge::execute_query_knowledge(input, &self.context.knowledge).await;
        if result.success {
            self.record_tool_execution("query_knowledge", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(name = "record_knowledge_application", description = "Record the result of applying knowledge")]
    async fn record_knowledge_application(
        &self,
        Parameters(input): Parameters<tools::knowledge::RecordKnowledgeApplicationInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("record_knowledge_application").await {
            tracing::warn!("Workflow enforcement blocked record_knowledge_application: {}", e.message);
            return enforcement_error_to_content(e);
        }
        let result = tools::knowledge::execute_record_knowledge_application(input, &self.context.knowledge).await;
        if result.success {
            self.record_tool_execution("record_knowledge_application", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(name = "get_knowledge_stats", description = "Get statistics about the knowledge base")]
    async fn get_knowledge_stats(
        &self,
        Parameters(input): Parameters<tools::knowledge::GetKnowledgeStatsInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("get_knowledge_stats").await {
            tracing::warn!("Workflow enforcement blocked get_knowledge_stats: {}", e.message);
            return enforcement_error_to_content(e);
        }
        let result = tools::knowledge::execute_get_knowledge_stats(input, &self.context.knowledge).await;
        if result.success {
            self.record_tool_execution("get_knowledge_stats", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(name = "get_mature_knowledge", description = "Get all mature (high-confidence) knowledge")]
    async fn get_mature_knowledge(
        &self,
        Parameters(input): Parameters<tools::knowledge::GetMatureKnowledgeInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("get_mature_knowledge").await {
            tracing::warn!("Workflow enforcement blocked get_mature_knowledge: {}", e.message);
            return enforcement_error_to_content(e);
        }
        let result = tools::knowledge::execute_get_mature_knowledge(input, &self.context.knowledge).await;
        if result.success {
            self.record_tool_execution("get_mature_knowledge", None).await;
        }
        tool_output_to_content(result)
    }
}
