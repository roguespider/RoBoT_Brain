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

    // Note: External MCP server tests are informational only
    // connect_mcp_server and call_tool require an actual external MCP server
    // These are tested separately in integration environments
    test_external_mcp_capability(client, stats).await?;

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
            crate::teeprintln!("  [OK] get_workflow({}) - SUCCESS", purpose);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] get_workflow({}) - FAILED: {}", purpose, e);
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
            crate::teeprintln!("  [OK] list_tools({}) - SUCCESS", c);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] list_tools({:?}) - FAILED: {}", category, e);
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
            crate::teeprintln!("  [OK] get_tool('{}') - SUCCESS", tool_name);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] get_tool('{}') - FAILED: {}", tool_name, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

/// Test external MCP server connectivity (informational)
/// Note: These tools require an actual external MCP server to be meaningful
async fn test_external_mcp_capability(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    // Test if the external MCP tools are available
    let tools = client.list_tools().await?;
    
    let has_connect = tools.iter().any(|t| t.get("name").and_then(|n| n.as_str()) == Some("connect_mcp_server"));
    let has_call = tools.iter().any(|t| t.get("name").and_then(|n| n.as_str()) == Some("call_tool"));
    
    if has_connect && has_call {
        crate::teeprintln!("  [INFO] External MCP client tools available (connect_mcp_server, call_tool)");
        crate::teeprintln!("    → These require an actual external MCP server for full testing");
        crate::teeprintln!("    → Skipping live connection tests in unit test environment");
        stats.skipped += 2;
    }
    
    Ok(())
}
