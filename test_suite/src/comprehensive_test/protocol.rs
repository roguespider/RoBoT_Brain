//! MCP Protocol Testing Module
//!
//! Tests basic MCP protocol functionality.

use crate::TestMcpClient;
use crate::TestStats;

/// Test basic MCP protocol functionality
pub async fn test_mcp_basics(client: &mut TestMcpClient, stats: &mut TestStats) -> bool {
    let mut all_ok = true;
    
    // Test tools/list
    crate::teeprintln!("  Testing tools/list...");
    match client.list_tools().await {
        Ok(tools) => {
            crate::teeprintln!("    ✓ tools/list - SUCCESS ({} tools)", tools.len());
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("    ✗ tools/list - FAILED: {}", e);
            stats.failed += 1;
            all_ok = false;
        }
    }
    
    // Test tools/call (this will likely fail)
    crate::teeprintln!("  Testing tools/call...");
    match client.call_tool("get_workflow", serde_json::json!({"purpose": "test"})).await {
        Ok(_) => {
            crate::teeprintln!("    ✓ tools/call - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            let error_str = e.to_string();
            if error_str.contains("method_not_found") || error_str.contains("-32601") {
                crate::teeprintln!("    ✗ tools/call - NOT IMPLEMENTED");
                crate::teeprintln!("    ℹ  Server returns method_not_found for tools/call");
            } else {
                crate::teeprintln!("    ✗ tools/call - ERROR: {}", e);
            }
            stats.failed += 1;
            all_ok = false;
        }
    }
    
    all_ok
}
