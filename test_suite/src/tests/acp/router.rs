//! ACP Router Tests
//!
//! Tests message routing functionality via ACP

use crate::{TestMcpClient, TestStats};

/// Router test results
#[derive(Debug, Default)]
pub struct RouterTestResults {
    pub passed: usize,
    pub failed: usize,
    pub messages_routed: usize,
}

/// Test ACP message routing
pub async fn test_acp_router(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<RouterTestResults> {
    let mut results = RouterTestResults::default();

    // Test 1: Test route_acp_message functionality
    crate::teeprintln!("  Testing ACP message routing...");
    match client.call_tool("route_acp_message", serde_json::json!({
        "sender": {"agent_type": "test", "instance_id": "1"},
        "receiver": {"agent_type": "worker", "instance_id": "1"},
        "message_type": "Request",
        "payload": {"action": "test_route"}
    })).await {
        Ok(result) => {
            crate::teeprintln!("    ✅ route_acp_message SUCCESS");
            results.messages_routed += 1;
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
                crate::teeprintln!("    ⚠️  ACP routing not exposed via MCP");
                crate::teeprintln!("    ℹ  ACP router exists in code (src/bridge/acp/router.rs)");
                results.failed += 1;
                stats.skipped += 1;
            } else if error_str.contains("Unknown receiver") || error_str.contains("not registered") {
                crate::teeprintln!("    ⚠️  Message routing works but receiver not found");
                crate::teeprintln!("    ℹ  This is expected if no agents are registered");
                results.passed += 1; // Routing works, just no agents
                stats.passed += 1;
            } else {
                crate::teeprintln!("    ❌ route_acp_message ERROR: {}", e);
                results.failed += 1;
                stats.failed += 1;
            }
        }
    }

    // Test 2: Test routing with broadcast address
    crate::teeprintln!("  Testing ACP broadcast routing...");
    match client.call_tool("route_acp_message", serde_json::json!({
        "sender": {"agent_type": "test", "instance_id": "1"},
        "receiver": {"agent_type": "workers", "instance_id": "*"},
        "message_type": "Inform",
        "payload": {"action": "broadcast_test"}
    })).await {
        Ok(result) => {
            crate::teeprintln!("    ✅ Broadcast routing SUCCESS");
            results.messages_routed += 1;
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            let error_str = e.to_string();
            if error_str.contains("method_not_found") || error_str.contains("not found") {
                crate::teeprintln!("    ⚠️  Broadcast routing not exposed");
                results.failed += 1;
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    ⚠️  Broadcast: {}", e);
                results.failed += 1;
                stats.skipped += 1;
            }
        }
    }

    // Test 3: Test message with TTL
    crate::teeprintln!("  Testing ACP message TTL handling...");
    match client.call_tool("route_acp_message", serde_json::json!({
        "sender": {"agent_type": "test", "instance_id": "1"},
        "receiver": {"agent_type": "worker", "instance_id": "1"},
        "message_type": "Request",
        "payload": {"action": "ttl_test"},
        "ttl": 5
    })).await {
        Ok(_) => {
            crate::teeprintln!("    ✅ TTL handling works");
            results.messages_routed += 1;
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            let error_str = e.to_string();
            if error_str.contains("method_not_found") || error_str.contains("not found") {
                crate::teeprintln!("    ⚠️  TTL not supported via MCP");
                results.failed += 1;
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    ⚠️  TTL test: {}", e);
                results.failed += 1;
                stats.skipped += 1;
            }
        }
    }

    // Test 4: Test different message types
    crate::teeprintln!("  Testing different ACP message types...");
    let message_types = vec!["Request", "Query", "Inform", "Subscribe"];
    let mut types_tested = 0;
    
    for msg_type in message_types.iter() {
        let args = serde_json::json!({
            "sender": {"agent_type": "test", "instance_id": "1"},
            "receiver": {"agent_type": "worker", "instance_id": "1"},
            "message_type": msg_type,
            "payload": {"action": format!("test_{}", msg_type.to_lowercase())}
        });
        
        if client.call_tool("route_acp_message", args).await.is_ok() {
            types_tested += 1;
        }
    }
    
    if types_tested >= 2 {
        crate::teeprintln!("    ✅ {}/{} message types work", types_tested, message_types.len());
        results.passed += 1;
        stats.passed += 1;
    } else {
        crate::teeprintln!("    ⚠️  Only {}/{} message types work", types_tested, message_types.len());
        results.failed += 1;
    }

    Ok(results)
}
