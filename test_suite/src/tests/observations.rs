//! T1-10B-11 — migrated from `src/database/queries/observations.rs`
//! `#[cfg(test)] mod tests` (test_link_observation_to_experience).
//!
//! The src/ test verified insert_observation, link_observation_to_experience
//! and get_observation against an in-memory SQLite DB. test_suite cannot
//! import robot_brain source, so the record/retrieve behavior is re-expressed
//! through the public MCP surface instead:
//!
//!   - `record_observation` calls `insert_observation` under the hood.
//!   - `list_observations` calls `list_observations` (queries.rs) and returns
//!     content/context/observation_type/id, so a recorded observation is
//!     observable across the process boundary.
//!
//! Note: `link_observation_to_experience` (the original test's focus) has NO
//! MCP surface and no production callers — it was `#[cfg(test)]`-only dead
//! code. It is deleted from observations.rs along with the test module (and
//! `get_observation`, which only `link_observation_to_experience` called).
//! The record+list behavior — the genuinely MCP-reachable part — is what this
//! test now covers.

use crate::TestMcpClient;
use crate::TestStats;

/// Extract the JSON payload carried in `result.content[0].text`.
fn payload_json(result: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let text = result
        .pointer("/content/0/text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("no content text in tool result"))?;
    Ok(serde_json::from_str(text)?)
}

pub async fn run_observations_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    crate::teeprintln!("\n--- Observations record+list (T1-10B-11) ---");

    // Record a unique observation so we can find it among any pre-existing ones.
    let marker = format!("T1-10B-11-marker-{}", uuid_marker());
    let obs_content = format!("{} observed a new pattern", marker);
    let obs_context = "test context";
    let obs_type = "pattern";

    let record_result = client
        .call_tool(
            "record_observation",
            serde_json::json!({
                "content": obs_content,
                "context": obs_context,
                "observation_type": obs_type
            }),
        )
        .await;
    let recorded_ok = match record_result {
        Ok(r) => match payload_json(&r) {
            Ok(v) => {
                v.get("status").and_then(|s| s.as_str()) == Some("observation_recorded")
                    || v.pointer("/observation/id").is_some()
            }
            Err(e) => {
                crate::teeprintln!("  [FAIL] record_observation payload parse — {}", e);
                stats.failed += 1;
                return Ok(());
            }
        },
        Err(e) => {
            crate::teeprintln!("  [FAIL] record_observation — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if recorded_ok {
        crate::teeprintln!("  [OK] record_observation persisted (insert_observation)");
        stats.passed += 1;
    } else {
        crate::teeprintln!("  [FAIL] record_observation did not report success/id");
        stats.failed += 1;
        return Ok(());
    }

    // list_observations and confirm the recorded observation appears with the
    // right content/type (exercises list_observations retrieval path).
    let list_result = client
        .call_tool(
            "list_observations",
            serde_json::json!({ "limit": 50, "observation_type": obs_type }),
        )
        .await;
    let found = match list_result {
        Ok(r) => match payload_json(&r) {
            Ok(v) => v
                .get("observations")
                .and_then(|o| o.as_array())
                .map(|arr| {
                    arr.iter().any(|o| {
                        o.get("content")
                            .and_then(|c| c.as_str())
                            .is_some_and(|c| c.contains(&marker))
                            && o.get("observation_type")
                                .and_then(|t| t.as_str())
                                .is_some_and(|t| t == obs_type)
                    })
                })
                .unwrap_or(false),
            Err(e) => {
                crate::teeprintln!("  [FAIL] list_observations payload parse — {}", e);
                stats.failed += 1;
                return Ok(());
            }
        },
        Err(e) => {
            crate::teeprintln!("  [FAIL] list_observations — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if found {
        crate::teeprintln!(
            "  [OK] list_observations returned the recorded observation (content+type match)"
        );
        stats.passed += 1;
    } else {
        crate::teeprintln!(
            "  [FAIL] list_observations did not return the recorded observation (marker={})",
            marker
        );
        stats.failed += 1;
    }

    Ok(())
}

/// A short unique suffix to make the observation content findable.
fn uuid_marker() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}
