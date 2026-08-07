//! ACP (Agent Communication Protocol) Tests
//!
//! Tests the inter-agent communication protocol including:
//! - Agent registration and discovery
//! - Message routing
//! - Agent capabilities
//! - Message types and conversations

pub mod registry;
pub mod router;
pub mod agents;
pub mod messages;

pub use registry::test_acp_registry;
pub use router::test_acp_router;
pub use agents::test_acp_agents;
pub use messages::test_acp_messages;

use crate::{TestMcpClient, TestStats};

/// Run all ACP tests
pub async fn run_acp_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    _filter: Option<&str>,
) -> anyhow::Result<AcpTestResults> {
    crate::teeprintln!("\n{}", "=".repeat(80));
    crate::teeprintln!("ACP (AGENT COMMUNICATION PROTOCOL) TESTS");
    crate::teeprintln!("Testing inter-agent communication, routing, and registration");
    crate::teeprintln!("{}", "=".repeat(80));

    // Phase 1: Registry tests
    crate::teeprintln!("\n📋 PHASE 1: AGENT REGISTRY");
    crate::teeprintln!("{}", "-".repeat(60));
    let registry_results = registry::test_acp_registry(client, stats).await?;

    // Phase 2: Router tests
    crate::teeprintln!("\n📋 PHASE 2: MESSAGE ROUTING");
    crate::teeprintln!("{}", "-".repeat(60));
    let router_results = router::test_acp_router(client, stats).await?;

    // Phase 3: Agent tests
    crate::teeprintln!("\n📋 PHASE 3: AGENT CAPABILITIES");
    crate::teeprintln!("{}", "-".repeat(60));
    let agent_results = agents::test_acp_agents(client, stats).await?;

    // Phase 4: Message tests
    crate::teeprintln!("\n📋 PHASE 4: MESSAGE HANDLING");
    crate::teeprintln!("{}", "-".repeat(60));
    let message_results = messages::test_acp_messages(client, stats).await?;

    let results = AcpTestResults {
        registry: registry_results,
        router: router_results,
        agents: agent_results,
        messages: message_results,
    };

    print_acp_results(&results);
    Ok(results)
}

/// ACP test results
#[derive(Debug, Default)]
pub struct AcpTestResults {
    pub registry: registry::RegistryTestResults,
    pub router: router::RouterTestResults,
    pub agents: agents::AgentTestResults,
    pub messages: messages::MessageTestResults,
}

impl AcpTestResults {
    pub fn total_passed(&self) -> usize {
        self.registry.passed + self.router.passed + 
        self.agents.passed + self.messages.passed
    }

    pub fn total_failed(&self) -> usize {
        self.registry.failed + self.router.failed + 
        self.agents.failed + self.messages.failed
    }
}

fn print_acp_results(results: &AcpTestResults) {
    crate::teeprintln!("\n{}", "=".repeat(80));
    crate::teeprintln!("ACP TEST RESULTS SUMMARY");
    crate::teeprintln!("{}", "=".repeat(80));

    // Registry results
    crate::teeprintln!("\n📝 Agent Registry:");
    crate::teeprintln!("  ✅ Passed: {}", results.registry.passed);
    crate::teeprintln!("  ❌ Failed: {}", results.registry.failed);
    crate::teeprintln!("  ℹ  Registered Agents: {}", results.registry.agents_registered);

    // Router results
    crate::teeprintln!("\n🔀 Message Router:");
    crate::teeprintln!("  ✅ Passed: {}", results.router.passed);
    crate::teeprintln!("  ❌ Failed: {}", results.router.failed);
    crate::teeprintln!("  ℹ  Messages Routed: {}", results.router.messages_routed);

    // Agent results
    crate::teeprintln!("\n🤖 Agent Capabilities:");
    crate::teeprintln!("  ✅ Passed: {}", results.agents.passed);
    crate::teeprintln!("  ❌ Failed: {}", results.agents.failed);
    crate::teeprintln!("  ℹ  Agents Tested: {}", results.agents.agents_tested);

    // Message results
    crate::teeprintln!("\n💬 Message Handling:");
    crate::teeprintln!("  ✅ Passed: {}", results.messages.passed);
    crate::teeprintln!("  ❌ Failed: {}", results.messages.failed);
    crate::teeprintln!("  ℹ  Messages Handled: {}", results.messages.messages_handled);

    crate::teeprintln!("\n📊 Overall:");
    crate::teeprintln!("  Total Passed: {}", results.total_passed());
    crate::teeprintln!("  Total Failed: {}", results.total_failed());
}
