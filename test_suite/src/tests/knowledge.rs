



//! Knowledge tool tests
use crate::TestMcpClient;
use crate::TestStats;

/// Run knowledge tool tests
pub async fn run_knowledge_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    _filter: Option<&str>,
) -> anyhow::Result<()> {
    crate::teeprintln!("\n--- Knowledge Tools Tests ---");
    
    let kno_id = test_add_knowledge(client, stats, "Files should be imported before processing").await?;
    test_add_knowledge(client, stats, "Memory system stores context between sessions").await?;
    test_add_knowledge(client, stats, "Workflow enforces agent behavior patterns").await?;
    
    test_query_knowledge(client, stats, "files").await?;
    test_query_knowledge(client, stats, "memory").await?;
    test_query_knowledge(client, stats, "workflow").await?;
    
    test_get_mature_knowledge(client, stats, Some(5)).await?;
    test_get_knowledge_stats(client, stats).await?;
    
    // Record application for a valid knowledge ID
    if let Some(ref id) = kno_id {
        test_record_knowledge_application(client, stats, id, true).await?;
        test_record_knowledge_application(client, stats, id, false).await?;
    } else {
        // Skip if no valid ID was created
        stats.skipped += 2;
    }
    
    Ok(())
}

async fn test_add_knowledge(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    content: &str,
) -> anyhow::Result<Option<String>> {
    match client.call_tool("add_knowledge", serde_json::json!({
        "statement": content
    })).await {
        Ok(result) => {
            let truncated = if content.len() > 40 { &content[..40] } else { content };
            crate::teeprintln!("  ✓ add_knowledge('{}...') - SUCCESS", truncated);
            stats.passed += 1;
            // Try to extract knowledge_id from result
            Ok(extract_knowledge_id(&result))
        }
        Err(e) => {
            crate::teeprintln!("  ✗ add_knowledge('{}...') - FAILED: {}", &content[..40.min(content.len())], e);
            stats.failed += 1;
            Ok(None)
        }
    }
}

fn extract_knowledge_id(result: &serde_json::Value) -> Option<String> {
    if let Some(id) = result.get("knowledge_id").and_then(|v| v.as_str()) {
        return Some(id.to_string());
    }
    if let Some(id) = result.get("id").and_then(|v| v.as_str()) {
        return Some(id.to_string());
    }
    if let Some(data) = result.get("data").and_then(|v| v.as_object()) {
        if let Some(id) = data.get("knowledge_id").and_then(|v| v.as_str()) {
            return Some(id.to_string());
        }
        if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
            return Some(id.to_string());
        }
    }
    None
}

async fn test_query_knowledge(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    query: &str,
) -> anyhow::Result<()> {
    match client.call_tool("query_knowledge", serde_json::json!({
        "query": query
    })).await {
        Ok(_) => {
            crate::teeprintln!("  ✓ query_knowledge('{}') - SUCCESS", query);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ query_knowledge('{}') - FAILED: {}", query, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_get_mature_knowledge(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    min_applications: Option<i32>,
) -> anyhow::Result<()> {
    let mut args = serde_json::json!({});
    if let Some(m) = min_applications {
        args["min_applications"] = serde_json::json!(m);
    }
    
    match client.call_tool("get_mature_knowledge", args).await {
        Ok(_) => {
            crate::teeprintln!("  ✓ get_mature_knowledge({:?}) - SUCCESS", min_applications);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ get_mature_knowledge({:?}) - FAILED: {}", min_applications, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_get_knowledge_stats(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("get_knowledge_stats", serde_json::json!({})).await {
        Ok(_) => {
            crate::teeprintln!("  ✓ get_knowledge_stats - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ get_knowledge_stats - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_record_knowledge_application(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    knowledge_id: &str,
    successful: bool,
) -> anyhow::Result<()> {
    match client.call_tool("record_knowledge_application", serde_json::json!({
        "knowledge_id": knowledge_id,
        "success": successful
    })).await {
        Ok(_) => {
            crate::teeprintln!("  ✓ record_knowledge_application({}, {}) - SUCCESS", knowledge_id, successful);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ record_knowledge_application({}, {}) - FAILED: {}", knowledge_id, successful, e);
            stats.failed += 1;
        }
    }
    Ok(())
}
