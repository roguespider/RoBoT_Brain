



//! Experience tool tests
use crate::TestMcpClient;
use crate::TestStats;

/// Run experience tool tests
pub async fn run_experience_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    _filter: Option<&str>,
) -> anyhow::Result<()> {
    crate::teeprintln!("\n--- Experience Tools Tests ---");
    
    // Record experiences and get the ID from the first one
    let exp_id = test_record_experience_with_id(client, stats, "Tool Execution Success", "Success", "tool_execution").await?;
    test_record_experience(client, stats, "Memory Lookup", "Success", "memory_lookup").await?;
    test_record_experience(client, stats, "Partial Success", "Partial", "memory_store").await?;
    test_record_experience(client, stats, "Failed Attempt", "Failure", "tool_execution").await?;
    
    // Test get_experience with a valid ID (will return not found but valid UUID)
    test_get_experience(client, stats, &exp_id).await?;
    // Also test with a non-existent but valid UUID
    test_get_experience(client, stats, "00000000-0000-0000-0000-000000000000").await?;
    
    test_list_experiences(client, stats, None).await?;
    test_list_experiences(client, stats, Some("tool_execution")).await?;
    test_get_experience_stats(client, stats, None).await?;
    test_get_experience_stats(client, stats, Some("day")).await?;
    
    Ok(())
}

async fn test_record_experience_with_id(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    title: &str,
    outcome: &str,
    experience_type: &str,
) -> anyhow::Result<String> {
    match client.call_tool("record_experience", serde_json::json!({
        "title": title,
        "description": format!("Test description for {}", title),
        "experience_type": experience_type,
        "outcome": outcome
    })).await {
        Ok(result) => {
            crate::teeprintln!("  [OK] record_experience({}, {}) - SUCCESS", title, outcome);
            stats.passed += 1;
            // Extract the ID from the result
            if let Some(id) = result.get("id").and_then(|v| v.as_str()) {
                Ok(id.to_string())
            } else {
                Ok("00000000-0000-0000-0000-000000000000".to_string())
            }
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] record_experience({}, {}) - FAILED: {}", title, outcome, e);
            stats.failed += 1;
            Ok("00000000-0000-0000-0000-000000000000".to_string())
        }
    }
}

async fn test_record_experience(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    title: &str,
    outcome: &str,
    experience_type: &str,
) -> anyhow::Result<()> {
    match client.call_tool("record_experience", serde_json::json!({
        "title": title,
        "description": format!("Test description for {}", title),
        "experience_type": experience_type,
        "outcome": outcome
    })).await {
        Ok(_) => {
            crate::teeprintln!("  [OK] record_experience({}, {}) - SUCCESS", title, outcome);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] record_experience({}, {}) - FAILED: {}", title, outcome, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_get_experience(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    id: &str,
) -> anyhow::Result<()> {
    match client.call_tool("get_experience", serde_json::json!({
        "id": id
    })).await {
        Ok(_) => {
            crate::teeprintln!("  [OK] get_experience({}) - SUCCESS", id);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] get_experience({}) - FAILED: {}", id, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_list_experiences(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    filter: Option<&str>,
) -> anyhow::Result<()> {
    let mut args = serde_json::json!({});
    if let Some(f) = filter {
        args["filter"] = serde_json::json!(f);
    }
    
    match client.call_tool("list_experiences", args).await {
        Ok(_) => {
            let f = filter.unwrap_or("all");
            crate::teeprintln!("  [OK] list_experiences({}) - SUCCESS", f);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] list_experiences({:?}) - FAILED: {}", filter, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_get_experience_stats(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    time_window: Option<&str>,
) -> anyhow::Result<()> {
    let mut args = serde_json::json!({});
    if let Some(t) = time_window {
        args["time_window"] = serde_json::json!(t);
    }
    
    match client.call_tool("get_experience_stats", args).await {
        Ok(_) => {
            let t = time_window.unwrap_or("all");
            crate::teeprintln!("  [OK] get_experience_stats({}) - SUCCESS", t);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] get_experience_stats({:?}) - FAILED: {}", time_window, e);
            stats.failed += 1;
        }
    }
    Ok(())
}
