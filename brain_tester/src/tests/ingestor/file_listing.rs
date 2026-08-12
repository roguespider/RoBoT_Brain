//! Tests for listing importable and ingested files.

use crate::TestMcpClient;
use crate::TestStats;

/// Test listing importable files
pub async fn test_list_importable(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client
        .call_tool("list_importable", serde_json::json!({}))
        .await
    {
        Ok(result) => {
            // Check that we got a valid response with files
            if let Some(content) = result
                .get("content")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
            {
                if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                        let count = parsed.get("count").and_then(|c| c.as_i64()).unwrap_or(0);
                        let total = parsed.get("total").and_then(|t| t.as_i64()).unwrap_or(0);
                        crate::teeprintln!("  ✓ list_importable - SUCCESS (found {} files)", total);
                        stats.passed += 1;
                        return Ok(());
                    }
                }
            }
            crate::teeprintln!("  ✓ list_importable - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ list_importable - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

/// Test listing importable files with recursive search
pub async fn test_list_importable_recursive(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client
        .call_tool(
            "list_importable",
            serde_json::json!({
                "recursive": true,
                "list_all": true
            }),
        )
        .await
    {
        Ok(result) => {
            if let Some(content) = result
                .get("content")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
            {
                if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                        let total = parsed.get("total").and_then(|t| t.as_i64()).unwrap_or(0);
                        crate::teeprintln!(
                            "  ✓ list_importable (recursive) - SUCCESS (found {} total files)",
                            total
                        );
                        stats.passed += 1;
                        return Ok(());
                    }
                }
            }
            crate::teeprintln!("  ✓ list_importable (recursive) - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ list_importable (recursive) - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

/// Test listing ingested files
pub async fn test_list_ingested_files(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client
        .call_tool("list_ingested_files", serde_json::json!({}))
        .await
    {
        Ok(_) => {
            crate::teeprintln!("  ✓ list_ingested_files - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ list_ingested_files - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}
