//! Workflow tool tests
use crate::TestMcpClient;
use crate::TestStats;

/// Run workflow tool tests
pub async fn run_workflow_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    _filter: Option<&str>,
) -> anyhow::Result<()> {
    println!("\n--- Workflow Tools Tests ---");
    
    test_create_workflow(client, stats, "Test Workflow").await?;
    test_add_workflow_step(client, stats, "Step 1", "store_memory").await?;
    test_add_workflow_step(client, stats, "Step 2", "search_memory").await?;
    test_add_workflow_step(client, stats, "Step 3", "record_experience").await?;
    test_get_workflow_status(client, stats).await?;
    test_start_workflow(client, stats).await?;
    test_pause_workflow(client, stats).await?;
    test_resume_workflow(client, stats).await?;
    
    // Test cancel/delete workflow
    test_create_workflow(client, stats, "Cancel Test").await?;
    test_cancel_workflow(client, stats).await?;
    test_delete_workflow(client, stats).await?;
    
    // Test list workflows
    test_list_workflows(client, stats, None).await?;
    test_list_workflows(client, stats, Some("running")).await?;
    test_list_workflows(client, stats, Some("completed")).await?;
    
    Ok(())
}

async fn test_create_workflow(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    name: &str,
) -> anyhow::Result<()> {
    match client.call_tool("create_workflow", serde_json::json!({
        "name": name
    })).await {
        Ok(_) => {
            println!("  ✓ create_workflow('{}') - SUCCESS", name);
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ create_workflow('{}') - FAILED: {}", name, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_add_workflow_step(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    name: &str,
    tool_name: &str,
) -> anyhow::Result<()> {
    match client.call_tool("add_workflow_step", serde_json::json!({
        "name": name,
        "tool_name": tool_name
    })).await {
        Ok(_) => {
            println!("  ✓ add_workflow_step('{}', '{}') - SUCCESS", name, tool_name);
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ add_workflow_step('{}', '{}') - FAILED: {}", name, tool_name, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_get_workflow_status(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("get_workflow_status", serde_json::json!({})).await {
        Ok(_) => {
            println!("  ✓ get_workflow_status - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ get_workflow_status - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_start_workflow(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("start_workflow", serde_json::json!({})).await {
        Ok(_) => {
            println!("  ✓ start_workflow - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ start_workflow - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_pause_workflow(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("pause_workflow", serde_json::json!({})).await {
        Ok(_) => {
            println!("  ✓ pause_workflow - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ pause_workflow - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_resume_workflow(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("resume_workflow", serde_json::json!({})).await {
        Ok(_) => {
            println!("  ✓ resume_workflow - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ resume_workflow - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_cancel_workflow(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("cancel_workflow", serde_json::json!({})).await {
        Ok(_) => {
            println!("  ✓ cancel_workflow - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ cancel_workflow - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_delete_workflow(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("delete_workflow", serde_json::json!({})).await {
        Ok(_) => {
            println!("  ✓ delete_workflow - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ delete_workflow - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_list_workflows(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    status: Option<&str>,
) -> anyhow::Result<()> {
    let mut args = serde_json::json!({});
    if let Some(s) = status {
        args["status"] = serde_json::json!(s);
    }
    
    match client.call_tool("list_workflows", args).await {
        Ok(_) => {
            let s = status.unwrap_or("all");
            println!("  ✓ list_workflows({}) - SUCCESS", s);
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ list_workflows({:?}) - FAILED: {}", status, e);
            stats.failed += 1;
        }
    }
    Ok(())
}
