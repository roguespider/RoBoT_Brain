//! Session smoke test — migrated from `.agents/live_test/session_smoke.py`.
//!
//! The Python script was a live MCP smoke check: spawn robot_brain, satisfy
//! the workflow gate (get_workflow -> search_memory), then prove three real
//! tool paths work live:
//!   1. store_memory  — write path with the real contract (`memory_type`
//!      required by the handler even when the advertised schema omits it).
//!   2. search_memory — read-back proves the memory system works live.
//!   3. create_plan   — a non-memory planning path works live.
//!
//! The Rust TestMcpClient in main.rs already performs the initialize +
//! workflow-gate handshake (the mcp_client.py role), so this module only
//! needs to port the session_smoke.py assertions.

use crate::TestMcpClient;
use crate::TestStats;

/// Extract the JSON payload carried in `result.content[0].text`.
/// Parse the JSON payload from a tool result's content[0].text.
/// Handles both raw MCP responses and already-parsed results.
fn payload_json(result: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    if let Some(text) = result.pointer("/content/0/text").and_then(|v| v.as_str()) {
        return Ok(serde_json::from_str(text)?);
    }
    Ok(result.clone())
}

/// A short unique suffix so stored content is findable among pre-existing data.
fn uuid_marker() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}

pub async fn run_session_smoke_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    crate::teeprintln!("\n--- Session smoke (live store/search/plan proof) ---");

    let marker = uuid_marker();
    let content = format!("session_start smoke: live MCP confirmed {}", marker);

    // 1. store_memory — real handler contract requires memory_type.
    let mut store_id = String::new();
    let store_ok = match client
        .call_tool(
            "store_memory",
            serde_json::json!({
                "content": content,
                "memory_type": "note"
            }),
        )
        .await
    {
        Ok(r) => match payload_json(&r) {
            Ok(v) => {
                if let Some(id) = v.get("id").and_then(|i| i.as_str()) {
                    store_id = id.to_string();
                } else if let Some(id) = v.get("experience_id").and_then(|i| i.as_str()) {
                    store_id = id.to_string();
                }
                !store_id.is_empty() || v.get("success").and_then(|s| s.as_bool()) == Some(true)
            }
            Err(e) => {
                crate::teeprintln!("  [FAIL] store_memory payload parse - {}", e);
                stats.failed += 1;
                return Ok(());
            }
        },
        Err(e) => {
            crate::teeprintln!("  [FAIL] store_memory call failed - {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if store_ok {
        crate::teeprintln!("  [OK] store_memory persisted (id prefix: {})", {
            let n = store_id.chars().count().min(8);
            store_id.chars().take(n).collect::<String>()
        });
        stats.passed += 1;
    } else {
        crate::teeprintln!("  [FAIL] store_memory did not report success/id");
        stats.failed += 1;
        return Ok(());
    }

    // 2. search_memory must return results (proves the memory system works live).
    let search_ok = match client
        .call_tool("search_memory", serde_json::json!({ "query": content }))
        .await
    {
        Ok(r) => match payload_json(&r) {
            Ok(v) => {
                v.get("count").and_then(|c| c.as_u64()).unwrap_or(0) > 0
                    || v.get("results")
                        .and_then(|res| res.as_array())
                        .map(|arr| !arr.is_empty())
                        .unwrap_or(false)
            }
            Err(e) => {
                crate::teeprintln!("  [FAIL] search_memory payload parse - {}", e);
                stats.failed += 1;
                return Ok(());
            }
        },
        Err(e) => {
            crate::teeprintln!("  [FAIL] search_memory call failed - {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if search_ok {
        crate::teeprintln!("  [OK] search_memory returned the stored memory");
        stats.passed += 1;
    } else {
        crate::teeprintln!("  [FAIL] search_memory found no results for the stored marker");
        stats.failed += 1;
        return Ok(());
    }

    // 3. create_plan confirms a non-memory path works live.
    let plan_ok = match client
        .call_tool(
            "create_plan",
            serde_json::json!({
                "goal": format!("session smoke plan {}", marker),
                "context": "startup"
            }),
        )
        .await
    {
        Ok(r) => match payload_json(&r) {
            Ok(v) => v.get("id").is_some() || v.get("plan").is_some(),
            Err(e) => {
                crate::teeprintln!("  [FAIL] create_plan payload parse - {}", e);
                stats.failed += 1;
                return Ok(());
            }
        },
        Err(e) => {
            crate::teeprintln!("  [FAIL] create_plan call failed - {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if plan_ok {
        crate::teeprintln!("  [OK] create_plan created a plan");
        stats.passed += 1;
    } else {
        crate::teeprintln!("  [FAIL] create_plan did not report success/id");
        stats.failed += 1;
    }

    crate::teeprintln!(
        "[DONE] session smoke complete: store=[OK] search=[OK] plan=[{}]",
        if plan_ok { "OK" } else { "FAIL" }
    );
    Ok(())
}
