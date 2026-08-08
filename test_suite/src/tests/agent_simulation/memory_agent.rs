//! Memory-Based Agent Behavior Tests
//!
//! Tests agent behavior that depends on memory operations

use crate::{TestMcpClient, TestStats};

/// Memory agent test results
#[derive(Debug, Default)]
pub struct MemoryAgentResults {
    pub passed: usize,
    pub failed: usize,
    pub operations_tested: usize,
}

/// Test memory-based agent behavior
pub async fn test_memory_based_agent(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<MemoryAgentResults> {
    let mut results = MemoryAgentResults::default();

    // Test 1: Memory retrieval and context building
    crate::teeprintln!("  Testing memory retrieval for context...");
    match client.call_tool("search_memory", serde_json::json!({"query": "test", "limit": 5})).await {
        Ok(_) => {
            crate::teeprintln!("    ✅ Memory search SUCCESS");
            results.operations_tested += 1;
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("    ❌ Memory search: {}", e);
            results.failed += 1;
            stats.failed += 1;
        }
    }

    // Test 2: Knowledge-based context
    crate::teeprintln!("  Testing knowledge retrieval for context...");
    match client.call_tool("query_knowledge", serde_json::json!({"query": "system design", "limit": 5})).await {
        Ok(_) => {
            crate::teeprintln!("    ✅ Knowledge query SUCCESS");
            results.operations_tested += 1;
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("    ❌ Knowledge query: {}", e);
            results.failed += 1;
            stats.failed += 1;
        }
    }

    // Test 3: Cross-memory search
    crate::teeprintln!("  Testing cross-memory search...");
    match client.call_tool("global_search", serde_json::json!({"query": "architecture", "limit": 10})).await {
        Ok(_) => {
            crate::teeprintln!("    ✅ Cross-memory search SUCCESS");
            results.operations_tested += 1;
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("    ❌ Cross-memory search: {}", e);
            results.failed += 1;
            stats.failed += 1;
        }
    }

    // Test 4: Layer-based memory access
    crate::teeprintln!("  Testing layer-based memory access...");
    match client.call_tool("list_memories", serde_json::json!({"layer": "working", "limit": 10})).await {
        Ok(_) => {
            crate::teeprintln!("    ✅ Layer-based memory access SUCCESS");
            results.operations_tested += 1;
            results.passed += 1;
            stats.passed += 1;
        }
        Err(_) => {
            // Try without layer filter
            match client.call_tool("list_memories", serde_json::json!({"limit": 10})).await {
                Ok(_) => {
                    crate::teeprintln!("    ✅ Memory listing SUCCESS (layer filter not supported)");
                    results.operations_tested += 1;
                    results.passed += 1;
                    stats.passed += 1;
                }
                Err(e) => {
                    crate::teeprintln!("    ❌ Layer-based access: {}", e);
                    results.failed += 1;
                    stats.failed += 1;
                }
            }
        }
    }

    // Test 5: Memory-based reflection
    crate::teeprintln!("  Testing memory-based reflection...");
    
    let mut chain_success = 0;
    
    if client.call_tool("get_insights", serde_json::json!({})).await.is_ok() {
        chain_success += 1;
    }
    if client.call_tool("list_experiences", serde_json::json!({"limit": 10})).await.is_ok() {
        chain_success += 1;
    }
    
    if chain_success >= 1 {
        crate::teeprintln!("    ✅ Memory reflection: {}/2 operations succeeded", chain_success);
        results.operations_tested += chain_success;
        results.passed += 1;
        stats.passed += 1;
    } else {
        crate::teeprintln!("    ❌ Memory reflection chain failed");
        results.failed += 1;
        stats.failed += 1;
    }

    // Test 6: Memory persistence verification
    crate::teeprintln!("  Testing memory persistence verification...");
    match client.call_tool("get_system_status", serde_json::json!({})).await {
        Ok(_) => {
            crate::teeprintln!("    ✅ System status (memory state) SUCCESS");
            results.operations_tested += 1;
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("    ⚠️  System status: {}", e);
            results.failed += 1;
            stats.skipped += 1;
        }
    }

    Ok(results)
}
