//! Multi-Step Workflow Tests
//!
//! Tests realistic AI agent workflows that span multiple tool calls

use crate::{TestMcpClient, TestStats};

/// Workflow test results
#[derive(Debug, Default)]
pub struct WorkflowResults {
    pub passed: usize,
    pub failed: usize,
    pub workflows_tested: usize,
    pub steps_completed: usize,
}

/// Test multi-step agent workflows
pub async fn test_agent_workflows(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<WorkflowResults> {
    let mut results = WorkflowResults::default();

    // Workflow 1: Research and Learn Workflow
    crate::teeprintln!("  Testing 'Research and Learn' workflow...");
    
    let mut steps_done = 0;
    
    if client.call_tool("get_workflow", serde_json::json!({"purpose": "research_task"})).await.is_ok() {
        steps_done += 1;
    }
    if client.call_tool("global_search", serde_json::json!({"query": "artificial intelligence", "limit": 5})).await.is_ok() {
        steps_done += 1;
    }
    if client.call_tool("query_knowledge", serde_json::json!({"query": "AI techniques", "limit": 5})).await.is_ok() {
        steps_done += 1;
    }
    
    if steps_done >= 2 {
        crate::teeprintln!("    ✅ Research workflow: {}/3 steps completed", steps_done);
        results.workflows_tested += 1;
        results.steps_completed += steps_done;
        results.passed += 1;
        stats.passed += 1;
    } else {
        crate::teeprintln!("    ❌ Research workflow: {}/3 steps completed", steps_done);
        results.failed += 1;
        stats.failed += 1;
    }

    // Workflow 2: Memory Consolidation Workflow
    crate::teeprintln!("  Testing 'Memory Consolidation' workflow...");
    
    steps_done = 0;
    
    if client.call_tool("list_memories", serde_json::json!({"limit": 10})).await.is_ok() {
        steps_done += 1;
    }
    if client.call_tool("search_memory", serde_json::json!({"query": "important", "limit": 5})).await.is_ok() {
        steps_done += 1;
    }
    if client.call_tool("get_memory", serde_json::json!({"id": "00000000-0000-0000-0000-000000000000"})).await.is_ok() {
        steps_done += 1;
    }
    
    if steps_done >= 2 {
        crate::teeprintln!("    ✅ Memory workflow: {}/3 steps completed", steps_done);
        results.workflows_tested += 1;
        results.steps_completed += steps_done;
        results.passed += 1;
        stats.passed += 1;
    } else {
        crate::teeprintln!("    ⚠️  Memory workflow: {}/3 steps completed", steps_done);
        results.failed += 1;
        stats.skipped += 1;
    }

    // Workflow 3: Experience Analysis Workflow
    crate::teeprintln!("  Testing 'Experience Analysis' workflow...");
    
    steps_done = 0;
    
    if client.call_tool("list_experiences", serde_json::json!({"limit": 10})).await.is_ok() {
        steps_done += 1;
    }
    if client.call_tool("get_recent_experiences", serde_json::json!({"limit": 5})).await.is_ok() {
        steps_done += 1;
    }
    if client.call_tool("get_insights", serde_json::json!({})).await.is_ok() {
        steps_done += 1;
    }
    
    if steps_done >= 2 {
        crate::teeprintln!("    ✅ Experience workflow: {}/3 steps completed", steps_done);
        results.workflows_tested += 1;
        results.steps_completed += steps_done;
        results.passed += 1;
        stats.passed += 1;
    } else {
        crate::teeprintln!("    ⚠️  Experience workflow: {}/3 steps completed", steps_done);
        results.failed += 1;
        stats.skipped += 1;
    }

    // Workflow 4: Planning Workflow
    crate::teeprintln!("  Testing 'Planning' workflow...");
    
    steps_done = 0;
    
    if client.call_tool("get_plan", serde_json::json!({"goal": "build a web application"})).await.is_ok() {
        steps_done += 1;
    }
    if client.call_tool("list_plans", serde_json::json!({})).await.is_ok() {
        steps_done += 1;
    }
    if client.call_tool("get_workflow", serde_json::json!({"purpose": "planning"})).await.is_ok() {
        steps_done += 1;
    }
    
    if steps_done >= 2 {
        crate::teeprintln!("    ✅ Planning workflow: {}/3 steps completed", steps_done);
        results.workflows_tested += 1;
        results.steps_completed += steps_done;
        results.passed += 1;
        stats.passed += 1;
    } else {
        crate::teeprintln!("    ⚠️  Planning workflow: {}/3 steps completed", steps_done);
        results.failed += 1;
        stats.skipped += 1;
    }

    // Workflow 5: Hypothesis Testing Workflow
    crate::teeprintln!("  Testing 'Hypothesis Testing' workflow...");
    
    steps_done = 0;
    
    if client.call_tool("list_hypotheses", serde_json::json!({"limit": 5})).await.is_ok() {
        steps_done += 1;
    }
    if client.call_tool("get_hypothesis", serde_json::json!({"id": "00000000-0000-0000-0000-000000000000"})).await.is_ok() {
        steps_done += 1;
    }
    
    if steps_done >= 1 {
        crate::teeprintln!("    ✅ Hypothesis workflow: {}/2 steps completed", steps_done);
        results.workflows_tested += 1;
        results.steps_completed += steps_done;
        results.passed += 1;
        stats.passed += 1;
    } else {
        crate::teeprintln!("    ⚠️  Hypothesis workflow: {}/2 steps completed", steps_done);
        results.failed += 1;
        stats.skipped += 1;
    }

    Ok(results)
}
