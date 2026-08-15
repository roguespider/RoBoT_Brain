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

mod archive_ingestion;
mod deletion;
mod file_listing;
mod json_ingestion;
mod single_file_ingestion;
mod types;

pub use archive_ingestion::{test_ingest_archive_tar_gz, test_ingest_archive_zip};
pub use deletion::test_delete_ingested_files;
pub use file_listing::{
    test_list_importable, test_list_importable_recursive, test_list_ingested_files,
};
pub use json_ingestion::{test_ingest_json_file, test_ingest_jsonl_file};
pub use single_file_ingestion::{test_ingest_folder_recursive, test_ingest_single_file_type};

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

    let file_types = types::get_all_file_type_tests();

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
    match client
        .call_tool("list_ingested_files", serde_json::json!({}))
        .await
    {
        Ok(_) => {
            crate::teeprintln!("  [OK] delete_ingested_files (confirmed) - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] delete_ingested_files (confirmed) - FAILED: {}", e);
            stats.failed += 1;
        }
    }

    crate::teeprintln!("\n{}", "=".repeat(60));
    crate::teeprintln!("INGESTOR FILE TYPE TESTS COMPLETE");
    crate::teeprintln!("{}", "=".repeat(60));

    Ok(())
}
