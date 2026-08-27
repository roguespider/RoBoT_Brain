//! RMCP (Remote MCP) Protocol Tests
//!
//! Tests the MCP protocol implementation including:
//! - Protocol initialization and handshake
//! - Tool discovery (list_tools)
//! - Tool execution (call_tool)
//! - Session management
//! - Error handling

pub mod protocol;
pub mod sessions;
pub mod tools;

use crate::{TestMcpClient, TestStats};

/// Run all RMCP protocol tests
pub async fn run_rmcp_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    _filter: Option<&str>,
) -> anyhow::Result<RmcpTestResults> {
    crate::teeprintln!("\n{}", "=".repeat(80));
    crate::teeprintln!("RMCP PROTOCOL TESTS");
    crate::teeprintln!("Testing MCP stdio protocol, tool discovery, and execution");
    crate::teeprintln!("{}", "=".repeat(80));

    // Phase 1: Protocol tests
    crate::teeprintln!("\n[INFO] PHASE 1: PROTOCOL INITIALIZATION");
    crate::teeprintln!("{}", "-".repeat(60));
    let protocol_results = protocol::test_rmcp_protocol(client, stats).await?;

    // Phase 2: Tool discovery tests
    crate::teeprintln!("\n[INFO] PHASE 2: TOOL DISCOVERY (list_tools)");
    crate::teeprintln!("{}", "-".repeat(60));
    let tool_discovery = tools::test_tool_discovery(client, stats).await?;

    // Phase 3: Tool execution tests
    crate::teeprintln!("\n[INFO] PHASE 3: TOOL EXECUTION (call_tool)");
    crate::teeprintln!("{}", "-".repeat(60));
    let tool_execution = tools::test_tool_execution(client, stats).await?;

    // Phase 4: Session tests
    crate::teeprintln!("\n[INFO] PHASE 4: SESSION MANAGEMENT");
    crate::teeprintln!("{}", "-".repeat(60));
    let session_results = sessions::test_rmcp_sessions(client, stats).await?;

    let results = RmcpTestResults {
        protocol: protocol_results,
        tool_discovery,
        tool_execution,
        sessions: session_results,
    };

    print_rmcp_results(&results);
    Ok(results)
}

/// Test results for RMCP tests
#[derive(Debug, Default)]
pub struct RmcpTestResults {
    pub protocol: protocol::ProtocolTestResults,
    pub tool_discovery: tools::ToolDiscoveryResults,
    pub tool_execution: tools::ToolExecutionResults,
    pub sessions: sessions::SessionTestResults,
}

impl RmcpTestResults {
    pub fn total_passed(&self) -> usize {
        self.protocol.passed
            + self.tool_discovery.passed
            + self.tool_execution.passed
            + self.sessions.passed
    }

    pub fn total_failed(&self) -> usize {
        self.protocol.failed
            + self.tool_discovery.failed
            + self.tool_execution.failed
            + self.sessions.failed
    }
}

fn print_rmcp_results(results: &RmcpTestResults) {
    crate::teeprintln!("\n{}", "=".repeat(80));
    crate::teeprintln!("RMCP TEST RESULTS SUMMARY");
    crate::teeprintln!("{}", "=".repeat(80));

    // Status markers are conditional so a zero count never prints a [FAIL]
    // or [WARN] label (which would train the reader to ignore real failures).
    let marker = |n: usize| if n == 0 { "[OK]" } else { "[FAIL]" };

    // Protocol results
    crate::teeprintln!("\n[INFO] Protocol Tests:");
    crate::teeprintln!("  [OK] Passed: {}", results.protocol.passed);
    crate::teeprintln!(
        "  {} Failed: {}",
        marker(results.protocol.failed),
        results.protocol.failed
    );
    crate::teeprintln!(
        "  [INFO]  Initialization: {}",
        if results.protocol.init_ok {
            "OK"
        } else {
            "FAILED"
        }
    );
    crate::teeprintln!(
        "  [INFO]  Capabilities: {}",
        if results.protocol.capabilities_ok {
            "OK"
        } else {
            "FAILED"
        }
    );

    // Tool discovery results
    crate::teeprintln!("\n[INFO] Tool Discovery:");
    crate::teeprintln!("  [OK] Passed: {}", results.tool_discovery.passed);
    crate::teeprintln!(
        "  {} Failed: {}",
        marker(results.tool_discovery.failed),
        results.tool_discovery.failed
    );
    crate::teeprintln!(
        "  [INFO]  Tools Found: {}",
        results.tool_discovery.tools_found
    );
    crate::teeprintln!(
        "  [INFO]  Categories Covered: {:?}",
        results.tool_discovery.categories_found
    );

    // Tool execution results
    crate::teeprintln!("\n[INFO] Tool Execution:");
    crate::teeprintln!("  [OK] Passed: {}", results.tool_execution.passed);
    crate::teeprintln!(
        "  {} Failed: {}",
        marker(results.tool_execution.failed),
        results.tool_execution.failed
    );
    crate::teeprintln!(
        "  [INFO]  Tools Executed: {}",
        results.tool_execution.tools_executed
    );
    crate::teeprintln!(
        "  [INFO]  Categories Executed: {:?}",
        results.tool_execution.categories_executed
    );

    // Session results
    crate::teeprintln!("\n[INFO] Session Management:");
    crate::teeprintln!("  [OK] Passed: {}", results.sessions.passed);
    crate::teeprintln!(
        "  {} Failed: {}",
        marker(results.sessions.failed),
        results.sessions.failed
    );
    crate::teeprintln!(
        "  [INFO]  Sessions Tracked: {}",
        results.sessions.sessions_tracked
    );

    let total_failed = results.total_failed();
    crate::teeprintln!(
        "\n[{}] Overall:",
        if total_failed == 0 { "OK" } else { "FAIL" }
    );
    crate::teeprintln!("  Total Passed: {}", results.total_passed());
    crate::teeprintln!("  Total Failed: {}", total_failed);
}
