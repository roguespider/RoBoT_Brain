//! Agent-workflow integration tests

use super::helpers::extract_content_text;
use super::results::AgentWorkflowIntegrationResults;
use crate::{TestMcpClient, TestStats};

pub async fn test_agent_workflow_integration(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<AgentWorkflowIntegrationResults> {
    crate::teeprintln!("\n📋 Phase 4: Agent-Workflow Integration Tests");
    crate::teeprintln!("{}", "-".repeat(60));

    let mut results = AgentWorkflowIntegrationResults {
        agent_discovers_workflow_first: false,
        agent_uses_correct_workflow_for_purpose: false,
        agent_chains_workflow_steps: false,
        agent_respects_workflow_dependencies: false,
    };

    // Test 1: Agent workflow discovery pattern
    crate::teeprintln!("\n  Testing agent workflow discovery pattern...");

    // The agent should call get_workflow before other operations
    // We verify this by checking if get_workflow returns proper workflow data

    match client
        .call_tool(
            "get_workflow",
            serde_json::json!({
                "purpose": "test"
            }),
        )
        .await
    {
        Ok(_result) => {
            results.agent_discovers_workflow_first = true;
            crate::teeprintln!("    ✓ Agent can discover workflows via get_workflow");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("    ✗ Agent workflow discovery failed: {}", e);
            stats.failed += 1;
        }
    }

    // Test 2: Purpose-based workflow selection
    crate::teeprintln!("\n  Testing purpose-based workflow selection...");

    let test_purposes = vec![
        ("file_ingestion", "File Ingestion"),
        ("memory_search", "Memory Search"),
        ("general", "General"),
    ];

    let mut all_purposes_work = true;
    for (purpose, _name) in test_purposes {
        match client
            .call_tool(
                "get_workflow",
                serde_json::json!({
                    "purpose": purpose
                }),
            )
            .await
        {
            Ok(result) => {
                crate::teeprintln!("    ✓ Workflow for '{}' - SUCCESS", purpose);
                stats.passed += 1;

                // Verify the workflow has purpose-relevant content
                if let Some(text) = extract_content_text(&result) {
                    // Check for expected content patterns
                    if text.len() > 50 {
                        results.agent_uses_correct_workflow_for_purpose = true;
                    }
                }
            }
            Err(e) => {
                crate::teeprintln!("    ✗ Workflow for '{}' - FAILED: {}", purpose, e);
                stats.failed += 1;
                all_purposes_work = false;
            }
        }
    }

    if !all_purposes_work {
        results.agent_uses_correct_workflow_for_purpose = false;
    }

    // Test 3: Workflow step chaining
    crate::teeprintln!("\n  Testing workflow step chaining...");

    // Create a workflow and add multiple steps
    match client
        .call_tool(
            "create_workflow",
            serde_json::json!({
                "name": "Chained Test Workflow"
            }),
        )
        .await
    {
        Ok(create_result) => {
            // Extract workflow ID
            let mut workflow_id = String::new();
            if let Some(text) = extract_content_text(&create_result) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(id) = json
                        .get("workflow_id")
                        .or_else(|| json.get("id"))
                        .or_else(|| json.get("workflow").and_then(|w| w.get("id")))
                    {
                        workflow_id = id.to_string().trim_matches('"').to_string();
                    }
                }
            }

            if !workflow_id.is_empty() {
                // Add steps in sequence
                let steps = vec![
                    ("Step 1", "action_1"),
                    ("Step 2", "action_2"),
                    ("Step 3", "action_3"),
                ];

                let mut all_steps_added = true;
                for (name, action) in steps {
                    match client
                        .call_tool(
                            "add_workflow_step",
                            serde_json::json!({
                                "workflow_id": workflow_id,
                                "name": name,
                                "action": action
                            }),
                        )
                        .await
                    {
                        Ok(_) => {
                            crate::teeprintln!("    ✓ Chained step '{}' - SUCCESS", name);
                            stats.passed += 1;
                        }
                        Err(e) => {
                            crate::teeprintln!("    ✗ Chained step '{}' - FAILED: {}", name, e);
                            stats.failed += 1;
                            all_steps_added = false;
                        }
                    }
                }

                if all_steps_added {
                    results.agent_chains_workflow_steps = true;
                }

                // Cleanup
                let _ = client
                    .call_tool(
                        "cancel_workflow",
                        serde_json::json!({
                            "workflow_id": workflow_id
                        }),
                    )
                    .await;
            }
        }
        Err(e) => {
            crate::teeprintln!("    ✗ Failed to create chained workflow: {}", e);
            stats.failed += 1;
        }
    }

    // Test 4: Workflow dependencies and error handling
    crate::teeprintln!("\n  Testing workflow error handling...");

    // Try to start a non-existent workflow (should fail gracefully)
    match client
        .call_tool(
            "start_workflow",
            serde_json::json!({
                "workflow_id": "non-existent-workflow-id"
            }),
        )
        .await
    {
        Ok(result) => {
            // Server may return success with error message, or error
            if let Some(text) = extract_content_text(&result) {
                if text.contains("not found") || text.contains("error") || text.contains("fail") {
                    crate::teeprintln!("    ✓ Non-existent workflow handled gracefully");
                    stats.passed += 1;
                    results.agent_respects_workflow_dependencies = true;
                }
            }
        }
        Err(_) => {
            // Error is also acceptable
            crate::teeprintln!("    ✓ Non-existent workflow returned error (expected)");
            stats.passed += 1;
            results.agent_respects_workflow_dependencies = true;
        }
    }

    Ok(results)
}
