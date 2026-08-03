//! Helper functions for MCP Workflow tests

/// Extract text content from MCP response
pub fn extract_content_text(result: &serde_json::Value) -> Option<String> {
    // Try direct text field
    if let Some(text) = result.get("text").and_then(|t| t.as_str()) {
        return Some(text.to_string());
    }

    // Try content[0].text (MCP response format)
    if let Some(content) = result
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
    {
        if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
            return Some(text.to_string());
        }
    }

    // Try to serialize entire result
    serde_json::to_string(result).ok()
}

/// Verify multiple workflow tools exist
pub async fn verify_workflow_tools_exist(
    client: &mut crate::TestMcpClient,
    tool_names: &[&str],
) -> bool {
    let mut all_exist = true;

    for tool_name in tool_names {
        match client
            .call_tool(
                "get_tool",
                serde_json::json!({
                    "name": tool_name
                }),
            )
            .await
        {
            Ok(_) => {}
            Err(_) => all_exist = false,
        }
    }

    all_exist
}

/// Print comprehensive MCP workflow test results
pub fn print_mcp_workflow_results(results: &super::results::McpWorkflowTestResults) {
    crate::teeprintln!("\n{}", "=".repeat(80));
    crate::teeprintln!("MCP WORKFLOW INTEGRATION TEST RESULTS");
    crate::teeprintln!("{}", "=".repeat(80));

    // Discovery Results
    crate::teeprintln!("\n📋 Workflow Discovery:");
    crate::teeprintln!(
        "  - get_workflow tool available: {}",
        if results.workflow_discovery.get_workflow_available {
            "✓"
        } else {
            "✗"
        }
    );
    crate::teeprintln!(
        "  - Default workflow retrieved: {}",
        if results.workflow_discovery.default_workflow_retrieved {
            "✓"
        } else {
            "✗"
        }
    );
    crate::teeprintln!(
        "  - Purpose-based workflows found: {}",
        results.workflow_discovery.purpose_based_workflows.len()
    );
    crate::teeprintln!(
        "  - Workflow rules understood: {}",
        if results.workflow_discovery.workflow_rules_understood {
            "✓"
        } else {
            "✗"
        }
    );

    // Execution Results
    crate::teeprintln!("\n⚙️  Workflow Execution:");
    crate::teeprintln!(
        "  - Workflow creation: {}",
        if results.workflow_execution.create_workflow_succeeds {
            "✓"
        } else {
            "✗"
        }
    );
    crate::teeprintln!(
        "  - Workflow ID generated: {:?}",
        results.workflow_execution.workflow_id_generated
    );
    crate::teeprintln!(
        "  - Step addition: {}",
        if results.workflow_execution.add_step_succeeds {
            "✓"
        } else {
            "✗"
        }
    );
    crate::teeprintln!(
        "  - Workflow start: {}",
        if results.workflow_execution.start_workflow_succeeds {
            "✓"
        } else {
            "✗"
        }
    );
    crate::teeprintln!(
        "  - Workflow completion: {}",
        if results.workflow_execution.workflow_completes {
            "✓"
        } else {
            "✗"
        }
    );
    crate::teeprintln!(
        "  - Pause/Resume: {}",
        if results.workflow_execution.pause_resume_works {
            "✓"
        } else {
            "✗"
        }
    );

    // Tools Results
    crate::teeprintln!("\n🔧 Workflow Tools:");
    crate::teeprintln!("  - Total tools: {}", results.workflow_tools.total_tools);
    crate::teeprintln!(
        "  - Workflow tools available: {}",
        results.workflow_tools.workflow_tools.len()
    );
    crate::teeprintln!(
        "  - Agent tools available: {}",
        results.workflow_tools.agent_tools.len()
    );
    crate::teeprintln!(
        "  - Tool definitions valid: {}",
        if results.workflow_tools.workflow_tool_definitions_valid {
            "✓"
        } else {
            "✗"
        }
    );

    // Agent Integration Results
    crate::teeprintln!("\n🤖 Agent-Workflow Integration:");
    crate::teeprintln!(
        "  - Agent discovers workflow: {}",
        if results
            .agent_workflow_integration
            .agent_discovers_workflow_first
        {
            "✓"
        } else {
            "✗"
        }
    );
    crate::teeprintln!(
        "  - Agent uses purpose-based workflows: {}",
        if results
            .agent_workflow_integration
            .agent_uses_correct_workflow_for_purpose
        {
            "✓"
        } else {
            "✗"
        }
    );
    crate::teeprintln!(
        "  - Agent chains workflow steps: {}",
        if results
            .agent_workflow_integration
            .agent_chains_workflow_steps
        {
            "✓"
        } else {
            "✗"
        }
    );
    crate::teeprintln!(
        "  - Agent respects dependencies: {}",
        if results
            .agent_workflow_integration
            .agent_respects_workflow_dependencies
        {
            "✓"
        } else {
            "✗"
        }
    );

    // E2E Results
    crate::teeprintln!("\n🔄 End-to-End Scenarios:");
    crate::teeprintln!(
        "  - File ingestion workflow: {}",
        if results.end_to_end_scenarios.file_ingestion_workflow {
            "✓"
        } else {
            "✗"
        }
    );
    crate::teeprintln!(
        "  - Memory search workflow: {}",
        if results.end_to_end_scenarios.memory_search_workflow {
            "✓"
        } else {
            "✗"
        }
    );
    crate::teeprintln!(
        "  - Experience recording workflow: {}",
        if results.end_to_end_scenarios.experience_recording_workflow {
            "✓"
        } else {
            "✗"
        }
    );
    crate::teeprintln!(
        "  - Multi-step workflow: {}",
        if results.end_to_end_scenarios.multi_step_workflow {
            "✓"
        } else {
            "✗"
        }
    );

    // Overall Assessment
    let total_checks = 20;
    let passed_checks = [
        results.workflow_discovery.get_workflow_available,
        results.workflow_discovery.default_workflow_retrieved,
        results.workflow_discovery.workflow_rules_understood,
        results.workflow_execution.create_workflow_succeeds,
        results.workflow_execution.add_step_succeeds,
        results.workflow_execution.start_workflow_succeeds,
        results.workflow_execution.pause_resume_works,
        results.workflow_tools.workflow_tool_definitions_valid,
        results
            .agent_workflow_integration
            .agent_discovers_workflow_first,
        results
            .agent_workflow_integration
            .agent_uses_correct_workflow_for_purpose,
        results
            .agent_workflow_integration
            .agent_chains_workflow_steps,
        results
            .agent_workflow_integration
            .agent_respects_workflow_dependencies,
        results.end_to_end_scenarios.file_ingestion_workflow,
        results.end_to_end_scenarios.memory_search_workflow,
        results.end_to_end_scenarios.experience_recording_workflow,
        results.end_to_end_scenarios.multi_step_workflow,
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    crate::teeprintln!("\n{}", "-".repeat(80));
    crate::teeprintln!(
        "Overall MCP Workflow Integration Score: {}/{} checks passed ({:.0}%)",
        passed_checks,
        total_checks,
        (passed_checks as f64 / total_checks as f64) * 100.0
    );

    if passed_checks >= total_checks - 2 {
        crate::teeprintln!("\n🎉 AGENT WILL USE MCP WORKFLOWS CORRECTLY!");
    } else if passed_checks >= total_checks / 2 {
        crate::teeprintln!("\n⚠️  PARTIAL MCP WORKFLOW SUPPORT - Some issues need attention");
    } else {
        crate::teeprintln!("\n❌ MCP WORKFLOW INTEGRATION NEEDS SIGNIFICANT IMPROVEMENT");
    }

    crate::teeprintln!("{}", "=".repeat(80));
}
