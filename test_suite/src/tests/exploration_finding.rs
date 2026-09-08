//! T1-10B-10 — migrated from `src/experience/exploration/finding.rs`
//! `#[cfg(test)] mod tests` (test_finding_new_and_promote).
//!
//! The src/ unit test exercised `ExplorationFinding::new()` + `.promote()`
//! directly on the struct. test_suite cannot import robot_brain source, so the
//! test is re-expressed through the public MCP surface that actually invokes
//! those methods:
//!   - `complete_exploration` calls `ExplorationFinding::new(...)` to build
//!     findings from the input.
//!   - `promote_finding` calls `f.promote()` on the matched finding.
//!   - `get_exploration_status` reports `findings[].promoted`, so promotion is
//!     observable end-to-end across the process boundary.
//!
//! Flow: start_exploration -> complete_exploration (with one finding) ->
//! get_exploration_status (assert finding exists, promoted=false) ->
//! promote_finding (assert promoted=true) -> get_exploration_status (assert
//! finding.promoted=true in durable state).

use crate::TestMcpClient;
use crate::TestStats;

/// Parse the tool result's text payload as JSON.
/// Handles both raw MCP responses (with content[0].text) and already-parsed
/// results returned by TestMcpClient::call_tool (which auto-parses the
/// content text into a JSON object).
fn payload_json(result: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    // Check for raw MCP response first
    if let Some(text) = result.pointer("/content/0/text").and_then(|v| v.as_str()) {
        return Ok(serde_json::from_str(text)?);
    }
    // Already parsed by call_tool — return as-is
    Ok(result.clone())
}

