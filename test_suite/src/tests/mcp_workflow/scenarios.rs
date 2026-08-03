//! End-to-end scenario tests

use super::helpers::{extract_content_text, verify_workflow_tools_exist};
use super::results::EndToEndScenarioResults;
use crate::{TestMcpClient, TestStats};

pub async fn test_end_to_end_scenarios(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<EndToEndScenarioResults> {
    crate::teeprintln!("\n📋 Phase 5: End-to-End Scenario Tests");
    crate::teeprintln!("{}", "-".repeat(60));

    let mut results = EndToEndScenarioResults {
        file_ingestion_workflow: false,
        memory_search_workflow: false,
        experience_recording_workflow: false,
        multi_step_workflow: false,
    };

    // Scenario 1: File Ingestion Workflow
    crate::teeprintln!("\n  Testing File Ingestion Workflow...");
    match client
        .call_tool(
            "get_workflow",
            serde_json::json!({
                "purpose": "file_ingestion"
            }),
        )
        .await
    {
        Ok(_) => {
            // Verify workflow tools are available
            let tools_exist = verify_workflow_tools_exist(
                client,
                &["create_workflow", "add_workflow_step", "start_workflow"],
            )
            .await;

            if tools_exist {
                crate::teeprintln!("    ✓ File ingestion workflow path available");
                stats.passed += 1;
                results.file_ingestion_workflow = true;
            }
        }
        Err(e) => {
            crate::teeprintln!("    ✗ File ingestion workflow failed: {}", e);
            stats.failed += 1;
        }
    }

    // Scenario 2: Memory Search Workflow
    crate::teeprintln!("\n  Testing Memory Search Workflow...");
    match client
        .call_tool(
            "get_workflow",
            serde_json::json!({
                "purpose": "memory_search"
            }),
        )
        .await
    {
        Ok(_) => {
            let tools_exist =
                verify_workflow_tools_exist(client, &["create_workflow", "list_workflows"]).await;

            if tools_exist {
                crate::teeprintln!("    ✓ Memory search workflow path available");
                stats.passed += 1;
                results.memory_search_workflow = true;
            }
        }
        Err(e) => {
            crate::teeprintln!("    ✗ Memory search workflow failed: {}", e);
            stats.failed += 1;
        }
    }

    // Scenario 3: Experience Recording Workflow
    crate::teeprintln!("\n  Testing Experience Recording Workflow...");
    match client
        .call_tool(
            "get_workflow",
            serde_json::json!({
                "purpose": "experience_recording"
            }),
        )
        .await
    {
        Ok(_) => {
            crate::teeprintln!("    ✓ Experience recording workflow path available");
            stats.passed += 1;
            results.experience_recording_workflow = true;
        }
        Err(e) => {
            crate::teeprintln!("    ✗ Experience recording workflow failed: {}", e);
            stats.failed += 1;
        }
    }

    // Scenario 4: Multi-Step Workflow Execution
    crate::teeprintln!("\n  Testing Multi-Step Workflow...");

    // Create a workflow with multiple steps and execute it
    match client
        .call_tool(
            "create_workflow",
            serde_json::json!({
                "name": "Multi-Step E2E Test"
            }),
        )
        .await
    {
        Ok(result) => {
            let mut workflow_id = String::new();
            if let Some(text) = extract_content_text(&result) {
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
                // Add multiple steps
                let steps = vec![
                    ("Initialize", "init"),
                    ("Process Data", "process"),
                    ("Store Results", "store"),
                    ("Notify", "notify"),
                ];

                let steps_count = steps.len();
                let mut steps_added = 0;
                for (name, action) in &steps {
                    if client
                        .call_tool(
                            "add_workflow_step",
                            serde_json::json!({
                                "workflow_id": workflow_id,
                                "name": name,
                                "action": action
                            }),
                        )
                        .await
                        .is_ok()
                    {
                        steps_added += 1;
                    }
                }

                if steps_added == steps_count {
                    // Start the workflow
                    match client
                        .call_tool(
                            "start_workflow",
                            serde_json::json!({
                                "workflow_id": workflow_id
                            }),
                        )
                        .await
                    {
                        Ok(_) => {
                            crate::teeprintln!("    ✓ Multi-step workflow executed successfully");
                            stats.passed += 1;
                            results.multi_step_workflow = true;
                        }
                        Err(e) => {
                            crate::teeprintln!("    ✗ Multi-step workflow start failed: {}", e);
                            stats.failed += 1;
                        }
                    }
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
            crate::teeprintln!("    ✗ Multi-step workflow creation failed: {}", e);
            stats.failed += 1;
        }
    }

    Ok(results)
}
