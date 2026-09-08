//! T1-10B-05 — migrated from `src/knowledge/query.rs` `#[cfg(test)] mod tests`
//! (test_text_filter, test_confidence_filter, test_ranking).
//!
//! The src/ unit tests exercised `apply_query` (text filter, confidence
//! filter) and `rank_items` (ranking by relevance) directly. test_suite cannot
//! import robot_brain source, so the behavior is re-expressed through the
//! public MCP surface that invokes those exact functions:
//!   - `query_knowledge` calls `apply_query(&all_items, &query)` then
//!     `rank_items(filtered, &query)`, and returns `items[]` + `best_match`.
//!
//! Flow: add distinct items, then query_knowledge with text / min_confidence
//! filters and verify items[] membership + best_match ranking.

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

fn items_contains(items: &serde_json::Value, marker: &str) -> bool {
    items
        .as_array()
        .map(|arr| {
            arr.iter().any(|item| {
                item.get("statement")
                    .and_then(|s| s.as_str())
                    .is_some_and(|s| s.contains(marker))
            })
        })
        .unwrap_or(false)
}

fn unique_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}

pub async fn run_knowledge_query_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    crate::teeprintln!("\n--- Knowledge query text/confidence/ranking (T1-10B-05) ---");

    // --- test_text_filter ---
    // Add two items with distinct text, query for one text, verify only it matches.
    let suffix = unique_suffix();
    let rust_stmt = format!("RustIsFast-{}", suffix);
    let python_stmt = format!("PythonIsEasy-{}", suffix);
    let add_rust = client
        .call_tool(
            "add_knowledge",
            serde_json::json!({
                "statement": rust_stmt,
                "confidence": 0.8,
                "knowledge_type": "fact",
                "source": "user"
            }),
        )
        .await;
    let add_python = client
        .call_tool(
            "add_knowledge",
            serde_json::json!({
                "statement": python_stmt,
                "confidence": 0.7,
                "knowledge_type": "fact",
                "source": "user"
            }),
        )
        .await;
    if let Err(e) = add_rust {
        crate::teeprintln!("  [FAIL] add_knowledge(rust) — {}", e);
        stats.failed += 1;
        return Ok(());
    }
    if let Err(e) = add_python {
        crate::teeprintln!("  [FAIL] add_knowledge(python) — {}", e);
        stats.failed += 1;
        return Ok(());
    }

    // Query with text filter for the rust marker.
    let text_query = client
        .call_tool(
            "query_knowledge",
            serde_json::json!({
                "query": format!("RustIsFast-{}", suffix),
                "min_confidence": 0.0
            }),
        )
        .await;
    let (text_rust_found, text_python_absent) = match text_query {
        Ok(r) => {
            let v = payload_json(&r).ok().unwrap_or_default();
            let items = v
                .get("items")
                .cloned()
                .unwrap_or(serde_json::Value::Array(vec![]));
            let rust_found = items_contains(&items, &format!("RustIsFast-{}", suffix));
            let python_absent = !items_contains(&items, &format!("PythonIsEasy-{}", suffix));
            (rust_found, python_absent)
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] query_knowledge(text filter) — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if text_rust_found && text_python_absent {
        crate::teeprintln!(
            "  [OK] text filter: rust item found, python item excluded (apply_query text)"
        );
        stats.passed += 1;
    } else {
        crate::teeprintln!(
            "  [FAIL] text filter: rust_found={}, python_absent={}",
            text_rust_found,
            text_python_absent
        );
        stats.failed += 1;
        return Ok(());
    }

    // --- test_confidence_filter ---
    // Add a high-conf and low-conf item with a shared marker, query with
    // min_confidence=0.7 -> only high-conf should match.
    let conf_suffix = unique_suffix();
    let high_stmt = format!("HighConfShared-{}-item", conf_suffix);
    let low_stmt = format!("LowConfShared-{}-item", conf_suffix);
    let add_high = client
        .call_tool(
            "add_knowledge",
            serde_json::json!({
                "statement": high_stmt,
                "confidence": 0.9,
                "knowledge_type": "fact",
                "source": "user"
            }),
        )
        .await;
    let add_low = client
        .call_tool(
            "add_knowledge",
            serde_json::json!({
                "statement": low_stmt,
                "confidence": 0.3,
                "knowledge_type": "fact",
                "source": "user"
            }),
        )
        .await;
    if let Err(e) = add_high {
        crate::teeprintln!("  [FAIL] add_knowledge(high-conf) — {}", e);
        stats.failed += 1;
        return Ok(());
    }
    if let Err(e) = add_low {
        crate::teeprintln!("  [FAIL] add_knowledge(low-conf) — {}", e);
        stats.failed += 1;
        return Ok(());
    }

    let conf_query = client
        .call_tool(
            "query_knowledge",
            serde_json::json!({
                "query": format!("Shared-{}-item", conf_suffix),
                "min_confidence": 0.7
            }),
        )
        .await;
    let (high_found, low_absent) = match conf_query {
        Ok(r) => {
            let v = payload_json(&r).ok().unwrap_or_default();
            let items = v
                .get("items")
                .cloned()
                .unwrap_or(serde_json::Value::Array(vec![]));
            let hf = items_contains(&items, &format!("HighConfShared-{}", conf_suffix));
            let la = !items_contains(&items, &format!("LowConfShared-{}", conf_suffix));
            (hf, la)
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] query_knowledge(confidence filter) — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if high_found && low_absent {
        crate::teeprintln!(
            "  [OK] confidence filter: high-conf found, low-conf excluded (apply_query min_confidence)"
        );
        stats.passed += 1;
    } else {
        crate::teeprintln!(
            "  [FAIL] confidence filter: high_found={}, low_absent={}",
            high_found,
            low_absent
        );
        stats.failed += 1;
        return Ok(());
    }

    // --- test_ranking ---
    // Both items match the text "match". The high-conf one (0.9) should rank
    // first (best_match), since rank_items sorts by relevance score (which
    // starts from overall_confidence).
    let rank_suffix = unique_suffix();
    let high_rank = format!("HighMatchRank-{}-item", rank_suffix);
    let low_rank = format!("LowMatchRank-{}-item", rank_suffix);
    let add_hr = client
        .call_tool(
            "add_knowledge",
            serde_json::json!({
                "statement": high_rank,
                "confidence": 0.9,
                "knowledge_type": "fact",
                "source": "user"
            }),
        )
        .await;
    let add_lr = client
        .call_tool(
            "add_knowledge",
            serde_json::json!({
                "statement": low_rank,
                "confidence": 0.3,
                "knowledge_type": "fact",
                "source": "user"
            }),
        )
        .await;
    if let Err(e) = add_hr {
        crate::teeprintln!("  [FAIL] add_knowledge(high-rank) — {}", e);
        stats.failed += 1;
        return Ok(());
    }
    if let Err(e) = add_lr {
        crate::teeprintln!("  [FAIL] add_knowledge(low-rank) — {}", e);
        stats.failed += 1;
        return Ok(());
    }

    let rank_query = client
        .call_tool(
            "query_knowledge",
            serde_json::json!({
                "query": format!("MatchRank-{}-item", rank_suffix),
                "min_confidence": 0.2
            }),
        )
        .await;
    let high_is_best = match rank_query {
        Ok(r) => {
            let v = payload_json(&r).ok().unwrap_or_default();
            v.get("best_match")
                .and_then(|bm| bm.get("statement"))
                .and_then(|s| s.as_str())
                .is_some_and(|s| s.contains(&format!("HighMatchRank-{}", rank_suffix)))
        }
        Err(e) => {
            crate::teeprintln!("  [FAIL] query_knowledge(ranking) — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if high_is_best {
        crate::teeprintln!(
            "  [OK] ranking: high-conf(0.9) is best_match over low-conf(0.3) (rank_items)"
        );
        stats.passed += 1;
    } else {
        crate::teeprintln!(
            "  [FAIL] ranking: high-conf not best_match (expected HighMatchRank first)"
        );
        stats.failed += 1;
    }

    Ok(())
}
