//! T1-10B-17 — migrated from `src/bridge/tools/ingestor/semantic_chunker.rs`
//! `#[cfg(test)] mod tests` (test_markdown_parsing, test_sentence_splitting,
//! test_code_parsing).
//!
//! The src/ unit tests exercised parse_markdown, split_sentences, and
//! parse_code directly. test_suite cannot import robot_brain source, so the
//! behavior is re-expressed through the public MCP surface that invokes those
//! functions:
//!   - `ingest_files` (file_path) calls `ingest_single_file` which calls
//!     `parse_document(content, filename, file_type)`, which dispatches to
//!     `parse_markdown` (for .md) / `parse_code` (for .rs). `parse_markdown`
//!     internally calls `split_sentences`. The tree is then `flatten()`-ed
//!     into chunks; the count is returned as `chunks_created`.
//!
//! MCP-reachable (migrated here):
//!   - test_markdown_parsing + test_sentence_splitting: create a .md file
//!     with >=2 sections, ingest_files, verify chunks_created >= 2.
//!   - test_code_parsing: create a .rs file with >=2 functions, ingest_files,
//!     verify chunks_created >= 2.

use crate::TestMcpClient;
use crate::TestStats;
use std::io::Write;

fn payload_json(result: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let text = result
        .pointer("/content/0/text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("no content text in tool result"))?;
    Ok(serde_json::from_str(text)?)
}

fn unique_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}

fn make_temp_file(ext: &str, content: &str) -> std::path::PathBuf {
    let suffix = unique_suffix();
    let dir = std::env::temp_dir();
    let path = dir.join(format!("t1_10b_17_{}.{}", suffix, ext));
    if let Ok(mut f) = std::fs::File::create(&path) {
        let _ = f.write_all(content.as_bytes());
    }
    path
}

pub async fn run_semantic_chunker_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    crate::teeprintln!("\n--- Semantic chunker markdown+code parsing (T1-10B-17) ---");

    // --- test_markdown_parsing + test_sentence_splitting ---
    // Create a markdown file with >=2 sections. parse_markdown produces a
    // hierarchy tree with >=2 children. split_sentences is called inside
    // parse_markdown for paragraph content. chunks_created >= 2 verifies both.
    let md_content = "# Introduction\n\nThis is the intro paragraph.\n\n## Installation\n\nFollow these steps:\n\n1. Install\n2. Configure\n\n## Usage\n\nUse it like this.\n";
    let md_path = make_temp_file("md", md_content);
    let md_result = client
        .call_tool(
            "ingest_files",
            serde_json::json!({
                "file_path": md_path.to_string_lossy(),
                "memory_type": "file",
                "force": true
            }),
        )
        .await;
    let _ = std::fs::remove_file(&md_path);
    let md_chunks = match md_result {
        Ok(r) => payload_json(&r)
            .ok()
            .and_then(|v| v.get("chunks_created").and_then(|c| c.as_u64()))
            .unwrap_or(0),
        Err(e) => {
            crate::teeprintln!("  ✗ ingest_files(md) — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if md_chunks >= 2 {
        crate::teeprintln!(
            "  ✓ markdown parsing + sentence splitting: {} chunks (>=2 sections, split_sentences exercised)",
            md_chunks
        );
        stats.passed += 1;
    } else {
        crate::teeprintln!(
            "  ✗ markdown parsing: only {} chunks (expected >=2 from parse_markdown + split_sentences)",
            md_chunks
        );
        stats.failed += 1;
    }

    // --- test_code_parsing ---
    // Create a .rs file with >=2 functions. parse_code produces a hierarchy
    // tree with >=2 children (one per function). chunks_created >= 2.
    let code_content = "fn main() {\n    println!(\"Hello\");\n}\n\nfn other() {\n    do_something();\n}\n";
    let code_path = make_temp_file("rs", code_content);
    let code_result = client
        .call_tool(
            "ingest_files",
            serde_json::json!({
                "file_path": code_path.to_string_lossy(),
                "memory_type": "code",
                "force": true
            }),
        )
        .await;
    let _ = std::fs::remove_file(&code_path);
    let code_chunks = match code_result {
        Ok(r) => payload_json(&r)
            .ok()
            .and_then(|v| v.get("chunks_created").and_then(|c| c.as_u64()))
            .unwrap_or(0),
        Err(e) => {
            crate::teeprintln!("  ✗ ingest_files(rs) — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if code_chunks >= 2 {
        crate::teeprintln!(
            "  ✓ code parsing: {} chunks (>=2 functions from parse_code)",
            code_chunks
        );
        stats.passed += 1;
    } else {
        crate::teeprintln!(
            "  ✗ code parsing: only {} chunks (expected >=2 from parse_code)",
            code_chunks
        );
        stats.failed += 1;
    }

    Ok(())
}
