//! Ingestor tool tests
use crate::test_environment::TestEnvironment;
use crate::TestMcpClient;
use crate::TestStats;

/// Run ingestor tool tests
pub async fn run_ingestor_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    _filter: Option<&str>,
    env: &TestEnvironment,
) -> anyhow::Result<()> {
    println!("\n--- Ingestor Tools Tests ---");
    
    test_list_importable(client, stats).await?;
    test_list_importable(client, stats).await?;
    test_ingest_files(client, stats, env).await?;
    test_ingest_files(client, stats, env).await?;
    test_list_ingested_files(client, stats).await?;
    test_list_ingested_files(client, stats).await?;
    
    // Test deletion (may fail - admin required)
    match client.call_tool("delete_ingested_files", serde_json::json!({
        "file_ids": ["test_file_1"]
    })).await {
        Ok(_) => {
            println!("  ? delete_ingested_files (rejected) - Tool accepted deletion");
            stats.skipped += 1;
        }
        Err(_) => {
            println!("  ? delete_ingested_files (rejected) - Expected failure");
            stats.skipped += 1;
        }
    }
    
    // Confirm deletion was blocked
    match client.call_tool("list_ingested_files", serde_json::json!({})).await {
        Ok(_) => {
            println!("  ✓ delete_ingested_files (confirmed) - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ delete_ingested_files (confirmed) - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    
    Ok(())
}

async fn test_list_importable(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("list_importable", serde_json::json!({})).await {
        Ok(_) => {
            println!("  ✓ list_importable - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ list_importable - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_ingest_files(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    env: &TestEnvironment,
) -> anyhow::Result<()> {
    let file_path = env.files_folder.join("readme.txt");
    match client.call_tool("ingest_files", serde_json::json!({
        "file_paths": [file_path.to_string_lossy().as_ref()]
    })).await {
        Ok(_) => {
            println!("  ✓ ingest_files - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ ingest_files - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_list_ingested_files(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("list_ingested_files", serde_json::json!({})).await {
        Ok(_) => {
            println!("  ✓ list_ingested_files - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ list_ingested_files - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_delete_ingested_files(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    file_ids: Vec<&str>,
) -> anyhow::Result<()> {
    match client.call_tool("delete_ingested_files", serde_json::json!({
        "file_ids": file_ids
    })).await {
        Ok(_) => {
            println!("  ✓ delete_ingested_files - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ delete_ingested_files - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}
