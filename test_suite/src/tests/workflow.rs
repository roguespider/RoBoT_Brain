



//! Workflow tool tests
use crate::TestMcpClient;
use crate::TestStats;

/// Run workflow tool tests
pub async fn run_workflow_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    _filter: Option<&str>,
) -> anyhow::Result<()> {
    crate::teeprintln!("\n--- Workflow Tools Tests ---");
    
    let workflow_id = test_create_workflow(client, stats, "Test Workflow").await?;
    
    if let Some(ref wid) = workflow_id {
        test_add_workflow_step(client, stats, wid, "Step 1", "store_memory").await?;
        test_add_workflow_step(client, stats, wid, "Step 2", "search_memory").await?;
        test_add_workflow_step(client, stats, wid, "Step 3", "record_experience").await?;
        test_get_workflow_status(client, stats, wid).await?;
        test_start_workflow(client, stats, wid).await?;
        test_pause_workflow(client, stats, wid).await?;
        test_resume_workflow(client, stats, wid).await?;
    }
    
    // Test cancel/delete workflow with a new workflow
    let cancel_wid = test_create_workflow(client, stats, "Cancel Test").await?;
    if let Some(ref wid) = cancel_wid {
        test_cancel_workflow(client, stats, wid).await?;
        test_delete_workflow(client, stats, wid).await?;
    }
    
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
) -> anyhow::Result<Option<String>> {
    match client.call_tool("create_workflow", serde_json::json!({
        "name": name
    })).await {
        Ok(result) => {
            // Try to extract workflow_id from result
            let workflow_id = extract_workflow_id(&result);
            crate::teeprintln!("  [OK] create_workflow('{}') - SUCCESS", name);
            stats.passed += 1;
            Ok(workflow_id.or_else(|| Some("test_workflow_001".to_string())))
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] create_workflow('{}') - FAILED: {}", name, e);
            stats.failed += 1;
            Ok(None)
        }
    }
}

fn extract_workflow_id(result: &serde_json::Value) -> Option<String> {
    // Try to parse the JSON and extract id field
    if let Some(id) = result.get("id").and_then(|v| v.as_str()) {
        return Some(id.to_string());
    }
    // Check in data field
    if let Some(data) = result.get("data").and_then(|v| v.as_object())
        && let Some(id) = data.get("id").and_then(|v| v.as_str()) {
            return Some(id.to_string());
        }
    None
}

async fn test_add_workflow_step(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    workflow_id: &str,
    name: &str,
    action: &str,
) -> anyhow::Result<()> {
    match client.call_tool("add_workflow_step", serde_json::json!({
        "workflow_id": workflow_id,
        "name": name,
        "action": action
    })).await {
        Ok(_) => {
            crate::teeprintln!("  [OK] add_workflow_step('{}', '{}') - SUCCESS", name, action);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] add_workflow_step('{}', '{}') - FAILED: {}", name, action, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_get_workflow_status(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    workflow_id: &str,
) -> anyhow::Result<()> {
    match client.call_tool("get_workflow_status", serde_json::json!({
        "workflow_id": workflow_id
    })).await {
        Ok(_) => {
            crate::teeprintln!("  [OK] get_workflow_status - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] get_workflow_status - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_start_workflow(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    workflow_id: &str,
) -> anyhow::Result<()> {
    match client.call_tool("start_workflow", serde_json::json!({
        "workflow_id": workflow_id
    })).await {
        Ok(_) => {
            crate::teeprintln!("  [OK] start_workflow - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] start_workflow - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_pause_workflow(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    workflow_id: &str,
) -> anyhow::Result<()> {
    match client.call_tool("pause_workflow", serde_json::json!({
        "workflow_id": workflow_id
    })).await {
        Ok(_) => {
            crate::teeprintln!("  [OK] pause_workflow - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] pause_workflow - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_resume_workflow(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    workflow_id: &str,
) -> anyhow::Result<()> {
    match client.call_tool("resume_workflow", serde_json::json!({
        "workflow_id": workflow_id
    })).await {
        Ok(_) => {
            crate::teeprintln!("  [OK] resume_workflow - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] resume_workflow - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_cancel_workflow(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    workflow_id: &str,
) -> anyhow::Result<()> {
    match client.call_tool("cancel_workflow", serde_json::json!({
        "workflow_id": workflow_id
    })).await {
        Ok(_) => {
            crate::teeprintln!("  [OK] cancel_workflow - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] cancel_workflow - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_delete_workflow(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    workflow_id: &str,
) -> anyhow::Result<()> {
    match client.call_tool("delete_workflow", serde_json::json!({
        "workflow_id": workflow_id
    })).await {
        Ok(_) => {
            crate::teeprintln!("  [OK] delete_workflow - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] delete_workflow - FAILED: {}", e);
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
            crate::teeprintln!("  [OK] list_workflows({}) - SUCCESS", s);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] list_workflows({:?}) - FAILED: {}", status, e);
            stats.failed += 1;
        }
    }
    Ok(())
}
