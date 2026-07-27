



//! Agent tool tests
use crate::TestMcpClient;
use crate::TestStats;

/// Run agent tool tests
pub async fn run_agent_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    _filter: Option<&str>,
) -> anyhow::Result<()> {
    println!("\n--- Agent Tools Tests ---");
    
    test_get_workflow(client, stats, "default").await?;
    test_get_workflow(client, stats, "general").await?;
    test_list_tools(client, stats, None).await?;
    test_list_tools(client, stats, Some("memory")).await?;
    test_get_tool(client, stats, "store_memory").await?;
    test_get_tool(client, stats, "search_memory").await?;
    
    Ok(())
}

async fn test_get_workflow(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    purpose: &str,
) -> anyhow::Result<()> {
    match client.call_tool("get_workflow", serde_json::json!({
        "purpose": purpose
    })).await {
        Ok(_) => {
            println!("  ✓ get_workflow({}) - SUCCESS", purpose);
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ get_workflow({}) - FAILED: {}", purpose, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_list_tools(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    category: Option<&str>,
) -> anyhow::Result<()> {
    let mut args = serde_json::json!({});
    if let Some(c) = category {
        args["category"] = serde_json::json!(c);
    }
    
    match client.call_tool("list_tools", args).await {
        Ok(_) => {
            let c = category.unwrap_or("all");
            println!("  ✓ list_tools({}) - SUCCESS", c);
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ list_tools({:?}) - FAILED: {}", category, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_get_tool(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    tool_name: &str,
) -> anyhow::Result<()> {
    match client.call_tool("get_tool", serde_json::json!({
        "name": tool_name
    })).await {
        Ok(_) => {
            println!("  ✓ get_tool('{}') - SUCCESS", tool_name);
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ get_tool('{}') - FAILED: {}", tool_name, e);
            stats.failed += 1;
        }
    }
    Ok(())
}
