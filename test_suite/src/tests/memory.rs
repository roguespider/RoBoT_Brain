



//! Memory tool tests
use crate::TestMcpClient;
use crate::TestStats;

/// Run memory tool tests
pub async fn run_memory_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    _filter: Option<&str>,
) -> anyhow::Result<()> {
    crate::teeprintln!("\n--- Memory Tools Tests ---");
    
    // Search memory first (required by workflow enforcement)
    test_search_memory(client, stats, "test").await?;
    
    let memory_id = test_store_memory_with_id(client, stats, "note", "Test note content", Some(0.9), Some(0.8)).await?;
    test_store_memory(client, stats, "fact", "Important fact", None, None).await?;
    test_store_memory(client, stats, "task", "Task to complete", Some(0.7), Some(0.9)).await?;
    test_store_memory(client, stats, "code", "fn main() {}", Some(0.8), None).await?;
    test_store_memory(client, stats, "decision", "Chose option A", None, None).await?;
    test_store_memory(client, stats, "event", "User clicked button", None, None).await?;
    
    test_search_memory(client, stats, "test").await?;
    test_search_memory(client, stats, "important").await?;
    test_search_memory(client, stats, "task").await?;
    
    // Test with valid ID from previous store
    if let Some(id) = &memory_id {
        test_get_memory(client, stats, id).await?;
    }
    // Also test with a non-existent but valid UUID
    test_get_memory(client, stats, "00000000-0000-0000-0000-000000000000").await?;
    
    test_list_memories(client, stats, None).await?;
    test_list_memories(client, stats, Some("note")).await?;
    test_list_memories(client, stats, Some("fact")).await?;
    
    Ok(())
}

async fn test_store_memory(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    memory_type: &str,
    content: &str,
    confidence: Option<f32>,
    importance: Option<f32>,
) -> anyhow::Result<Option<String>> {
    let mut args = serde_json::json!({
        "content": content,
        "memory_type": memory_type
    });
    
    if let Some(c) = confidence {
        args["confidence"] = serde_json::json!(c);
    }
    if let Some(i) = importance {
        args["importance"] = serde_json::json!(i);
    }
    
    match client.call_tool("store_memory", args).await {
        Ok(result) => {
            crate::teeprintln!("  [OK] store_memory({}) - SUCCESS", memory_type);
            stats.passed += 1;
            // Extract the ID from the result
            let id = result.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
            Ok(id)
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] store_memory({}) - FAILED: {}", memory_type, e);
            stats.failed += 1;
            Ok(None)
        }
    }
}

// Alias for backwards compatibility
async fn test_store_memory_with_id(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    memory_type: &str,
    content: &str,
    confidence: Option<f32>,
    importance: Option<f32>,
) -> anyhow::Result<Option<String>> {
    test_store_memory(client, stats, memory_type, content, confidence, importance).await
}

async fn test_search_memory(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    query: &str,
) -> anyhow::Result<()> {
    match client.call_tool("search_memory", serde_json::json!({
        "query": query
    })).await {
        Ok(_) => {
            crate::teeprintln!("  [OK] search_memory('{}') - SUCCESS", query);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] search_memory('{}') - FAILED: {}", query, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_get_memory(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    id: &str,
) -> anyhow::Result<()> {
    match client.call_tool("get_memory", serde_json::json!({
        "id": id
    })).await {
        Ok(_result) => {
            crate::teeprintln!("  [OK] get_memory({}) - SUCCESS", id.chars().take(8).collect::<String>());
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] get_memory({}) - FAILED: {}", id.chars().take(8).collect::<String>(), e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_list_memories(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    memory_type: Option<&str>,
) -> anyhow::Result<()> {
    let mut args = serde_json::json!({});
    if let Some(t) = memory_type {
        args["memory_type"] = serde_json::json!(t);
    }
    
    match client.call_tool("list_memories", args).await {
        Ok(_) => {
            let filter = memory_type.unwrap_or("all");
            crate::teeprintln!("  [OK] list_memories({}) - SUCCESS", filter);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] list_memories({:?}) - FAILED: {}", memory_type, e);
            stats.failed += 1;
        }
    }
    Ok(())
}
