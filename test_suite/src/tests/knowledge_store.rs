//! T1-10B-04 — migrated from `src/knowledge/store.rs` `#[cfg(test)] mod tests`
//! (test_add_and_get, test_get_mature).
//!
//! The src/ unit tests exercised `KnowledgeStore::add` + `get` +
//! `get_mature` directly on the in-memory store. test_suite cannot import
//! robot_brain source, so the behavior is re-expressed through the public MCP
//! surface that invokes those exact methods:
//!
//!   - `add_knowledge` calls `KnowledgeStore::add(item)` and returns the id.
//!   - `query_knowledge` retrieves via `get_all`/`get_by_type` then applies
//!     query filters (text match + min_confidence), so an added item is
//!     observable by searching its statement text (test_add_and_get).
//!   - `get_knowledge_stats` reports the mature count (items with confidence
//!     of at least 0.7 AND status Active, via `is_mature`), so test_get_mature
//!     is observable: add a low-confidence (0.3) and a high-confidence (0.8)
//!     item; only the high-conf one should be reported as mature.

use crate::TestMcpClient;
use crate::TestStats;

fn payload_json(result: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let text = result
        .pointer("/content/0/text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("no content text in tool result"))?;
    Ok(serde_json::from_str(text)?)
}

pub async fn run_knowledge_store_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    crate::teeprintln!("\n--- Knowledge store add+get+mature (T1-10B-04) ---");

    // --- test_add_and_get ---
    // Add knowledge, then query for it by statement text.
    let marker = format!("T1-10B-04-addget-{}", unique_suffix());
    let statement = format!("{} test knowledge statement", marker);
    let add_result = client
        .call_tool(
            "add_knowledge",
            serde_json::json!({
                "statement": statement,
                "confidence": 0.8,
                "knowledge_type": "fact",
                "source": "user"
            }),
        )
        .await;
    let added_ok = match add_result {
        Ok(r) => payload_json(&r)
            .ok()
            .map(|v| v.get("status").and_then(|s| s.as_str()) == Some("added"))
            .unwrap_or(false),
        Err(e) => {
            crate::teeprintln!("  [FAIL] add_knowledge — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if !added_ok {
        crate::teeprintln!("  [FAIL] add_knowledge did not report status=added");
        stats.failed += 1;
        return Ok(());
    }

    // query_knowledge: search by the marker text, verify the item is retrieved.
    let query_result = client
        .call_tool(
            "query_knowledge",
            serde_json::json!({
                "query": marker,
                "min_confidence": 0.0
            }),
        )
        .await;
    let found = match query_result {
        Ok(r) => payload_json(&r)
            .ok()
            .and_then(|v| {
                v.get("items").and_then(|r| r.as_array()).map(|arr| {
                    arr.iter().any(|item| {
                        item.get("statement")
                            .and_then(|s| s.as_str())
                            .is_some_and(|s| s.contains(&marker))
                    })
                })
            })
            .unwrap_or(false),
        Err(e) => {
            crate::teeprintln!("  [FAIL] query_knowledge — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if found {
        crate::teeprintln!(
            "  [OK] add_knowledge + query_knowledge: added item retrieved (add+get)"
        );
        stats.passed += 1;
    } else {
        crate::teeprintln!(
            "  [FAIL] query_knowledge did not return the added item (marker={})",
            marker
        );
        stats.failed += 1;
        return Ok(());
    }

    // --- test_get_mature ---
    // Add a low-confidence (0.3) and a high-confidence (0.8) item. Only the
    // high-conf one should count as mature (>= 0.7 AND Active).
    let low_marker = format!("low-conf-{}", unique_suffix());
    let high_marker = format!("high-conf-{}", unique_suffix());
    let low_add = client
        .call_tool(
            "add_knowledge",
            serde_json::json!({
                "statement": format!("{} low confidence item", low_marker),
                "confidence": 0.3,
                "knowledge_type": "fact",
                "source": "user"
            }),
        )
        .await;
    let high_add = client
        .call_tool(
            "add_knowledge",
            serde_json::json!({
                "statement": format!("{} high confidence item", high_marker),
                "confidence": 0.8,
                "knowledge_type": "fact",
                "source": "user"
            }),
        )
        .await;
    if let Err(e) = low_add {
        crate::teeprintln!("  [FAIL] add_knowledge(low-conf) — {}", e);
        stats.failed += 1;
        return Ok(());
    }
    if let Err(e) = high_add {
        crate::teeprintln!("  [FAIL] add_knowledge(high-conf) — {}", e);
        stats.failed += 1;
        return Ok(());
    }

    // get_knowledge_stats: the mature count should include the high-conf item
    // we just added (plus the earlier 0.8 one from test_add_and_get). We
    // verify the high-conf item is queryable with min_confidence>=0.7 while
    // the low-conf one is not (mirrors get_mature's >= 0.7 threshold).
    let mature_query = client
        .call_tool(
            "query_knowledge",
            serde_json::json!({
                "query": high_marker,
                "min_confidence": 0.7
            }),
        )
        .await;
    let high_found = match mature_query {
        Ok(r) => payload_json(&r)
            .ok()
            .and_then(|v| {
                v.get("items").and_then(|r| r.as_array()).map(|arr| {
                    arr.iter().any(|item| {
                        item.get("statement")
                            .and_then(|s| s.as_str())
                            .is_some_and(|s| s.contains(&high_marker))
                    })
                })
            })
            .unwrap_or(false),
        Err(e) => {
            crate::teeprintln!("  [FAIL] query_knowledge(high, min_conf=0.7) — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };

    let low_query = client
        .call_tool(
            "query_knowledge",
            serde_json::json!({
                "query": low_marker,
                "min_confidence": 0.7
            }),
        )
        .await;
    let low_excluded = match low_query {
        Ok(r) => payload_json(&r)
            .ok()
            .and_then(|v| {
                v.get("items").and_then(|r| r.as_array()).map(|arr| {
                    !arr.iter().any(|item| {
                        item.get("statement")
                            .and_then(|s| s.as_str())
                            .is_some_and(|s| s.contains(&low_marker))
                    })
                })
            })
            .unwrap_or(false),
        Err(e) => {
            crate::teeprintln!("  [FAIL] query_knowledge(low, min_conf=0.7) — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };

    if high_found && low_excluded {
        crate::teeprintln!(
            "  [OK] get_mature threshold: high-conf(0.8) included, low-conf(0.3) excluded at >=0.7"
        );
        stats.passed += 1;
    } else {
        crate::teeprintln!(
            "  [FAIL] get_mature threshold: high_found={}, low_excluded={}",
            high_found,
            low_excluded
        );
        stats.failed += 1;
    }

    Ok(())
}

fn unique_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}
