//! Workflow discovery tests

use super::helpers::extract_content_text;
use super::results::WorkflowDiscoveryResults;
use crate::{TestMcpClient, TestStats};

pub async fn test_workflow_discovery(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<WorkflowDiscoveryResults> {
    crate::teeprintln!("\n📋 Phase 1: Workflow Discovery Tests");
    crate::teeprintln!("{}", "-".repeat(60));

    let mut results = WorkflowDiscoveryResults {
        get_workflow_available: false,
        default_workflow_retrieved: false,
        purpose_based_workflows: Vec::new(),
        workflow_rules_understood: false,
    };

    // Test 1: get_workflow tool is available
    crate::teeprintln!("\n  Testing get_workflow tool availability...");
    match client
        .call_tool(
            "get_workflow",
            serde_json::json!({
                "purpose": "default"
            }),
        )
        .await
    {
        Ok(result) => {
            crate::teeprintln!("    ✓ get_workflow('default') - SUCCESS");
            stats.passed += 1;
            results.get_workflow_available = true;
            results.default_workflow_retrieved = true;

            // Check if workflow contains rules/instructions
            if let Some(text) = extract_content_text(&result)
                && (text.contains("workflow")
                    || text.contains("rules")
                    || text.contains("guidelines"))
                {
                    results.workflow_rules_understood = true;
                    crate::teeprintln!("    ✓ Workflow rules/instructions present in response");
                }
        }
        Err(e) => {
            crate::teeprintln!("    ✗ get_workflow('default') - FAILED: {}", e);
            stats.failed += 1;
        }
    }

    // Test 2: Get workflow for specific purposes
    crate::teeprintln!("\n  Testing purpose-based workflow retrieval...");
    let purposes = vec![
        "file_ingestion",
        "memory_search",
        "general",
        "experience_recording",
    ];

    for purpose in purposes {
        match client
            .call_tool(
                "get_workflow",
                serde_json::json!({
                    "purpose": purpose
                }),
            )
            .await
        {
            Ok(_result) => {
                crate::teeprintln!("    ✓ get_workflow('{}') - SUCCESS", purpose);
                stats.passed += 1;
                results.purpose_based_workflows.push(purpose.to_string());
            }
            Err(e) => {
                crate::teeprintln!("    ✗ get_workflow('{}') - FAILED: {}", purpose, e);
                stats.failed += 1;
            }
        }
    }

    Ok(results)
}
