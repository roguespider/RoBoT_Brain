//! MCP Workflow Integration Tests
//!
//! This module specifically tests the integration between the agent and MCP workflows.
//! It ensures that:
//! 1. The agent discovers and uses workflows through MCP
//! 2. Workflow tools are properly registered and accessible
//! 3. The agent follows workflow patterns for different task types
//! 4. End-to-end workflow execution works correctly
//!
//! This test suite validates the MCP tool system by testing:
//! - Tool discovery and registration
//! - Tool execution (if server supports it)
//! - Workflow management
//! - Agent-tool interaction patterns

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

    // First, test the MCP protocol itself
    crate::teeprintln!("\n📋 PHASE 0: MCP PROTOCOL VALIDATION");
    crate::teeprintln!("{}", "-".repeat(60));
    
    let mcp_protocol_ok = test_mcp_protocol(client, stats).await?;
    
    if !mcp_protocol_ok {
        crate::teeprintln!("\n⚠️  MCP Protocol tests failed - server may not be properly configured");
        crate::teeprintln!("    The MCP server needs to implement ServerHandler trait methods:");
        crate::teeprintln!("    - list_tools() - for listing available tools");
        crate::teeprintln!("    - call_tool() - for executing tools");
        crate::teeprintln!("    - get_tool() - for getting tool definitions");
    }

    // Run discovery tests
    let workflow_discovery = discovery::test_workflow_discovery(client, stats).await?;
    
    // Run execution tests (may fail if call_tool not implemented)
    let workflow_execution = execution::test_workflow_execution(client, stats).await?;
    
    // Run tools validation
    let workflow_tools = tools::test_workflow_tools(client, stats).await?;
    
    // Run integration tests
    let agent_workflow_integration = integration::test_agent_workflow_integration(client, stats)
        .await?;
    
    // Run scenarios
    let end_to_end_scenarios = scenarios::test_end_to_end_scenarios(client, stats).await?;

    let results = McpWorkflowTestResults {
        workflow_discovery,
        workflow_execution,
        workflow_tools,
        agent_workflow_integration,
        end_to_end_scenarios,
        mcp_protocol_valid: mcp_protocol_ok,
    };

    helpers::print_mcp_workflow_results(&results);

    Ok(results)
}

/// Test basic MCP protocol functionality
async fn test_mcp_protocol(
    client: &mut crate::TestMcpClient,
    stats: &mut crate::TestStats,
) -> anyhow::Result<bool> {
    let mut all_ok = true;
    
    // Test 1: Initialize handshake (already done in client creation, but verify)
    crate::teeprintln!("  Testing MCP protocol initialization...");
    
    // Test 2: Try to list tools
    crate::teeprintln!("  Testing tools/list method...");
    match client.list_tools().await {
        Ok(tools) => {
            crate::teeprintln!("    ✓ tools/list - SUCCESS ({} tools returned)", tools.len());
            stats.passed += 1;
            
            // Show what tools are available
            if tools.is_empty() {
                crate::teeprintln!("    ⚠️  Server returned 0 tools via MCP protocol");
                crate::teeprintln!("    ℹ  The server claimed 87 tools during initialization");
                crate::teeprintln!("    ℹ  Issue: list_tools() method not properly implemented");
                crate::teeprintln!("    ℹ  Root cause: ServerHandler trait list_tools() returns empty");
                
                // Check if this is a known issue
                all_ok = false;
            } else {
                crate::teeprintln!("    ✅ Server returned {} tools", tools.len());
                // Show first few tool names
                let tool_names: Vec<_> = tools.iter()
                    .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
                    .take(5)
                    .collect();
                if !tool_names.is_empty() {
                    crate::teeprintln!("    ℹ  Sample tools: {:?}", tool_names);
                }
            }
        }
        Err(e) => {
            crate::teeprintln!("    ✗ tools/list - FAILED: {}", e);
            crate::teeprintln!("    ℹ  This is expected if the server uses old MCP protocol");
            stats.failed += 1;
            all_ok = false;
        }
    }
    
    // Test 3: Try to call a tool directly via tools/call
    crate::teeprintln!("  Testing tools/call method...");
    match client.call_tool("get_workflow", serde_json::json!({"purpose": "test"})).await {
        Ok(result) => {
            crate::teeprintln!("    ✅ tools/call - SUCCESS");
            stats.passed += 1;
            
            // Try to parse the result
            if let Some(text) = result.get("content").and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|t| t.get("text"))
                .and_then(|t| t.as_str()) 
            {
                if text.len() < 100 {
                    crate::teeprintln!("    ℹ  Result: {}", text);
                }
            }
        }
        Err(e) => {
            // Check if it's a method_not_found error
            let error_str = e.to_string();
            if error_str.contains("method_not_found") || error_str.contains("-32601") {
                crate::teeprintln!("    ✗ tools/call - NOT IMPLEMENTED");
                crate::teeprintln!("    ⚠️  ROOT CAUSE: ServerHandler::call_tool() not implemented");
                crate::teeprintln!("    ℹ  The MCP server needs to implement call_tool() method");
                crate::teeprintln!("    ℹ  This is the PRIMARY BLOCKER for MCP tool testing");
            } else {
                crate::teeprintln!("    ✗ tools/call - ERROR: {}", e);
            }
            stats.failed += 1;
            all_ok = false;
        }
    }
    
    // Provide summary of what's needed to fix
    if !all_ok {
        crate::teeprintln!("\n  📋 SUMMARY: MCP Server Fix Required");
        crate::teeprintln!("  ────────────────────────────────────────");
        crate::teeprintln!("  To enable full MCP tool support, implement in src/bridge/rmcp/mod.rs:");
        crate::teeprintln!("");
        crate::teeprintln!("  1. list_tools() method:");
        crate::teeprintln!("     - Should return ListToolsResult with all registered tools");
        crate::teeprintln!("     - Collect tools from ToolHandlerCollection");
        crate::teeprintln!("     - Convert Tool structs to rmcp::model::Tool");
        crate::teeprintln!("");
        crate::teeprintln!("  2. call_tool() method:");
        crate::teeprintln!("     - Should execute tools by name");
        crate::teeprintln!("     - Route to appropriate handler via ToolHandlerCollection");
        crate::teeprintln!("     - Return CallToolResult with execution output");
        crate::teeprintln!("");
        crate::teeprintln!("  3. get_tool() method (optional):");
        crate::teeprintln!("     - Return Tool definition for a specific tool name");
    }
    
    Ok(all_ok)
}