pub async fn run_exploration_finding_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    crate::teeprintln!("\n--- Exploration Finding new+promote (T1-10B-10) ---");

    // 1. Start an exploration to get an exploration_id.
    let start_result = client
        .call_tool(
            "start_exploration",
            serde_json::json!({
                "title": "T1-10B-10 finding promote probe",
                "purpose": "verify ExplorationFinding::new + promote via MCP"
            }),
        )
        .await;
    let exploration_id = match start_result {
        Ok(r) => match payload_json(&r) {
            Ok(v) => v
                .get("exploration_id")
                .and_then(|i| i.as_str())
                .map(|s| s.to_string()),
            Err(e) => {
                crate::teeprintln!("  [FAIL] start_exploration payload parse — {}", e);
                stats.failed += 1;
                return Ok(());
            }
        },
        Err(e) => {
            crate::teeprintln!("  [FAIL] start_exploration — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    let Some(exploration_id) = exploration_id else {
        crate::teeprintln!("  [FAIL] start_exploration — no exploration_id");
        stats.failed += 1;
        return Ok(());
    };
    crate::teeprintln!("  • exploration_id = {}", exploration_id);

    // 2. Complete the exploration with one finding. This invokes
    //    ExplorationFinding::new(id, description, confidence) inside
    //    execute_complete_exploration, exercising the constructor + clamp.
    let finding_desc = "Discovered a new pattern";
    let finding_conf = 0.85;
    let complete_result = client
        .call_tool(
            "complete_exploration",
            serde_json::json!({
                "exploration_id": exploration_id,
                "findings": [
                    { "description": finding_desc, "confidence": finding_conf }
                ]
            }),
        )
        .await;
    let finding_count = match complete_result {
        Ok(r) => match payload_json(&r) {
            Ok(v) => v.get("finding_count").and_then(|c| c.as_i64()).unwrap_or(0),
            Err(e) => {
                crate::teeprintln!("  [FAIL] complete_exploration payload parse — {}", e);
                stats.failed += 1;
                return Ok(());
            }
        },
        Err(e) => {
            crate::teeprintln!("  [FAIL] complete_exploration — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if finding_count == 1 {
        crate::teeprintln!(
            "  [OK] complete_exploration created 1 finding (ExplorationFinding::new)"
        );
        stats.passed += 1;
    } else {
        crate::teeprintln!(
            "  [FAIL] complete_exploration finding_count={} (expected 1)",
            finding_count
        );
        stats.failed += 1;
        return Ok(());
    }

    // 3. get_exploration_status: confirm the finding exists and is NOT promoted
    //    yet (new() sets promoted=false).
    let status_result = client
        .call_tool(
            "get_exploration_status",
            serde_json::json!({ "exploration_id": exploration_id }),
        )
        .await;
    let (finding_id, promoted_before) = match status_result {
        Ok(r) => match payload_json(&r) {
            Ok(v) => {
                let findings = v.get("findings").and_then(|f| f.as_array());
                match findings.and_then(|arr| arr.first()) {
                    Some(f) => {
                        let id = f
                            .get("id")
                            .and_then(|i| i.as_str())
                            .unwrap_or("")
                            .to_string();
                        let promoted = f.get("promoted").and_then(|p| p.as_bool()).unwrap_or(true);
                        (id, promoted)
                    }
                    None => {
                        crate::teeprintln!(
                            "  [FAIL] get_exploration_status — no findings in payload"
                        );
                        stats.failed += 1;
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                crate::teeprintln!("  [FAIL] get_exploration_status payload parse — {}", e);
                stats.failed += 1;
                return Ok(());
            }
        },
        Err(e) => {
            crate::teeprintln!("  [FAIL] get_exploration_status — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if finding_id.is_empty() {
        crate::teeprintln!("  [FAIL] get_exploration_status — finding has empty id");
        stats.failed += 1;
        return Ok(());
    }
    if !promoted_before {
        crate::teeprintln!(
            "  [OK] finding '{}' created with promoted=false (new())",
            finding_id
        );
        stats.passed += 1;
    } else {
        crate::teeprintln!(
            "  [FAIL] finding '{}' unexpectedly promoted right after creation",
            finding_id
        );
        stats.failed += 1;
        return Ok(());
    }

    // 4. promote_finding: invokes f.promote() on the matched finding.
    let promote_result = client
        .call_tool(
            "promote_finding",
            serde_json::json!({
                "exploration_id": exploration_id,
                "finding_id": finding_id
            }),
        )
        .await;
    let promote_ok = match promote_result {
        Ok(r) => match payload_json(&r) {
            Ok(v) => v.get("promoted").and_then(|p| p.as_bool()).unwrap_or(false),
            Err(e) => {
                crate::teeprintln!("  [FAIL] promote_finding payload parse — {}", e);
                stats.failed += 1;
                return Ok(());
            }
        },
        Err(e) => {
            crate::teeprintln!("  [FAIL] promote_finding — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if promote_ok {
        crate::teeprintln!("  [OK] promote_finding reported promoted=true (f.promote())",);
        stats.passed += 1;
    } else {
        crate::teeprintln!("  [FAIL] promote_finding did not report promoted=true");
        stats.failed += 1;
        return Ok(());
    }

    // 5. get_exploration_status again: confirm promoted persisted as true.
    let status2 = client
        .call_tool(
            "get_exploration_status",
            serde_json::json!({ "exploration_id": exploration_id }),
        )
        .await;
    let promoted_after = match status2 {
        Ok(r) => match payload_json(&r) {
            Ok(v) => v
                .get("findings")
                .and_then(|f| f.as_array())
                .and_then(|arr| arr.first())
                .and_then(|f| f.get("promoted"))
                .and_then(|p| p.as_bool())
                .unwrap_or(false),
            Err(e) => {
                crate::teeprintln!("  [FAIL] get_exploration_status(2) payload parse — {}", e);
                stats.failed += 1;
                return Ok(());
            }
        },
        Err(e) => {
            crate::teeprintln!("  [FAIL] get_exploration_status(2) — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if promoted_after {
        crate::teeprintln!("  [OK] finding promoted=true persisted in exploration status");
        stats.passed += 1;
    } else {
        crate::teeprintln!("  [FAIL] finding promoted=false after promote_finding");
        stats.failed += 1;
    }

    Ok(())
}
