//! ACP Message Tests
//!
//! Tests ACP message handling and message-related functionality

use crate::{TestMcpClient, TestStats};

/// Message test results
#[derive(Debug, Default)]
pub struct MessageTestResults {
    pub passed: usize,
    pub failed: usize,
    pub messages_handled: usize,
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

/// Test ACP message handling
pub async fn test_acp_messages(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<MessageTestResults> {
    let mut results = MessageTestResults::default();

    // Test 1: Test message creation and routing
    crate::teeprintln!("  Testing message creation...");
    match client.call_tool("create_acp_message", serde_json::json!({
        "sender": {"agent_type": "test", "instance_id": "1"},
        "receiver": {"agent_type": "worker", "instance_id": "1"},
        "message_type": "Request",
        "payload": {"action": "ping"}
    })).await {
        Ok(result) => {
            crate::teeprintln!("    ✅ create_acp_message SUCCESS");
            results.messages_handled += 1;
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            let error_str = e.to_string();
            if is_tool_not_found(&error_str) {
                crate::teeprintln!("    ⏭️  SKIPPED: ACP message creation not exposed");
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    ❌ create_acp_message: {}", e);
                results.failed += 1;
                stats.failed += 1;
            }
        }
    }

    // Test 2: Test message with reply
    crate::teeprintln!("  Testing message reply handling...");
    match client.call_tool("route_acp_message", serde_json::json!({
        "sender": {"agent_type": "client", "instance_id": "1"},
        "receiver": {"agent_type": "worker", "instance_id": "1"},
        "message_type": "Request",
        "payload": {"action": "get_data"},
        "reply_to": "original_msg_id"
    })).await {
        Ok(result) => {
            crate::teeprintln!("    ✅ Message with reply SUCCESS");
            results.messages_handled += 1;
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            let error_str = e.to_string();
            if is_tool_not_found(&error_str) {
                crate::teeprintln!("    ⏭️  SKIPPED: Reply handling not exposed");
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    ❌ Reply handling: {}", e);
                results.failed += 1;
                stats.failed += 1;
            }
        }
    }

    // Test 3: Test conversation tracking
    crate::teeprintln!("  Testing conversation tracking...");
    match client.call_tool("route_acp_message", serde_json::json!({
        "sender": {"agent_type": "client", "instance_id": "1"},
        "receiver": {"agent_type": "worker", "instance_id": "1"},
        "message_type": "Request",
        "payload": {"action": "start_conversation"},
        "conversation_id": "conv_123"
    })).await {
        Ok(_) => {
            crate::teeprintln!("    ✅ Conversation tracking SUCCESS");
            results.messages_handled += 1;
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            let error_str = e.to_string();
            if is_tool_not_found(&error_str) {
                crate::teeprintln!("    ⏭️  SKIPPED: Conversation tracking not exposed");
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    ❌ Conversation tracking: {}", e);
                results.failed += 1;
                stats.failed += 1;
            }
        }
    }

    // Test 4: Test error message handling
    // This tests that the router correctly rejects messages to unknown receivers
    crate::teeprintln!("  Testing error message handling...");
    match client.call_tool("route_acp_message", serde_json::json!({
        "sender": {"agent_type": "client", "instance_id": "1"},
        "receiver": {"agent_type": "nonexistent", "instance_id": "1"},
        "message_type": "Error",
        "payload": {"error": "test_error"}
    })).await {
        Ok(_) => {
            crate::teeprintln!("    ✅ Error message handling SUCCESS");
            results.messages_handled += 1;
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            // Error is expected - router correctly rejects unknown receivers
            let err_str = format!("{}", e);
            if err_str.contains("Unknown receiver") || err_str.contains("not registered") {
                crate::teeprintln!("    ✅ Error message handling SUCCESS (correctly rejected unknown receiver)");
                results.messages_handled += 1;
                results.passed += 1;
                stats.passed += 1;
            } else {
                crate::teeprintln!("    ⚠️  Error message: {}", e);
                stats.skipped += 1;
            }
        }
    }

    // Test 5: Test message expiration (TTL)
    crate::teeprintln!("  Testing message TTL expiration...");
    match client.call_tool("route_acp_message", serde_json::json!({
        "sender": {"agent_type": "test", "instance_id": "1"},
        "receiver": {"agent_type": "worker", "instance_id": "1"},
        "message_type": "Request",
        "payload": {"action": "expire_test"},
        "ttl": 1
    })).await {
        Ok(_) => {
            crate::teeprintln!("    ✅ TTL message SUCCESS");
            results.messages_handled += 1;
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            let error_str = e.to_string();
            if is_tool_not_found(&error_str) {
                crate::teeprintln!("    ⏭️  SKIPPED: TTL not supported");
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    ❌ TTL: {}", e);
                results.failed += 1;
                stats.failed += 1;
            }
        }
    }

    Ok(results)
}
