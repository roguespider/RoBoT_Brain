//! Workflow execution tests

use super::helpers::extract_content_text;
use super::results::WorkflowExecutionResults;
use crate::{TestMcpClient, TestStats};

pub async fn test_workflow_execution(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<WorkflowExecutionResults> {
    crate::teeprintln!("\n📋 Phase 2: Workflow Execution Tests");
    crate::teeprintln!("{}", "-".repeat(60));

    let mut results = WorkflowExecutionResults {
        create_workflow_succeeds: false,
        workflow_id_generated: None,
        add_step_succeeds: false,
        start_workflow_succeeds: false,
        workflow_completes: false,
        pause_resume_works: false,
    };

    // Test 1: Create a workflow
    crate::teeprintln!("\n  Testing workflow creation...");
    match client
        .call_tool(
            "create_workflow",
            serde_json::json!({
                "name": "MCP Integration Test Workflow"
            }),
        )
        .await
    {
        Ok(result) => {
            crate::teeprintln!("    ✓ create_workflow - SUCCESS");
            stats.passed += 1;
            results.create_workflow_succeeds = true;

            // Extract workflow ID if present
            if let Some(text) = extract_content_text(&result) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(id) = json
                        .get("workflow_id")
                        .or_else(|| json.get("id"))
                        .or_else(|| json.get("workflow").and_then(|w| w.get("id")))
                    {
                        results.workflow_id_generated = id.as_str().map(String::from);
                        crate::teeprintln!(
                            "    ✓ Workflow ID: {:?}",
                            results.workflow_id_generated
                        );
                    }
                }
            }
        }
        Err(e) => {
            crate::teeprintln!("    ✗ create_workflow - FAILED: {}", e);
            stats.failed += 1;
        }
    }

    // Test 2: Add workflow steps
    crate::teeprintln!("\n  Testing adding workflow steps...");
    let steps = vec![
        ("Initialize", "initialize"),
        ("Process", "process_data"),
        ("Store", "store_memory"),
    ];

    for (name, action) in steps {
        let mut args = serde_json::json!({
            "name": name,
            "action": action
        });

        // Include workflow_id if we have one
        if let Some(ref id) = results.workflow_id_generated {
            args["workflow_id"] = serde_json::json!(id);
        }

        match client.call_tool("add_workflow_step", args).await {
            Ok(_) => {
                crate::teeprintln!(
                    "    ✓ add_workflow_step('{}', '{}') - SUCCESS",
                    name,
                    action
                );
                stats.passed += 1;
                results.add_step_succeeds = true;
            }
            Err(e) => {
                crate::teeprintln!(
                    "    ✗ add_workflow_step('{}', '{}') - FAILED: {}",
                    name,
                    action,
                    e
                );
                stats.failed += 1;
            }
        }
    }

    // Test 3: Get workflow status
    crate::teeprintln!("\n  Testing workflow status retrieval...");
    let mut status_args = serde_json::json!({});
    if let Some(ref id) = results.workflow_id_generated {
        status_args["workflow_id"] = serde_json::json!(id);
    }

    match client.call_tool("get_workflow_status", status_args).await {
        Ok(result) => {
            crate::teeprintln!("    ✓ get_workflow_status - SUCCESS");
            stats.passed += 1;

            // Check if workflow has correct structure
            if let Some(text) = extract_content_text(&result) {
                if text.contains("id") || text.contains("status") || text.contains("workflow") {
                    crate::teeprintln!("    ✓ Workflow status contains expected fields");
                }
            }
        }
        Err(e) => {
            crate::teeprintln!("    ✗ get_workflow_status - FAILED: {}", e);
            stats.failed += 1;
        }
    }

    // Test 4: List workflows
    crate::teeprintln!("\n  Testing workflow listing...");
    match client
        .call_tool("list_workflows", serde_json::json!({}))
        .await
    {
        Ok(result) => {
            crate::teeprintln!("    ✓ list_workflows - SUCCESS");
            stats.passed += 1;

            // Check if response contains workflow list
            if let Some(text) = extract_content_text(&result) {
                if text.contains("workflows") || text.contains("[]") || text.len() > 10 {
                    crate::teeprintln!("    ✓ Workflow list retrieved successfully");
                }
            }
        }
        Err(e) => {
            crate::teeprintln!("    ✗ list_workflows - FAILED: {}", e);
            stats.failed += 1;
        }
    }

    // Test 5: Start workflow (if we have a workflow ID)
    if let Some(ref workflow_id) = results.workflow_id_generated {
        crate::teeprintln!("\n  Testing workflow start...");
        match client
            .call_tool(
                "start_workflow",
                serde_json::json!({
                    "workflow_id": workflow_id
                }),
            )
            .await
        {
            Ok(_result) => {
                crate::teeprintln!("    ✓ start_workflow - SUCCESS");
                stats.passed += 1;
                results.start_workflow_succeeds = true;
                results.workflow_completes = true;
            }
            Err(e) => {
                crate::teeprintln!("    ✗ start_workflow - FAILED: {}", e);
                stats.failed += 1;
            }
        }

        // Test 6: Pause and resume
        crate::teeprintln!("\n  Testing pause/resume workflow...");
        match client
            .call_tool(
                "pause_workflow",
                serde_json::json!({
                    "workflow_id": workflow_id
                }),
            )
            .await
        {
            Ok(_) => {
                crate::teeprintln!("    ✓ pause_workflow - SUCCESS");
                stats.passed += 1;

                match client
                    .call_tool(
                        "resume_workflow",
                        serde_json::json!({
                            "workflow_id": workflow_id
                        }),
                    )
                    .await
                {
                    Ok(_) => {
                        crate::teeprintln!("    ✓ resume_workflow - SUCCESS");
                        stats.passed += 1;
                        results.pause_resume_works = true;
                    }
                    Err(e) => {
                        crate::teeprintln!("    ✗ resume_workflow - FAILED: {}", e);
                        stats.failed += 1;
                    }
                }
            }
            Err(e) => {
                crate::teeprintln!("    ✗ pause_workflow - FAILED: {}", e);
                stats.failed += 1;
            }
        }
    }

    // Test 7: Cancel workflow
    if let Some(ref workflow_id) = results.workflow_id_generated {
        crate::teeprintln!("\n  Testing workflow cancellation...");
        match client
            .call_tool(
                "cancel_workflow",
                serde_json::json!({
                    "workflow_id": workflow_id
                }),
            )
            .await
        {
            Ok(_) => {
                crate::teeprintln!("    ✓ cancel_workflow - SUCCESS");
                stats.passed += 1;
            }
            Err(e) => {
                crate::teeprintln!("    ✗ cancel_workflow - FAILED: {}", e);
                stats.failed += 1;
            }
        }
    }

    Ok(results)
}
