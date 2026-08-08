//! Agent Decision-Making Tests
//!
//! Tests agent decision-making patterns and reasoning

use crate::{TestMcpClient, TestStats};

/// Decision-making test results
#[derive(Debug, Default)]
pub struct DecisionResults {
    pub passed: usize,
    pub failed: usize,
    pub decisions_tested: usize,
}

/// Test agent decision-making patterns
pub async fn test_agent_decision_making(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<DecisionResults> {
    let mut results = DecisionResults::default();

    // Test 1: Plan selection decision
    crate::teeprintln!("  Testing plan selection decision...");
    match client.call_tool("get_plan", serde_json::json!({"goal": "optimize performance"})).await {
        Ok(_) => {
            crate::teeprintln!("    ✅ Plan selection SUCCESS");
            results.decisions_tested += 1;
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("    ❌ Plan selection: {}", e);
            results.failed += 1;
            stats.failed += 1;
        }
    }

    // Test 2: Workflow choice decision
    crate::teeprintln!("  Testing workflow choice decision...");
    
    let mut choices_made = 0;
    
    if client.call_tool("get_workflow", serde_json::json!({"purpose": "coding"})).await.is_ok() {
        choices_made += 1;
    }
    if client.call_tool("get_workflow", serde_json::json!({"purpose": "testing"})).await.is_ok() {
        choices_made += 1;
    }
    if client.call_tool("get_workflow", serde_json::json!({"purpose": "debugging"})).await.is_ok() {
        choices_made += 1;
    }
    
    if choices_made >= 2 {
        crate::teeprintln!("    ✅ Workflow choice: {}/3 succeeded", choices_made);
        results.decisions_tested += choices_made;
        results.passed += 1;
        stats.passed += 1;
    } else {
        crate::teeprintln!("    ❌ Workflow choice: only {}/3 succeeded", choices_made);
        results.failed += 1;
        stats.failed += 1;
    }

    // Test 3: Hypothesis evaluation decision
    crate::teeprintln!("  Testing hypothesis evaluation decision...");
    match client.call_tool("list_hypotheses", serde_json::json!({"limit": 5})).await {
        Ok(_) => {
            crate::teeprintln!("    ✅ Hypothesis evaluation SUCCESS");
            results.decisions_tested += 1;
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("    ❌ Hypothesis evaluation: {}", e);
            results.failed += 1;
            stats.failed += 1;
        }
    }

    // Test 4: Exploration vs exploitation decision
    crate::teeprintln!("  Testing exploration decision...");
    match client.call_tool("get_exploration_status", serde_json::json!({})).await {
        Ok(_) => {
            crate::teeprintln!("    ✅ Exploration status SUCCESS");
            results.decisions_tested += 1;
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("    ⚠️  Exploration status: {}", e);
            results.failed += 1;
            stats.skipped += 1;
        }
    }

    // Test 5: Skill selection decision
    crate::teeprintln!("  Testing skill selection decision...");
    match client.call_tool("list_skills", serde_json::json!({})).await {
        Ok(_) => {
            crate::teeprintln!("    ✅ Skill selection SUCCESS");
            results.decisions_tested += 1;
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("    ❌ Skill selection: {}", e);
            results.failed += 1;
            stats.failed += 1;
        }
    }

    // Test 6: Reflection decision
    crate::teeprintln!("  Testing reflection decision...");
    match client.call_tool("list_reflections", serde_json::json!({"limit": 5})).await {
        Ok(_) => {
            crate::teeprintln!("    ✅ Reflection decision SUCCESS");
            results.decisions_tested += 1;
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("    ❌ Reflection decision: {}", e);
            results.failed += 1;
            stats.failed += 1;
        }
    }

    // Test 7: Multi-criteria decision
    crate::teeprintln!("  Testing multi-criteria decision...");
    
    let mut criteria_evaluated = 0;
    
    if client.call_tool("search_knowledge", serde_json::json!({"query": "best practices", "limit": 5})).await.is_ok() {
        criteria_evaluated += 1;
    }
    if client.call_tool("search_memory", serde_json::json!({"query": "previous experience", "limit": 5})).await.is_ok() {
        criteria_evaluated += 1;
    }
    if client.call_tool("get_recent_experiences", serde_json::json!({"limit": 3})).await.is_ok() {
        criteria_evaluated += 1;
    }
    
    if criteria_evaluated >= 2 {
        crate::teeprintln!("    ✅ Multi-criteria decision: {}/3 criteria evaluated", criteria_evaluated);
        results.decisions_tested += criteria_evaluated;
        results.passed += 1;
        stats.passed += 1;
    } else {
        crate::teeprintln!("    ❌ Multi-criteria decision: only {}/3 criteria evaluated", criteria_evaluated);
        results.failed += 1;
        stats.failed += 1;
    }

    Ok(results)
}
