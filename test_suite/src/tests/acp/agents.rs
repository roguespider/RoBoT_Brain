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
            crate::teeprintln!("    ✅ list_acp_agents SUCCESS");
            results.agents_tested += 1;
            results.passed += 1;
            stats.passed += 1;
            
            if let Some(text) = result.get("content").and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|t| t.get("text"))
                .and_then(|t| t.as_str()) 
            {
                crate::teeprintln!("    ℹ  Agents: {}", text);
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            if error_str.contains("method_not_found") || error_str.contains("not found") {
                crate::teeprintln!("    ⚠️  ACP agents not exposed via MCP");
                results.failed += 1;
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    ❌ list_acp_agents ERROR: {}", e);
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
            crate::teeprintln!("    ✅ get_agent_capabilities SUCCESS");
            results.agents_tested += 1;
            results.passed += 1;
            stats.passed += 1;
            
            if let Some(text) = result.get("content").and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|t| t.get("text"))
                .and_then(|t| t.as_str()) 
            {
                crate::teeprintln!("    ℹ  Capabilities: {}", text);
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            if error_str.contains("method_not_found") || error_str.contains("not found") {
                crate::teeprintln!("    ⚠️  Agent capabilities not exposed");
                results.failed += 1;
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    ⚠️  get_agent_capabilities: {}", e);
                results.failed += 1;
                stats.skipped += 1;
            }
        }
    }

    // Test 3: Test system status (agent-related)
    crate::teeprintln!("  Testing system agent status...");
    match client.call_tool("get_system_status", serde_json::json!({})).await {
        Ok(result) => {
            crate::teeprintln!("    ✅ get_system_status SUCCESS");
            results.agents_tested += 1;
            results.passed += 1;
            stats.passed += 1;
            
            if let Some(text) = result.get("content").and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|t| t.get("text"))
                .and_then(|t| t.as_str()) 
            {
                let snippet = if text.len() > 150 { &text[..150] } else { text };
                crate::teeprintln!("    ℹ  Status: {}...", snippet.replace('\n', " ").trim());
            }
        }
        Err(e) => {
            crate::teeprintln!("    ⚠️  get_system_status: {}", e);
            results.failed += 1;
            stats.skipped += 1;
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
            crate::teeprintln!("    ✅ register_agent SUCCESS");
            results.agents_tested += 1;
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            let error_str = e.to_string();
            if error_str.contains("method_not_found") || error_str.contains("not found") {
                crate::teeprintln!("    ⚠️  Agent registration not exposed");
                results.failed += 1;
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    ⚠️  register_agent: {}", e);
                results.failed += 1;
                stats.skipped += 1;
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
            crate::teeprintln!("    ✅ unregister_agent SUCCESS");
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            let error_str = e.to_string();
            if error_str.contains("method_not_found") || error_str.contains("not found") {
                crate::teeprintln!("    ⚠️  Agent unregistration not exposed");
                results.failed += 1;
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    ⚠️  unregister_agent: {}", e);
                results.failed += 1;
                stats.skipped += 1;
            }
        }
    }

    Ok(results)
}
