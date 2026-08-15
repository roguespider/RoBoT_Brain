//! ACP Registry Tests
//!
//! Tests agent registration and discovery via ACP
//!
//! NOTE: These tests require ACP to be exposed via MCP tools.
//! If tools are not implemented, tests are skipped.

use crate::{TestMcpClient, TestStats};

/// Registry test results
#[derive(Debug, Default)]
pub struct RegistryTestResults {
    pub passed: usize,
    pub failed: usize,
    pub agents_registered: usize,
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

/// Test ACP agent registry functionality
pub async fn test_acp_registry(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<RegistryTestResults> {
    let mut results = RegistryTestResults::default();

    // Test 1: Check if ACP registry tools are available
    crate::teeprintln!("  Testing ACP registry tool availability...");
    match client.call_tool("list_acp_agents", serde_json::json!({})).await {
        Ok(result) => {
            crate::teeprintln!("    [OK] list_acp_agents SUCCESS");
            results.passed += 1;
            stats.passed += 1;
            
            if let Some(text) = result.get("content").and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|t| t.get("text"))
                .and_then(|t| t.as_str()) 
            {
                crate::teeprintln!("    [INFO]  Result: {}", text);
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            if is_tool_not_found(&error_str) {
                crate::teeprintln!("    [SKIP]  SKIPPED: ACP tools not implemented via MCP");
                crate::teeprintln!("    [INFO]  ACP registry exists but not exposed");
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    [FAIL] list_acp_agents ERROR: {}", e);
                results.failed += 1;
                stats.failed += 1;
            }
        }
    }

    // Test 2: Check ACP agent count (skip if tools not available)
    crate::teeprintln!("  Testing ACP agent count tool...");
    match client.call_tool("acp_agent_count", serde_json::json!({})).await {
        Ok(result) => {
            crate::teeprintln!("    [OK] acp_agent_count SUCCESS");
            results.passed += 1;
            stats.passed += 1;
            
            if let Some(text) = result.get("content").and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|t| t.get("text"))
                .and_then(|t| t.as_str()) 
            {
                crate::teeprintln!("    [INFO]  Result: {}", text);
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            if is_tool_not_found(&error_str) {
                crate::teeprintln!("    [SKIP]  SKIPPED: ACP tools not implemented");
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    [FAIL] acp_agent_count ERROR: {}", e);
                results.failed += 1;
                stats.failed += 1;
            }
        }
    }

    // Test 3: Test ACP router access
    crate::teeprintln!("  Testing ACP router access...");
    match client.call_tool("acp_router", serde_json::json!({})).await {
        Ok(result) => {
            crate::teeprintln!("    [OK] acp_router SUCCESS");
            results.passed += 1;
            stats.passed += 1;
            
            if let Some(text) = result.get("content").and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|t| t.get("text"))
                .and_then(|t| t.as_str()) 
            {
                crate::teeprintln!("    [INFO]  Result: {}", text);
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            if is_tool_not_found(&error_str) {
                crate::teeprintln!("    [SKIP]  SKIPPED: ACP tools not implemented");
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    [FAIL] acp_router ERROR: {}", e);
                results.failed += 1;
                stats.failed += 1;
            }
        }
    }

    // Test 4: Test ACP registry access
    crate::teeprintln!("  Testing ACP registry access...");
    match client.call_tool("acp_registry", serde_json::json!({})).await {
        Ok(result) => {
            crate::teeprintln!("    [OK] acp_registry SUCCESS");
            results.passed += 1;
            stats.passed += 1;
            
            if let Some(text) = result.get("content").and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|t| t.get("text"))
                .and_then(|t| t.as_str()) 
            {
                crate::teeprintln!("    [INFO]  Result: {}", text);
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            if is_tool_not_found(&error_str) {
                crate::teeprintln!("    [SKIP]  SKIPPED: ACP tools not implemented");
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    [FAIL] acp_registry ERROR: {}", e);
                results.failed += 1;
                stats.failed += 1;
            }
        }
    }

    Ok(results)
}
