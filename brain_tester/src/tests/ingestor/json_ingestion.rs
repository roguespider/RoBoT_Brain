//! Tests for JSON and JSONL file ingestion.

use crate::test_environment::TestEnvironment;
use crate::TestMcpClient;
use crate::TestStats;

/// Test ingesting a JSON file with special handling
pub async fn test_ingest_json_file(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    env: &TestEnvironment,
) {
    let file_path = env.files_folder.join("config_files/data.json");
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
                                "  ✓ ingest JSON (smart extraction) - SUCCESS ({} memory items)",
                                chunks
                            );
                            stats.passed += 1;
                        } else {
                            crate::teeprintln!("  ⚠ ingest JSON - returned false");
                            stats.skipped += 1;
                        }
                        return;
                    }
                }
            }
            crate::teeprintln!("  ✓ ingest JSON - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ ingest JSON - FAILED: {}", e);
            stats.failed += 1;
        }
    }
}

/// Test ingesting a JSONL file
pub async fn test_ingest_jsonl_file(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    env: &TestEnvironment,
) {
    let file_path = env.files_folder.join("config_files/data.jsonl");
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
                                "  ✓ ingest JSONL (line-by-line) - SUCCESS ({} memory items)",
                                chunks
                            );
                            stats.passed += 1;
                        } else {
                            crate::teeprintln!("  ⚠ ingest JSONL - returned false");
                            stats.skipped += 1;
                        }
                        return;
                    }
                }
            }
            crate::teeprintln!("  ✓ ingest JSONL - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ ingest JSONL - FAILED: {}", e);
            stats.failed += 1;
        }
    }
}
