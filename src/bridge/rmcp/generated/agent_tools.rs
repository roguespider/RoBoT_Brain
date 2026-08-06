// agent_tools.rs - Agent and MCP server tools

use crate::bridge::rmcp::types::McpServerHandler;
use crate::tools;
use crate::tools::ToolOutput;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ContentBlock;
use rmcp::tool_router;
use rmcp::tool;
use crate::bridge::rmcp::helpers::tool_output_to_content;

#[tool_router]
impl McpServerHandler {
#[tool(
    name = "list_tools",
    description = "List all available MCP tools with optional filter"
)]
async fn list_tools(
    &self,
    Parameters(input): Parameters<tools::agent::ListToolsInput>,
) -> ContentBlock {
    match tools::agent::execute_list_tools(input).await {
        Ok(result) => tool_output_to_content(result),
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(
    name = "get_tool",
    description = "Get detailed information about a specific tool"
)]
async fn get_tool(
    &self,
    Parameters(input): Parameters<tools::agent::GetToolInput>,
) -> ContentBlock {
    match tools::agent::execute_get_tool(input).await {
        Ok(result) => tool_output_to_content(result),
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(
    name = "connect_mcp_server",
    description = "Connect to an external MCP server via child process"
)]
async fn connect_mcp_server(
    &self,
    Parameters(input): Parameters<tools::agent::ConnectMcpServerInput>,
) -> ContentBlock {
    match tools::agent::execute_connect_mcp_server(input).await {
        Ok(result) => tool_output_to_content(result),
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(
    name = "call_tool",
    description = "Call a tool on a connected MCP server"
)]
async fn call_tool(
    &self,
    Parameters(input): Parameters<tools::agent::CallMcpToolInput>,
) -> ContentBlock {
    match tools::agent::execute_call_mcp_tool(input).await {
        Ok(result) => tool_output_to_content(result),
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}
}
