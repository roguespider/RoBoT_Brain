//! Planner tool tests
use crate::TestMcpClient;
use crate::TestStats;

/// Run planner tool tests
pub async fn run_planner_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    _filter: Option<&str>,
) -> anyhow::Result<()> {
    println!("\n--- Planner Tools Tests ---");
    
    test_create_plan(client, stats, "Complete feature implementation").await?;
    test_add_plan_step(client, stats, "Step 1: Design").await?;
    test_add_plan_step(client, stats, "Step 2: Implement").await?;
    test_add_plan_step(client, stats, "Step 3: Test").await?;
    test_add_step_dependency(client, stats, 0, 1).await?;
    test_add_step_dependency(client, stats, 1, 2).await?;
    test_get_plan(client, stats).await?;
    test_start_plan(client, stats).await?;
    test_complete_step(client, stats, 0).await?;
    test_fail_step(client, stats, 1).await?;
    
    // Test cancel
    test_create_plan(client, stats, "Cancel test").await?;
    test_cancel_plan(client, stats).await?;
    
    // Test list plans
    test_list_plans(client, stats, None).await?;
    test_list_plans(client, stats, Some("active")).await?;
    
    Ok(())
}

async fn test_create_plan(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    description: &str,
) -> anyhow::Result<()> {
    match client.call_tool("create_plan", serde_json::json!({
        "description": description
    })).await {
        Ok(_) => {
            let truncated = if description.len() > 30 { &description[..30] } else { description };
            println!("  ✓ create_plan('{}...') - SUCCESS", truncated);
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ create_plan('{}') - FAILED: {}", description, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_add_plan_step(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    description: &str,
) -> anyhow::Result<()> {
    match client.call_tool("add_plan_step", serde_json::json!({
        "description": description
    })).await {
        Ok(_) => {
            println!("  ✓ add_plan_step('{}') - SUCCESS", description);
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ add_plan_step('{}') - FAILED: {}", description, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_add_step_dependency(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    from_step: usize,
    to_step: usize,
) -> anyhow::Result<()> {
    match client.call_tool("add_step_dependency", serde_json::json!({
        "from_step": from_step,
        "to_step": to_step
    })).await {
        Ok(_) => {
            println!("  ✓ add_step_dependency - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ add_step_dependency - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_get_plan(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("get_plan", serde_json::json!({})).await {
        Ok(_) => {
            println!("  ✓ get_plan - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ get_plan - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_start_plan(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("start_plan", serde_json::json!({})).await {
        Ok(_) => {
            println!("  ✓ start_plan - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ start_plan - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_complete_step(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    step_index: usize,
) -> anyhow::Result<()> {
    match client.call_tool("complete_step", serde_json::json!({
        "step_index": step_index
    })).await {
        Ok(_) => {
            println!("  ✓ complete_step({}) - SUCCESS", step_index);
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ complete_step({}) - FAILED: {}", step_index, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_fail_step(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    step_index: usize,
) -> anyhow::Result<()> {
    match client.call_tool("fail_step", serde_json::json!({
        "step_index": step_index,
        "error": "Test failure"
    })).await {
        Ok(_) => {
            println!("  ✓ fail_step({}) - SUCCESS", step_index);
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ fail_step({}) - FAILED: {}", step_index, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_cancel_plan(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("cancel_plan", serde_json::json!({})).await {
        Ok(_) => {
            println!("  ✓ cancel_plan - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ cancel_plan - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_list_plans(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    status: Option<&str>,
) -> anyhow::Result<()> {
    let mut args = serde_json::json!({});
    if let Some(s) = status {
        args["status"] = serde_json::json!(s);
    }
    
    match client.call_tool("list_plans", args).await {
        Ok(_) => {
            let s = status.unwrap_or("all");
            println!("  ✓ list_plans({}) - SUCCESS", s);
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ list_plans({:?}) - FAILED: {}", status, e);
            stats.failed += 1;
        }
    }
    Ok(())
}
