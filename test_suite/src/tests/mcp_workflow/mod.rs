//! MCP Workflow Integration Tests
//!
//! This module specifically tests the integration between the agent and MCP workflows.
//! It ensures that:
//! 1. The agent discovers and uses workflows through MCP
//! 2. Workflow tools are properly registered and accessible
//! 3. The agent follows workflow patterns for different task types
//! 4. End-to-end workflow execution works correctly

pub mod discovery;
pub mod execution;
pub mod helpers;
pub mod integration;
pub mod results;
pub mod scenarios;
pub mod tests;
pub mod tools;

pub use results::McpWorkflowTestResults;

/// Run all MCP workflow integration tests
pub async fn run_mcp_workflow_tests(
    client: &mut crate::TestMcpClient,
    stats: &mut crate::TestStats,
    _filter: Option<&str>,
) -> anyhow::Result<McpWorkflowTestResults> {
    crate::teeprintln!("\n{}", "=".repeat(80));
    crate::teeprintln!("MCP WORKFLOW INTEGRATION TESTS");
    crate::teeprintln!("Testing agent workflow discovery, execution, and MCP integration");
    crate::teeprintln!("{}", "=".repeat(80));

    let results = McpWorkflowTestResults {
        workflow_discovery: discovery::test_workflow_discovery(client, stats).await?,
        workflow_execution: execution::test_workflow_execution(client, stats).await?,
        workflow_tools: tools::test_workflow_tools(client, stats).await?,
        agent_workflow_integration: integration::test_agent_workflow_integration(client, stats)
            .await?,
        end_to_end_scenarios: scenarios::test_end_to_end_scenarios(client, stats).await?,
    };

    helpers::print_mcp_workflow_results(&results);

    Ok(results)
}
