//! T1-10B-06 — migrated from `src/memory/retrieval.rs` `#[cfg(test)] mod tests`
//! (test_retrieve_working, test_unified_retrieve). Tests 2+4 reclassified to
//! Group B (see PLAN.md note).
//!
//! The src/ unit tests exercised `MemoryRetrieval::get_from_working` and
//! `retrieve` (unified working+permanent) directly. test_suite cannot import
//! robot_brain source, so the behavior is re-expressed through the public MCP
//! surface that invokes those exact functions:
//!   - `search_memory` calls `memory_retrieval.retrieve(&query)`, which calls
//!     `get_from_working` + `get_from_permanent` and returns `results[]` with
//!     `content` + `relevance_score`.
//!
//! MCP-reachable (migrated here):
//!   - test_retrieve_working: store_memory → search → content found in results
//!   - test_unified_retrieve: store 2 items → search → 2 results returned
//!
//! Group B (internal-only, LEAVE as Rust unit test):
//!
//!   - test_retrieve_permanent: store_memory only writes to Working layer;
//!     PermanentMemory's in-memory cache isn't populated by any MCP tool, so
//!     get_from_permanent can't be exercised via MCP.
//!   - test_confidence_filtering: retrieve_with_query(min_confidence) is never
//!     called by an MCP tool (search_memory uses retrieve() with no
//!     confidence filter).

use crate::TestMcpClient;
use crate::TestStats;

/// Parse the JSON payload from a tool result's content[0].text.
/// Handles both raw MCP responses and already-parsed results.
fn payload_json(result: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    if let Some(text) = result.pointer("/content/0/text").and_then(|v| v.as_str()) {
        return Ok(serde_json::from_str(text)?);
    }
    Ok(result.clone())
}

fn unique_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}

/// Store a memory and return its content marker for later search verification.
async fn store_memory(client: &mut TestMcpClient, content: &str) -> anyhow::Result<()> {
    client
        .call_tool(
            "store_memory",
            serde_json::json!({
                "content": content,
                "memory_type": "note",
                "confidence": 0.8,
                "importance": 0.7
            }),
        )
        .await?;
    Ok(())
}

pub async fn run_memory_retrieval_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    crate::teeprintln!("\n--- Memory retrieval working+unified (T1-10B-06) ---");

    // --- test_retrieve_working ---
    // Store an item with a unique marker, search for it, verify it's in results.
    let suffix = unique_suffix();
    let marker = format!("PythonIsGreatLang-{}", suffix);
    store_memory(client, &marker).await?;

    let search_result = client
        .call_tool(
            "search_memory",
            serde_json::json!({ "query": marker.clone(), "limit": 10 }),
        )
        .await;
    let found = match search_result {
        Ok(r) => payload_json(&r)
            .ok()
            .and_then(|v| {
                v.get("results").and_then(|r| r.as_array()).map(|arr| {
                    arr.iter().any(|item| {
                        item.get("content")
                            .and_then(|c| c.as_str())
                            .is_some_and(|c| c.contains(&marker))
                    })
                })
            })
            .unwrap_or(false),
        Err(e) => {
            crate::teeprintln!("  [FAIL] search_memory(working) — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if found {
        crate::teeprintln!(
            "  [OK] retrieve working: stored item found in search results (get_from_working via retrieve)"
        );
        stats.passed += 1;
    } else {
        crate::teeprintln!(
            "  [FAIL] retrieve working: marker not found in results (marker={})",
            marker
        );
        stats.failed += 1;
        return Ok(());
    }

    // --- test_unified_retrieve ---
    // Store 2 distinct items with a shared marker, search for the shared marker,
    // verify both are returned (retrieve unions working+permanent results).
    let suffix2 = unique_suffix();
    let shared = format!("SharedCtx-{}", suffix2);
    let item1 = format!("{}-alpha", shared);
    let item2 = format!("{}-beta", shared);
    store_memory(client, &item1).await?;
    store_memory(client, &item2).await?;

    let unified_result = client
        .call_tool(
            "search_memory",
            serde_json::json!({ "query": shared.clone(), "limit": 20 }),
        )
        .await;
    let (found_alpha, found_beta) = match unified_result {
        Ok(r) => {
            let v = payload_json(&r).ok().unwrap_or_default();
            let results = v
                .get("results")
                .cloned()
                .unwrap_or(serde_json::Value::Array(vec![]));
            let fa = results
                .as_array()
                .map(|arr| {
                    arr.iter().any(|item| {
                        item.get("content")
                            .and_then(|c| c.as_str())
                            .is_some_and(|c| c.contains(&item1))
                    })
                })
                .unwrap_or(false);
            let fb = results
                .as_array()
                .map(|arr| {
                    arr.iter().any(|item| {
                        item.get("content")
                            .and_then(|c| c.as_str())
                            .is_some_and(|c| c.contains(&item2))
                    })
                })
                .unwrap_or(false);
            (fa, fb)
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] search_memory(unified) — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if found_alpha && found_beta {
        crate::teeprintln!(
            "  [OK] retrieve unified: both items found in search results (retrieve unions results)"
        );
        stats.passed += 1;
    } else {
        crate::teeprintln!(
            "  [FAIL] retrieve unified: alpha={}, beta={} (expected both)",
            found_alpha,
            found_beta
        );
        stats.failed += 1;
    }

    Ok(())
}
