//! RMCP Session Tests
//!
//! Tests session management via MCP protocol

use crate::{TestMcpClient, TestStats};

/// Session test results
#[derive(Debug, Default)]
pub struct SessionTestResults {
    pub passed: usize,
    pub failed: usize,
    pub sessions_tracked: usize,
}

/// Test session management
pub async fn test_rmcp_sessions(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<SessionTestResults> {
    let mut results = SessionTestResults::default();

    // Test 1: Verify client maintains connection
    crate::teeprintln!("  Testing connection persistence...");
    if client.is_running() {
        crate::teeprintln!("    ✅ Connection maintained");
        results.passed += 1;
    } else {
        crate::teeprintln!("    ❌ Connection lost");
        results.failed += 1;
        return Ok(results);
    }

    // Test 2: Test multiple sequential requests (session tracking)
    crate::teeprintln!("  Testing sequential request handling...");
    let mut request_count = 0;
    let tools_to_test = vec![
        ("get_workflow", serde_json::json!({"purpose": "session_test_1"})),
        ("get_workflow", serde_json::json!({"purpose": "session_test_2"})),
        ("list_workflows", serde_json::json!({})),
        ("list_memories", serde_json::json!({"limit": 5})),
    ];

    for (tool, args) in tools_to_test {
        match client.call_tool(tool, args).await {
            Ok(_) => {
                request_count += 1;
            }
            Err(_) => {
                // Some tools may not be implemented, continue testing
            }
        }
    }
    
    if request_count >= 2 {
        crate::teeprintln!("    ✅ {} sequential requests processed", request_count);
        results.sessions_tracked = request_count;
        results.passed += 1;
    } else {
        crate::teeprintln!("    ⚠️  Only {} requests succeeded", request_count);
        results.sessions_tracked = request_count;
        results.failed += 1;
    }

    // Test 3: Test concurrent-ish behavior (rapid requests)
    crate::teeprintln!("  Testing rapid request handling...");
    let mut rapid_success = 0;
    for i in 0..3 {
        let args = serde_json::json!({"purpose": format!("rapid_test_{}", i)});
        if client.call_tool("get_workflow", args).await.is_ok() {
            rapid_success += 1;
        }
    }
    
    if rapid_success >= 2 {
        crate::teeprintln!("    ✅ {}/3 rapid requests succeeded", rapid_success);
        results.passed += 1;
    } else {
        crate::teeprintln!("    ⚠️  Only {}/3 rapid requests succeeded", rapid_success);
        results.failed += 1;
    }

    // Test 4: Verify server still responsive after tests
    crate::teeprintln!("  Testing server responsiveness after session...");
    match client.list_tools().await {
        Ok(tools) => {
            crate::teeprintln!("    ✅ Server still responsive - {} tools available", tools.len());
            results.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("    ❌ Server no longer responsive: {}", e);
            results.failed += 1;
        }
    }

    // Test 5: Test PID tracking
    crate::teeprintln!("  Testing process tracking...");
    if let Some(pid) = client.pid() {
        crate::teeprintln!("    ℹ  Server PID: {}", pid);
        results.passed += 1;
    } else {
        crate::teeprintln!("    ⚠️  Could not get server PID");
        results.failed += 1;
    }

    // Update overall stats from session test results
    stats.passed += results.passed;
    stats.failed += results.failed;

    Ok(results)
}
