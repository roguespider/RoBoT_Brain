// agent_tools.rs - Agent coordination tools

use crate::bridge::rmcp::generated::tool_traits::{
    AgentToolsHandlerTrait, ToolContext,
};
use crate::tools;
use crate::tools::ToolOutput;

/// Handler for agent tools - implements AgentToolsHandlerTrait
pub struct AgentToolsHandler;

impl AgentToolsHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AgentToolsHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentToolsHandlerTrait for AgentToolsHandler {
    async fn execute_call_mcp_tool(
        &self,
        context: &ToolContext,
        input: tools::agent::CallMcpToolInput,
    ) -> ToolOutput {
        match tools::agent::execute_call_mcp_tool(input, &context.context.mcp_client).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_connect_mcp_server(
        &self,
        context: &ToolContext,
        input: tools::agent::ConnectMcpServerInput,
    ) -> ToolOutput {
        match tools::agent::execute_connect_mcp_server(input, &context.context.mcp_client).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_tool(
        &self,
        context: &ToolContext,
        input: tools::agent::GetToolInput,
    ) -> ToolOutput {
        match tools::agent::execute_get_tool(input, &context.context.mcp_client).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_workflow(
        &self,
        context: &ToolContext,
        input: tools::agent::GetWorkflowInput,
    ) -> ToolOutput {
        match tools::agent::execute_get_workflow(input).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_list_tools(
        &self,
        context: &ToolContext,
        input: tools::agent::ListToolsInput,
    ) -> ToolOutput {
        match tools::agent::execute_list_tools(input, &context.context.mcp_client).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    fn list_tools(&self) -> Vec<rmcp::tool::Tool> {
        vec![
            tools::agent::call_mcp_tool_tool(),
            tools::agent::connect_mcp_server_tool(),
            tools::agent::get_tool_tool(),
            tools::agent::get_workflow_tool(),
            tools::agent::list_tools_tool(),
        ]
    }
}
