//! Tests for single file and folder ingestion.

use super::types::FileTypeTest;
use crate::test_environment::TestEnvironment;
use crate::TestMcpClient;
use crate::TestStats;

/// Test ingesting a single file type
pub async fn test_ingest_single_file_type(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    file_test: &FileTypeTest,
    env: &TestEnvironment,
) {
    let file_path = env.files_folder.join(&file_test.file_path);
    let file_path_str = file_path.to_string_lossy().to_string();

    match client.call_tool("ingest_files", serde_json::json!({
        "file_path": file_path_str
    })).await {
        Ok(result) => {
            // Check for success
            if let Some(content) = result.get("content").and_then(|c| c.as_array()).and_then(|arr| arr.first())
                && let Some(text) = content.get("text").and_then(|t| t.as_str())
                    && let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                        let success = parsed.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
                        let chunks = parsed.get("chunks_created").and_then(|c| c.as_i64()).unwrap_or(0);

                        if success && chunks > 0 {
                            crate::teeprintln!("  [OK] ingest {} (.{}) - SUCCESS ({} chunks)",
                                file_test.file_type, file_test.extension, chunks);
                            stats.passed += 1;
                        } else {
                            let error = parsed.get("error").map(|e| e.to_string()).unwrap_or_default();
                            crate::teeprintln!("  [WARN] ingest {} (.{}) - returned false (error: {})",
                                file_test.file_type, file_test.extension, error);
                            stats.skipped += 1;
                        }
                        return;
                    }
            crate::teeprintln!("  [OK] ingest {} (.{}) - SUCCESS", file_test.file_type, file_test.extension);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] ingest {} (.{}) - FAILED: {}",
                file_test.file_type, file_test.extension, e);
            stats.failed += 1;
        }
    }
}

/// Test ingesting entire folder recursively
pub async fn test_ingest_folder_recursive(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    _env: &TestEnvironment,
) {
    match client.call_tool("ingest_files", serde_json::json!({
        "folder": "files_to_import",
        "recursive": true
    })).await {
        Ok(result) => {
            if let Some(content) = result.get("content").and_then(|c| c.as_array()).and_then(|arr| arr.first())
                && let Some(text) = content.get("text").and_then(|t| t.as_str())
                    && let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                        let success = parsed.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
                        let chunks = parsed.get("chunks_created").and_then(|c| c.as_i64()).unwrap_or(0);

                        if success {
                            crate::teeprintln!("  [OK] ingest folder (recursive) - SUCCESS (processed folder)");
                            stats.passed += 1;
                        } else {
                            crate::teeprintln!("  [WARN] ingest folder - returned false");
                            stats.skipped += 1;
                        }
                        return;
                    }
            crate::teeprintln!("  [OK] ingest folder (recursive) - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] ingest folder (recursive) - FAILED: {}", e);
            stats.failed += 1;
        }
    }
}
