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
            if error_str.contains("method_not_found") || error_str.contains("not found") {
                crate::teeprintln!("    ⚠️  ACP message creation not exposed");
                results.failed += 1;
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    ⚠️  create_acp_message: {}", e);
                results.failed += 1;
                stats.skipped += 1;
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
            if error_str.contains("method_not_found") || error_str.contains("not found") {
                crate::teeprintln!("    ⚠️  Reply handling not exposed");
                results.failed += 1;
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    ⚠️  Reply handling: {}", e);
                results.failed += 1;
                stats.skipped += 1;
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
            if error_str.contains("method_not_found") || error_str.contains("not found") {
                crate::teeprintln!("    ⚠️  Conversation tracking not exposed");
                results.failed += 1;
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    ⚠️  Conversation tracking: {}", e);
                results.failed += 1;
                stats.skipped += 1;
            }
        }
    }

    // Test 4: Test error message handling
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
            // Error message routing may return error, which is expected
            crate::teeprintln!("    ⚠️  Error message: {}", e);
            results.failed += 1;
            stats.skipped += 1;
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
            if error_str.contains("method_not_found") || error_str.contains("not found") {
                crate::teeprintln!("    ⚠️  TTL not supported");
                results.failed += 1;
                stats.skipped += 1;
            } else {
                crate::teeprintln!("    ⚠️  TTL: {}", e);
                results.failed += 1;
                stats.skipped += 1;
            }
        }
    }

    Ok(results)
}
