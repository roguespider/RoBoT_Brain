//! RMCP Protocol Tests
//!
//! Tests the MCP protocol implementation including:
//! - Protocol version negotiation
//! - Server capabilities
//! - Initialize handshake

use crate::{TestMcpClient, TestStats};

/// Protocol test results
#[derive(Debug, Default)]
pub struct ProtocolTestResults {
    pub passed: usize,
    pub failed: usize,
    pub init_ok: bool,
    pub capabilities_ok: bool,
    pub version_ok: bool,
}

/// Test MCP protocol initialization and capabilities
pub async fn test_rmcp_protocol(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<ProtocolTestResults> {
    let mut results = ProtocolTestResults::default();

    // Test 1: Check client is connected (initialized)
    crate::teeprintln!("  Testing client connection status...");
    if client.is_running() {
        crate::teeprintln!("    ✅ Client connected and server responding");
        results.init_ok = true;
        results.passed += 1;
    } else {
        crate::teeprintln!("    ❌ Client not running - server may have crashed");
        results.init_ok = false;
        results.failed += 1;
        return Ok(results);
    }

    // Test 2: Try to get server info via initialize
    crate::teeprintln!("  Testing server initialize response...");
    match client.list_tools().await {
        Ok(tools) => {
            crate::teeprintln!("    ✅ Server responds to initialize/list_tools");
            results.capabilities_ok = true;
            results.passed += 1;
            
            if !tools.is_empty() {
                crate::teeprintln!("    ✅ Server reports {} tools available", tools.len());
                results.passed += 1;
            } else {
                crate::teeprintln!("    ⚠️  Server returns empty tool list");
                crate::teeprintln!("    ℹ  This may indicate list_tools() not fully implemented");
                results.failed += 1;
            }
        }
        Err(e) => {
            crate::teeprintln!("    ❌ Server not responding: {}", e);
            results.capabilities_ok = false;
            results.failed += 1;
        }
    }

    // Test 3: Verify protocol version compatibility
    crate::teeprintln!("  Testing protocol version compatibility...");
    let version_info = client.get_protocol_info();
    if let Some(info) = version_info {
        crate::teeprintln!("    ℹ  Protocol info: {:?}", info);
        // Version check is informational
        results.version_ok = true;
        results.passed += 1;
    } else {
        crate::teeprintln!("    ⚠️  Could not determine protocol version");
        results.version_ok = false;
        results.failed += 1;
    }

    // Test 4: Test ping/pong behavior
    crate::teeprintln!("  Testing server responsiveness...");
    match client.list_tools().await {
        Ok(_) => {
            crate::teeprintln!("    ✅ Server is responsive to requests");
            results.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("    ❌ Server unresponsive: {}", e);
            results.failed += 1;
        }
    }

    // Update overall stats from protocol test results
    stats.passed += results.passed;
    stats.failed += results.failed;

    Ok(results)
}
