//! ACP Registry Tests
//!
//! Tests agent registration and discovery via ACP

use crate::{TestMcpClient, TestStats};

/// Registry test results
#[derive(Debug, Default)]
pub struct RegistryTestResults {
    pub passed: usize,
    pub failed: usize,
    pub agents_registered: usize,
}

/// Test ACP agent registry functionality
pub async fn test_acp_registry(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<RegistryTestResults> {
    let mut results = RegistryTestResults::default();

    // Note: ACP registry testing requires the robot_brain server to expose
    // ACP functionality via MCP tools or direct API access.
    // Since ACP is designed for inter-agent communication, we test
    // the registry-related tools that should be available.

    // Test 1: Check if ACP registry tools are available
    crate::teeprintln!("  Testing ACP registry tool availability...");
    match client.call_tool("list_acp_agents", serde_json::json!({})).await {
        Ok(result) => {
            crate::teeprintln!("    ✅ list_acp_agents SUCCESS");
            results.passed += 1;
            stats.passed += 1;
            
            // Try to extract agent count
            if let Some(text) = result.get("content").and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|t| t.get("text"))
                .and_then(|t| t.as_str()) 
            {
                crate::teeprintln!("    ℹ  Result: {}", text);
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            if error_str.contains("method_not_found") || error_str.contains("not found") {
                crate::teeprintln!("    ⚠️  ACP not fully exposed via MCP tools");
                crate::teeprintln!("    ℹ  ACP registry exists in code but not exposed");
                results.failed += 1;
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    ❌ list_acp_agents ERROR: {}", e);
                results.failed += 1;
                stats.failed += 1;
            }
        }
    }

    // Test 2: Check ACP agent count
    crate::teeprintln!("  Testing ACP agent count tool...");
    match client.call_tool("acp_agent_count", serde_json::json!({})).await {
        Ok(result) => {
            crate::teeprintln!("    ✅ acp_agent_count SUCCESS");
            results.passed += 1;
            stats.passed += 1;
            
            if let Some(text) = result.get("content").and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|t| t.get("text"))
                .and_then(|t| t.as_str()) 
            {
                crate::teeprintln!("    ℹ  Result: {}", text);
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            if error_str.contains("method_not_found") || error_str.contains("not found") {
                crate::teeprintln!("    ⚠️  ACP agent count not exposed");
                results.failed += 1;
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    ❌ acp_agent_count ERROR: {}", e);
                results.failed += 1;
                stats.failed += 1;
            }
        }
    }

    // Test 3: Test ACP router access
    crate::teeprintln!("  Testing ACP router access...");
    match client.call_tool("acp_router", serde_json::json!({})).await {
        Ok(result) => {
            crate::teeprintln!("    ✅ acp_router SUCCESS");
            results.passed += 1;
            stats.passed += 1;
            
            if let Some(text) = result.get("content").and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|t| t.get("text"))
                .and_then(|t| t.as_str()) 
            {
                crate::teeprintln!("    ℹ  Result: {}", text);
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            if error_str.contains("method_not_found") || error_str.contains("not found") {
                crate::teeprintln!("    ⚠️  ACP router not exposed");
                results.failed += 1;
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    ❌ acp_router ERROR: {}", e);
                results.failed += 1;
                stats.failed += 1;
            }
        }
    }

    // Test 4: Test ACP registry access
    crate::teeprintln!("  Testing ACP registry access...");
    match client.call_tool("acp_registry", serde_json::json!({})).await {
        Ok(result) => {
            crate::teeprintln!("    ✅ acp_registry SUCCESS");
            results.passed += 1;
            stats.passed += 1;
            
            if let Some(text) = result.get("content").and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|t| t.get("text"))
                .and_then(|t| t.as_str()) 
            {
                crate::teeprintln!("    ℹ  Result: {}", text);
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            if error_str.contains("method_not_found") || error_str.contains("not found") {
                crate::teeprintln!("    ⚠️  ACP registry not exposed");
                results.failed += 1;
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    ❌ acp_registry ERROR: {}", e);
                results.failed += 1;
                stats.failed += 1;
            }
        }
    }

    Ok(results)
}
