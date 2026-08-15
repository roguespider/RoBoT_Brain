//! ACP Agent Tests
//!
//! Tests agent capabilities and agent-related functionality

use crate::{TestMcpClient, TestStats};

/// Agent test results
#[derive(Debug, Default)]
pub struct AgentTestResults {
    pub passed: usize,
    pub failed: usize,
    pub agents_tested: usize,
}

/// Check if an MCP error indicates a missing tool
fn is_tool_not_found(error: &str) -> bool {
    let lower = error.to_lowercase();
    lower.contains("method_not_found") 
        || lower.contains("not found") 
        || lower.contains("not found:")
        || lower.contains("unknown tool")
        || lower.contains("tool not found")
}

/// Test ACP agent functionality
pub async fn test_acp_agents(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<AgentTestResults> {
    let mut results = AgentTestResults::default();

    // Test 1: List registered ACP agents
    crate::teeprintln!("  Testing agent listing...");
    match client.call_tool("list_acp_agents", serde_json::json!({})).await {
        Ok(result) => {
            crate::teeprintln!("    [OK] list_acp_agents SUCCESS");
            results.agents_tested += 1;
            results.passed += 1;
            stats.passed += 1;
            
            if let Some(text) = result.get("content").and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|t| t.get("text"))
                .and_then(|t| t.as_str()) 
            {
                crate::teeprintln!("    [INFO]  Agents: {}", text);
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            if is_tool_not_found(&error_str) {
                crate::teeprintln!("    [SKIP]  SKIPPED: ACP agents not exposed via MCP");
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    [FAIL] list_acp_agents ERROR: {}", e);
                results.failed += 1;
                stats.failed += 1;
            }
        }
    }

    // Test 2: Get agent capabilities
    crate::teeprintln!("  Testing agent capability query...");
    match client.call_tool("get_agent_capabilities", serde_json::json!({
        "agent_id": "system"
    })).await {
        Ok(result) => {
            crate::teeprintln!("    [OK] get_agent_capabilities SUCCESS");
            results.agents_tested += 1;
            results.passed += 1;
            stats.passed += 1;
            
            if let Some(text) = result.get("content").and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|t| t.get("text"))
                .and_then(|t| t.as_str()) 
            {
                crate::teeprintln!("    [INFO]  Capabilities: {}", text);
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            if is_tool_not_found(&error_str) {
                crate::teeprintln!("    [SKIP]  SKIPPED: Agent capabilities not exposed");
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    [FAIL] get_agent_capabilities: {}", e);
                results.failed += 1;
                stats.failed += 1;
            }
        }
    }

    // Test 3: Test system status (agent-related)
    crate::teeprintln!("  Testing system agent status...");
    match client.call_tool("get_system_status", serde_json::json!({})).await {
        Ok(result) => {
            crate::teeprintln!("    [OK] get_system_status SUCCESS");
            results.agents_tested += 1;
            results.passed += 1;
            stats.passed += 1;
            
            if let Some(text) = result.get("content").and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|t| t.get("text"))
                .and_then(|t| t.as_str()) 
            {
                let snippet = if text.len() > 150 { &text[..150] } else { text };
                crate::teeprintln!("    [INFO]  Status: {}...", snippet.replace('\n', " ").trim());
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            if is_tool_not_found(&error_str) {
                crate::teeprintln!("    [SKIP]  SKIPPED: get_system_status not available");
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    [FAIL] get_system_status: {}", e);
                results.failed += 1;
                stats.failed += 1;
            }
        }
    }

    // Test 4: Test agent registration
    crate::teeprintln!("  Testing agent registration...");
    match client.call_tool("register_agent", serde_json::json!({
        "agent_type": "test_agent",
        "instance_id": "test_1",
        "capabilities": ["test_capability"]
    })).await {
        Ok(result) => {
            crate::teeprintln!("    [OK] register_agent SUCCESS (result keys: {})",
                result.as_object().map(|o| o.len()).unwrap_or(0));
            results.agents_tested += 1;
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            let error_str = e.to_string();
            if is_tool_not_found(&error_str) {
                crate::teeprintln!("    [SKIP]  SKIPPED: Agent registration not exposed");
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    [FAIL] register_agent: {}", e);
                results.failed += 1;
                stats.failed += 1;
            }
        }
    }

    // Test 5: Test agent unregistration
    crate::teeprintln!("  Testing agent unregistration...");
    match client.call_tool("unregister_agent", serde_json::json!({
        "agent_type": "test_agent",
        "instance_id": "test_1"
    })).await {
        Ok(result) => {
            crate::teeprintln!("    [OK] unregister_agent SUCCESS (result keys: {})",
                result.as_object().map(|o| o.len()).unwrap_or(0));
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            let error_str = e.to_string();
            if is_tool_not_found(&error_str) {
                crate::teeprintln!("    [SKIP]  SKIPPED: Agent unregistration not exposed");
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    [FAIL] unregister_agent: {}", e);
                results.failed += 1;
                stats.failed += 1;
            }
        }
    }

    Ok(results)
}
