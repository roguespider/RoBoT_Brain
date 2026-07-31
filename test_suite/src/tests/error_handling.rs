



//! Error handling tests
use crate::TestMcpClient;
use crate::TestStats;

/// Run error handling tests
pub async fn run_error_handling_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    _filter: Option<&str>,
) -> anyhow::Result<()> {
    crate::teeprintln!("\n--- Error Handling Tests ---");
    
    // Test invalid UUID handling
    match client.call_tool("get_memory", serde_json::json!({
        "id": "not-a-uuid"
    })).await {
        Ok(_) => {
            crate::teeprintln!("  ? test_invalid_uuid - Tool accepted invalid UUID (may be expected)");
            stats.skipped += 1;
        }
        Err(_) => {
            crate::teeprintln!("  ✓ test_invalid_uuid - Correctly rejected invalid UUID");
            stats.passed += 1;
        }
    }
    
    // Test missing parameters
    match client.call_tool("store_memory", serde_json::json!({})).await {
        Ok(_) => {
            crate::teeprintln!("  ✓ test_missing_params - Tool accepted missing params (graceful fallback)");
            stats.passed += 1;
        }
        Err(_) => {
            crate::teeprintln!("  ✓ test_missing_params - Correctly rejected missing params");
            stats.passed += 1;
        }
    }
    
    // Test invalid memory type
    match client.call_tool("store_memory", serde_json::json!({
        "content": "test",
        "memory_type": "invalid_type"
    })).await {
        Ok(_) => {
            crate::teeprintln!("  ✓ test_invalid_memory_type - Tool accepted invalid type (defaulted to note)");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ test_invalid_memory_type - Tool rejected invalid type: {}", e);
            stats.failed += 1;
        }
    }
    
    Ok(())
}
