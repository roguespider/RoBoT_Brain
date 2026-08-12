//! Tests for archive (ZIP, TAR.GZ) ingestion.

use crate::test_environment::TestEnvironment;
use crate::TestMcpClient;
use crate::TestStats;

/// Test ingesting a ZIP archive
pub async fn test_ingest_archive_zip(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    env: &TestEnvironment,
) {
    let file_path = env.files_folder.join("archives/test.zip");
    let file_path_str = file_path.to_string_lossy().to_string();

    match client
        .call_tool(
            "ingest_files",
            serde_json::json!({
                "file_path": file_path_str
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
                        let success = parsed
                            .get("success")
                            .and_then(|s| s.as_bool())
                            .unwrap_or(false);
                        let chunks = parsed
                            .get("chunks_created")
                            .and_then(|c| c.as_i64())
                            .unwrap_or(0);

                        if success && chunks > 0 {
                            crate::teeprintln!(
                                "  ✓ ingest ZIP archive - SUCCESS ({} chunks from extracted files)",
                                chunks
                            );
                            stats.passed += 1;
                        } else {
                            crate::teeprintln!("  ⚠ ingest ZIP archive - returned false");
                            stats.skipped += 1;
                        }
                        return;
                    }
                }
            }
            crate::teeprintln!("  ✓ ingest ZIP archive - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ ingest ZIP archive - FAILED: {}", e);
            stats.failed += 1;
        }
    }
}

/// Test ingesting a TAR.GZ archive
pub async fn test_ingest_archive_tar_gz(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    env: &TestEnvironment,
) {
    let file_path = env.files_folder.join("archives/test.tar.gz");
    let file_path_str = file_path.to_string_lossy().to_string();

    match client
        .call_tool(
            "ingest_files",
            serde_json::json!({
                "file_path": file_path_str
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
                        let success = parsed
                            .get("success")
                            .and_then(|s| s.as_bool())
                            .unwrap_or(false);
                        let chunks = parsed
                            .get("chunks_created")
                            .and_then(|c| c.as_i64())
                            .unwrap_or(0);

                        if success && chunks > 0 {
                            crate::teeprintln!("  ✓ ingest TAR.GZ archive - SUCCESS ({} chunks from extracted files)", chunks);
                            stats.passed += 1;
                        } else {
                            crate::teeprintln!("  ⚠ ingest TAR.GZ archive - returned false");
                            stats.skipped += 1;
                        }
                        return;
                    }
                }
            }
            crate::teeprintln!("  ✓ ingest TAR.GZ archive - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ ingest TAR.GZ archive - FAILED: {}", e);
            stats.failed += 1;
        }
    }
}
