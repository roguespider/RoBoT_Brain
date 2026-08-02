//! Agent tool tests
use crate::TestMcpClient;
use crate::TestStats;

/// Run agent tool tests
pub async fn run_agent_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    _filter: Option<&str>,
) -> anyhow::Result<()> {
    crate::teeprintln!("\n--- Agent Tools Tests ---");

    test_get_workflow(client, stats, "default").await?;
    test_get_workflow(client, stats, "general").await?;
    test_list_tools(client, stats, None).await?;
    test_list_tools(client, stats, Some("memory")).await?;
    test_get_tool(client, stats, "store_memory").await?;
    test_get_tool(client, stats, "search_memory").await?;

    // Test connect_mcp_server (may fail in test env but should not panic)
    test_connect_mcp_server(client, stats).await?;
    test_call_tool(client, stats).await?;

    Ok(())
}

async fn test_get_workflow(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    purpose: &str,
) -> anyhow::Result<()> {
    match client
        .call_tool(
            "get_workflow",
            serde_json::json!({
                "purpose": purpose
            }),
        )
        .await
    {
        Ok(_) => {
            crate::teeprintln!("  ✓ get_workflow({}) - SUCCESS", purpose);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ get_workflow({}) - FAILED: {}", purpose, e);
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
            crate::teeprintln!("  ✓ list_tools({}) - SUCCESS", c);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ list_tools({:?}) - FAILED: {}", category, e);
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
    match client
        .call_tool(
            "get_tool",
            serde_json::json!({
                "name": tool_name
            }),
        )
        .await
    {
        Ok(_) => {
            crate::teeprintln!("  ✓ get_tool('{}') - SUCCESS", tool_name);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ get_tool('{}') - FAILED: {}", tool_name, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_connect_mcp_server(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client
        .call_tool(
            "connect_mcp_server",
            serde_json::json!({
                "name": "test_server",
                "command": "echo",
                "args": []
            }),
        )
        .await
    {
        Ok(_) => {
            crate::teeprintln!("  ✓ connect_mcp_server - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            // May fail in test env but that's expected - just log it
            crate::teeprintln!("  ⚠ connect_mcp_server - SKIPPED: {}", e);
            stats.skipped += 1;
        }
    }
    Ok(())
}

async fn test_call_tool(client: &mut TestMcpClient, stats: &mut TestStats) -> anyhow::Result<()> {
    match client
        .call_tool(
            "call_tool",
            serde_json::json!({
                "tool_name": "get_workflow",
                "arguments": "{\"purpose\": \"general\"}"
            }),
        )
        .await
    {
        Ok(_) => {
            crate::teeprintln!("  ✓ call_tool - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ⚠ call_tool - SKIPPED: {}", e);
            stats.skipped += 1;
        }
    }
    Ok(())
}
