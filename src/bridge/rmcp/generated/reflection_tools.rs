// reflection_tools.rs - Reflection and pattern analysis tools

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
#[tool(name = "get_insights", description = "Get actionable insights from reflections")]
async fn get_insights(
    &self,
    Parameters(input): Parameters<tools::reflection::GetInsightsInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_insights").await {
        tracing::warn!("Workflow enforcement blocked get_insights: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::reflection::execute_get_insights(input, &self.context.reflection).await {
        Ok(result) => {
            self.record_tool_execution("get_insights", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "create_reflection", description = "Create a new reflection")]
async fn create_reflection(
    &self,
    Parameters(input): Parameters<tools::reflection::CreateReflectionInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("create_reflection").await {
        tracing::warn!("Workflow enforcement blocked create_reflection: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::reflection::execute_create_reflection(input, &self.context.reflection).await {
        Ok(result) => {
            self.record_tool_execution("create_reflection", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "analyze_patterns", description = "Analyze experiences to detect patterns")]
async fn analyze_patterns(
    &self,
    Parameters(input): Parameters<tools::reflection::AnalyzePatternsInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("analyze_patterns").await {
        tracing::warn!("Workflow enforcement blocked analyze_patterns: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::reflection::execute_analyze_patterns(input, &self.context.reflection).await {
        Ok(result) => {
            self.record_tool_execution("analyze_patterns", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "get_patterns", description = "Get detected patterns")]
async fn get_patterns(
    &self,
    Parameters(input): Parameters<tools::reflection::GetPatternsInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_patterns").await {
        tracing::warn!("Workflow enforcement blocked get_patterns: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::reflection::execute_get_patterns(input, &self.context.reflection).await {
        Ok(result) => {
            self.record_tool_execution("get_patterns", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}
}
