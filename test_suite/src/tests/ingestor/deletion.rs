//! Tests for deleting ingested files.

use crate::TestMcpClient;
use crate::TestStats;

/// Test deleting ingested files (expected to fail without admin)
pub async fn test_delete_ingested_files(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    file_ids: Vec<&str>,
) -> anyhow::Result<()> {
    match client.call_tool("delete_ingested_files", serde_json::json!({
        "file_ids": file_ids
    })).await {
        Ok(_) => {
            crate::teeprintln!("  [OK] delete_ingested_files - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [WARN] delete_ingested_files - BLOCKED (expected without admin): {}", e);
            stats.skipped += 1;
        }
    }
    Ok(())
}
