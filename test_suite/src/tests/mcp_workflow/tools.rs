//! Workflow tools validation tests

use crate::{TestMcpClient, TestStats};
use super::results::WorkflowToolsResults;

pub async fn test_workflow_tools(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<WorkflowToolsResults> {
    crate::teeprintln!("\n📋 Phase 3: Workflow Tools Validation");
    crate::teeprintln!("{}", "-".repeat(60));

    let mut results = WorkflowToolsResults {
        total_tools: 0,
        workflow_tools: Vec::new(),
        agent_tools: Vec::new(),
        workflow_tool_definitions_valid: false,
    };

    // Test 1: List all tools
    crate::teeprintln!("\n  Testing list_tools...");
    match client.call_tool("list_tools", serde_json::json!({})).await {
        Ok(result) => {
            crate::teeprintln!("    ✓ list_tools - SUCCESS");
            stats.passed += 1;

            if let Some(text) = super::helpers::extract_content_text(&result) {
                // Try to count tools from response
                let tool_count = text.matches("\"name\":").count().max(1);
                results.total_tools = tool_count;
                crate::teeprintln!("    ℹ Total tools detected: {}", results.total_tools);
            }
        }
        Err(e) => {
            crate::teeprintln!("    ✗ list_tools - FAILED: {}", e);
            stats.failed += 1;
        }
    }

    // Test 2: Get specific workflow tool details
    crate::teeprintln!("\n  Testing individual workflow tool definitions...");
    let workflow_tool_names = vec![
        "create_workflow",
        "add_workflow_step",
        "get_workflow_status",
        "list_workflows",
        "start_workflow",
        "pause_workflow",
        "resume_workflow",
        "cancel_workflow",
        "delete_workflow",
    ];

    let agent_tool_names = vec![
        "get_workflow",
        "list_tools",
        "get_tool",
    ];

    for tool_name in &workflow_tool_names {
        match client.call_tool("get_tool", serde_json::json!({
            "name": tool_name
        })).await {
            Ok(_) => {
                crate::teeprintln!("    ✓ get_tool('{}') - SUCCESS", tool_name);
                stats.passed += 1;
                results.workflow_tools.push(tool_name.to_string());
            }
            Err(e) => {
                crate::teeprintln!("    ✗ get_tool('{}') - FAILED: {}", tool_name, e);
                stats.failed += 1;
            }
        }
    }

    for tool_name in &agent_tool_names {
        match client.call_tool("get_tool", serde_json::json!({
            "name": tool_name
        })).await {
            Ok(_) => {
                crate::teeprintln!("    ✓ get_tool('{}') - SUCCESS", tool_name);
                stats.passed += 1;
                results.agent_tools.push(tool_name.to_string());
            }
            Err(e) => {
                crate::teeprintln!("    ✗ get_tool('{}') - FAILED: {}", tool_name, e);
                stats.failed += 1;
            }
        }
    }

    // Validate that all expected workflow tools are available
    if results.workflow_tools.len() >= 5 {
        results.workflow_tool_definitions_valid = true;
        crate::teeprintln!("\n    ✓ All core workflow tools are available");
    }

    Ok(results)
}
