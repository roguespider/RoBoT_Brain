//! Search tool tests
use crate::TestMcpClient;
use crate::TestStats;

/// Run search tool tests
pub async fn run_search_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    _filter: Option<&str>,
) -> anyhow::Result<()> {
    println!("\n--- Search Tools Tests ---");
    
    test_global_search(client, stats, "test").await?;
    test_global_search(client, stats, "memory").await?;
    test_global_search(client, stats, "experience").await?;
    
    test_get_recommendations(client, stats).await?;
    test_get_recommendations(client, stats).await?;
    
    test_get_reputation(client, stats, "test_tool").await?;
    
    Ok(())
}

async fn test_global_search(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    query: &str,
) -> anyhow::Result<()> {
    match client.call_tool("global_search", serde_json::json!({
        "query": query
    })).await {
        Ok(_) => {
            println!("  ✓ global_search('{}') - SUCCESS", query);
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ global_search('{}') - FAILED: {}", query, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_get_recommendations(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("get_recommendations", serde_json::json!({})).await {
        Ok(_) => {
            println!("  ✓ get_recommendations - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ get_recommendations - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_get_reputation(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    tool_name: &str,
) -> anyhow::Result<()> {
    match client.call_tool("get_reputation", serde_json::json!({
        "tool_name": tool_name
    })).await {
        Ok(_) => {
            println!("  ✓ get_reputation('{}') - SUCCESS", tool_name);
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ get_reputation('{}') - FAILED: {}", tool_name, e);
            stats.failed += 1;
        }
    }
    Ok(())
}
