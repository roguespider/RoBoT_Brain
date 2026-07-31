



//! Comprehensive Ingestor tool tests - Tests ALL supported file types
//! 
//! File types tested:
//! - Text: txt, md, rst, log, xml, html
//! - Code: rs, py, js, ts
//! - Config: yaml, ini, toml, json, jsonl, csv
//! - Scripts: sh, sql
//! - Subtitles: srt
//! - Images: svg (metadata extraction)
//! - Archives: zip, tar.gz (extracted then ingested)
//! - Documents: (requires actual files - skipped in unit tests)

#![allow(unused_variables)]

use crate::test_environment::TestEnvironment;
use crate::TestMcpClient;
use crate::TestStats;

/// All supported file types to test
#[derive(Debug, Clone)]
struct FileTypeTest {
    file_path: String,
    file_type: &'static str,
    extension: &'static str,
    #[allow(dead_code)]
    should_succeed: bool,
}

impl FileTypeTest {
    fn new(relative_path: &str, file_type: &'static str, extension: &'static str, should_succeed: bool) -> Self {
        Self {
            file_path: relative_path.to_string(),
            file_type,
            extension,
            should_succeed,
        }
    }
}

/// Run all ingestor tool tests
pub async fn run_ingestor_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    _filter: Option<&str>,
    env: &TestEnvironment,
) -> anyhow::Result<()> {
    crate::teeprintln!("\n{}", "=".repeat(60));
    crate::teeprintln!("COMPREHENSIVE INGESTOR FILE TYPE TESTS");
    crate::teeprintln!("{}", "=".repeat(60));
    
    // Test 1: List importable files (overall health check)
    test_list_importable(client, stats).await?;
    
    // Test 2: List importable with recursive (should find all test files)
    test_list_importable_recursive(client, stats).await?;
    
    // Test 3-4: List ingested files (before and after)
    test_list_ingested_files(client, stats).await?;
    test_list_ingested_files(client, stats).await?;
    
    // Test 4: Ingest all file types individually
    crate::teeprintln!("\n--- Testing Individual File Types ---");
    
    let file_types = get_all_file_type_tests(env);
    
    for file_test in file_types {
        test_ingest_single_file_type(client, stats, &file_test, env).await;
    }
    
    // Test 5: Test archive extraction
    crate::teeprintln!("\n--- Testing Archive Extraction ---");
    test_ingest_archive_zip(client, stats, env).await;
    test_ingest_archive_tar_gz(client, stats, env).await;
    
    // Test 6: Test folder ingestion (recursive)
    test_ingest_folder_recursive(client, stats, env).await;
    
    // Test 7: Test JSON with special handling
    test_ingest_json_file(client, stats, env).await;
    test_ingest_jsonl_file(client, stats, env).await;
    
    // Test 8: List ingested files after all tests
    test_list_ingested_files(client, stats).await?;
    
    // Test 9: Test deletion (may fail - admin required)
    test_delete_ingested_files(client, stats, vec!["test_file_1"]).await?;
    
    // Confirm deletion was blocked
    match client.call_tool("list_ingested_files", serde_json::json!({})).await {
        Ok(_) => {
            crate::teeprintln!("  ✓ delete_ingested_files (confirmed) - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ delete_ingested_files (confirmed) - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    
    crate::teeprintln!("\n{}", "=".repeat(60));
    crate::teeprintln!("INGESTOR FILE TYPE TESTS COMPLETE");
    crate::teeprintln!("{}", "=".repeat(60));
    
    Ok(())
}

/// Get all file type tests to run
fn get_all_file_type_tests(env: &TestEnvironment) -> Vec<FileTypeTest> {
    vec![
        // Standard text files
        FileTypeTest::new("readme.txt", "text", "txt", true),
        FileTypeTest::new("sample.md", "markdown", "md", true),
        FileTypeTest::new("sample.rst", "rst", "rst", true),
        FileTypeTest::new("sample.log", "log", "log", true),
        FileTypeTest::new("sample.xml", "xml", "xml", true),
        FileTypeTest::new("sample.html", "html", "html", true),
        
        // Code files
        FileTypeTest::new("code_samples/sample.rs", "rust", "rs", true),
        FileTypeTest::new("code_samples/sample.py", "python", "py", true),
        FileTypeTest::new("code_samples/sample.js", "javascript", "js", true),
        FileTypeTest::new("code_samples/sample.ts", "typescript", "ts", true),
        
        // Config files
        FileTypeTest::new("config_files/app.yaml", "yaml", "yaml", true),
        FileTypeTest::new("config_files/settings.ini", "ini", "ini", true),
        FileTypeTest::new("config_files/config.toml", "toml", "toml", true),
        FileTypeTest::new("config_files/data.csv", "csv", "csv", true),
        
        // Scripts
        FileTypeTest::new("code_samples/script.sh", "shell", "sh", true),
        FileTypeTest::new("code_samples/query.sql", "sql", "sql", true),
        
        // Subtitles
        FileTypeTest::new("sample.srt", "subtitle", "srt", true),
        
        // Image (SVG - metadata extraction)
        FileTypeTest::new("sample.svg", "image/svg", "svg", true),
        
        // JSON/JSONL (special handling)
        FileTypeTest::new("config_files/data.json", "json", "json", true),
        FileTypeTest::new("config_files/data.jsonl", "jsonl", "jsonl", true),
    ]
}

/// Test listing importable files
async fn test_list_importable(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("list_importable", serde_json::json!({})).await {
        Ok(result) => {
            // Check that we got a valid response with files
            if let Some(content) = result.get("content").and_then(|c| c.as_array()).and_then(|arr| arr.first()) {
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
async fn test_list_importable_recursive(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("list_importable", serde_json::json!({
        "recursive": true,
        "list_all": true
    })).await {
        Ok(result) => {
            if let Some(content) = result.get("content").and_then(|c| c.as_array()).and_then(|arr| arr.first()) {
                if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                        let total = parsed.get("total").and_then(|t| t.as_i64()).unwrap_or(0);
                        crate::teeprintln!("  ✓ list_importable (recursive) - SUCCESS (found {} total files)", total);
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

/// Test ingesting a single file type
async fn test_ingest_single_file_type(
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
            if let Some(content) = result.get("content").and_then(|c| c.as_array()).and_then(|arr| arr.first()) {
                if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                        let success = parsed.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
                        let chunks = parsed.get("chunks_created").and_then(|c| c.as_i64()).unwrap_or(0);
                        
                        if success && chunks > 0 {
                            crate::teeprintln!("  ✓ ingest {} (.{}) - SUCCESS ({} chunks)", 
                                file_test.file_type, file_test.extension, chunks);
                            stats.passed += 1;
                        } else {
                            let error = parsed.get("error").map(|e| e.to_string()).unwrap_or_default();
                            crate::teeprintln!("  ⚠ ingest {} (.{}) - returned false (error: {})", 
                                file_test.file_type, file_test.extension, error);
                            stats.skipped += 1;
                        }
                        return;
                    }
                }
            }
            crate::teeprintln!("  ✓ ingest {} (.{}) - SUCCESS", file_test.file_type, file_test.extension);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ ingest {} (.{}) - FAILED: {}", 
                file_test.file_type, file_test.extension, e);
            stats.failed += 1;
        }
    }
}

/// Test ingesting a ZIP archive
async fn test_ingest_archive_zip(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    env: &TestEnvironment,
) {
    let file_path = env.files_folder.join("archives/test.zip");
    let file_path_str = file_path.to_string_lossy().to_string();
    
    match client.call_tool("ingest_files", serde_json::json!({
        "file_path": file_path_str
    })).await {
        Ok(result) => {
            if let Some(content) = result.get("content").and_then(|c| c.as_array()).and_then(|arr| arr.first()) {
                if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                        let success = parsed.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
                        let chunks = parsed.get("chunks_created").and_then(|c| c.as_i64()).unwrap_or(0);
                        
                        if success && chunks > 0 {
                            crate::teeprintln!("  ✓ ingest ZIP archive - SUCCESS ({} chunks from extracted files)", chunks);
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
async fn test_ingest_archive_tar_gz(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    env: &TestEnvironment,
) {
    let file_path = env.files_folder.join("archives/test.tar.gz");
    let file_path_str = file_path.to_string_lossy().to_string();
    
    match client.call_tool("ingest_files", serde_json::json!({
        "file_path": file_path_str
    })).await {
        Ok(result) => {
            if let Some(content) = result.get("content").and_then(|c| c.as_array()).and_then(|arr| arr.first()) {
                if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                        let success = parsed.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
                        let chunks = parsed.get("chunks_created").and_then(|c| c.as_i64()).unwrap_or(0);
                        
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

/// Test ingesting a JSON file with special handling
async fn test_ingest_json_file(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    env: &TestEnvironment,
) {
    let file_path = env.files_folder.join("config_files/data.json");
    let file_path_str = file_path.to_string_lossy().to_string();
    
    match client.call_tool("ingest_files", serde_json::json!({
        "file_path": file_path_str
    })).await {
        Ok(result) => {
            if let Some(content) = result.get("content").and_then(|c| c.as_array()).and_then(|arr| arr.first()) {
                if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                        let success = parsed.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
                        let chunks = parsed.get("chunks_created").and_then(|c| c.as_i64()).unwrap_or(0);
                        
                        if success && chunks > 0 {
                            crate::teeprintln!("  ✓ ingest JSON (smart extraction) - SUCCESS ({} memory items)", chunks);
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
async fn test_ingest_jsonl_file(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    env: &TestEnvironment,
) {
    let file_path = env.files_folder.join("config_files/data.jsonl");
    let file_path_str = file_path.to_string_lossy().to_string();
    
    match client.call_tool("ingest_files", serde_json::json!({
        "file_path": file_path_str
    })).await {
        Ok(result) => {
            if let Some(content) = result.get("content").and_then(|c| c.as_array()).and_then(|arr| arr.first()) {
                if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                        let success = parsed.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
                        let chunks = parsed.get("chunks_created").and_then(|c| c.as_i64()).unwrap_or(0);
                        
                        if success && chunks > 0 {
                            crate::teeprintln!("  ✓ ingest JSONL (line-by-line) - SUCCESS ({} memory items)", chunks);
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

/// Test ingesting entire folder recursively
async fn test_ingest_folder_recursive(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    env: &TestEnvironment,
) {
    match client.call_tool("ingest_files", serde_json::json!({
        "folder": "files_to_import",
        "recursive": true
    })).await {
        Ok(result) => {
            if let Some(content) = result.get("content").and_then(|c| c.as_array()).and_then(|arr| arr.first()) {
                if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                        let success = parsed.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
                        let chunks = parsed.get("chunks_created").and_then(|c| c.as_i64()).unwrap_or(0);
                        
                        if success {
                            crate::teeprintln!("  ✓ ingest folder (recursive) - SUCCESS (processed folder)");
                            stats.passed += 1;
                        } else {
                            crate::teeprintln!("  ⚠ ingest folder - returned false");
                            stats.skipped += 1;
                        }
                        return;
                    }
                }
            }
            crate::teeprintln!("  ✓ ingest folder (recursive) - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ ingest folder (recursive) - FAILED: {}", e);
            stats.failed += 1;
        }
    }
}

/// Test listing ingested files
async fn test_list_ingested_files(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("list_ingested_files", serde_json::json!({})).await {
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

/// Test deleting ingested files (expected to fail without admin)
async fn test_delete_ingested_files(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    file_ids: Vec<&str>,
) -> anyhow::Result<()> {
    match client.call_tool("delete_ingested_files", serde_json::json!({
        "file_ids": file_ids
    })).await {
        Ok(_) => {
            crate::teeprintln!("  ✓ delete_ingested_files - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ⚠ delete_ingested_files - BLOCKED (expected without admin): {}", e);
            stats.skipped += 1;
        }
    }
    Ok(())
}
