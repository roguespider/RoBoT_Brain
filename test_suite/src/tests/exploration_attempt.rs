//! T1-10B-09 — migrated from `src/experience/exploration/attempt.rs`
//! `#[cfg(test)] mod tests` (test_attempt_builder, test_attempt_failure).
//!
//! The src/ unit tests exercised `ExplorationAttempt::new` +
//! `.with_expected_result` + `.with_actual_result` and the success/failure
//! comparison logic directly on the struct. test_suite cannot import
//! robot_brain source, so the behavior is re-expressed through the public MCP
//! surface that invokes those exact methods:
//!   - `record_attempt` calls `ExplorationAttempt::new`, then
//!     `.with_expected_result` and `.with_actual_result` (the builder methods
//!     under test), which set `success` by comparing expected vs actual.
//!   - `get_exploration_status` reports `attempts[].success`,
//!     `expected_result`, and `actual_result`, so both the success and failure
//!     branches are observable end-to-end.
//!
//! Flow: start_exploration -> record_attempt (expected==actual) ->
//! record_attempt (expected!=actual) -> get_exploration_status -> assert
//! attempt[0].success==true (test_attempt_builder) and
//! attempt[1].success==false (test_attempt_failure).

use crate::TestMcpClient;
use crate::TestStats;

fn payload_json(result: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let text = result
        .pointer("/content/0/text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("no content text in tool result"))?;
    Ok(serde_json::from_str(text)?)
}

pub async fn run_exploration_attempt_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    crate::teeprintln!("\n--- Exploration Attempt builder+success/failure (T1-10B-09) ---");

    // Start an exploration to host the attempts.
    let start_result = client
        .call_tool(
            "start_exploration",
            serde_json::json!({
                "title": "T1-10B-09 attempt probe",
                "purpose": "verify ExplorationAttempt new+with_expected/actual_result"
            }),
        )
        .await;
    let exploration_id = match start_result {
        Ok(r) => payload_json(&r)
            .ok()
            .and_then(|v| {
                v.get("exploration_id")
                    .and_then(|i| i.as_str())
                    .map(|s| s.to_string())
            }),
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

    // Attempt 1: expected == actual -> success=true (test_attempt_builder).
    let expected_ok = "Problem solved";
    let rec1 = client
        .call_tool(
            "record_attempt",
            serde_json::json!({
                "exploration_id": exploration_id,
                "action": "Try solution A",
                "expected_result": expected_ok,
                "actual_result": expected_ok
            }),
        )
        .await;
    let count1 = match rec1 {
        Ok(r) => payload_json(&r)
            .ok()
            .and_then(|v| v.get("attempt_count").and_then(|c| c.as_i64()))
            .unwrap_or(0),
        Err(e) => {
            crate::teeprintln!("  [FAIL] record_attempt (success case) — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if count1 == 1 {
        crate::teeprintln!("  [OK] record_attempt(success) created 1 attempt (new()+builders)");
        stats.passed += 1;
    } else {
        crate::teeprintln!("  [FAIL] record_attempt(success) count={} (expected 1)", count1);
        stats.failed += 1;
        return Ok(());
    }

    // Attempt 2: expected != actual -> success=false (test_attempt_failure).
    let rec2 = client
        .call_tool(
            "record_attempt",
            serde_json::json!({
                "exploration_id": exploration_id,
                "action": "Try solution B",
                "expected_result": expected_ok,
                "actual_result": "Still broken"
            }),
        )
        .await;
    let count2 = match rec2 {
        Ok(r) => payload_json(&r)
            .ok()
            .and_then(|v| v.get("attempt_count").and_then(|c| c.as_i64()))
            .unwrap_or(0),
        Err(e) => {
            crate::teeprintln!("  [FAIL] record_attempt (failure case) — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if count2 == 2 {
        crate::teeprintln!("  [OK] record_attempt(failure) created 2nd attempt");
        stats.passed += 1;
    } else {
        crate::teeprintln!("  [FAIL] record_attempt(failure) count={} (expected 2)", count2);
        stats.failed += 1;
        return Ok(());
    }

    // get_exploration_status: verify success flags on both attempts.
    let status = client
        .call_tool(
            "get_exploration_status",
            serde_json::json!({ "exploration_id": exploration_id }),
        )
        .await;
    let attempts = match status {
        Ok(r) => payload_json(&r)
            .ok()
            .and_then(|v| v.get("attempts").and_then(|a| a.as_array()).cloned()),
        Err(e) => {
            crate::teeprintln!("  [FAIL] get_exploration_status — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    let Some(attempts) = attempts else {
        crate::teeprintln!("  [FAIL] get_exploration_status — no attempts array");
        stats.failed += 1;
        return Ok(());
    };

    // First attempt: success=true, expected==actual (test_attempt_builder).
    let success0 = attempts
        .first()
        .and_then(|a| a.get("success").and_then(|s| s.as_bool()))
        .unwrap_or(false);
    let exp0 = attempts
        .first()
        .and_then(|a| a.get("expected_result").and_then(|e| e.as_str()))
        .unwrap_or("");
    let act0 = attempts
        .first()
        .and_then(|a| a.get("actual_result").and_then(|a| a.as_str()))
        .unwrap_or("");
    if success0 && exp0 == expected_ok && act0 == expected_ok {
        crate::teeprintln!(
            "  [OK] attempt[0] success=true, expected==actual (with_actual_result match)"
        );
        stats.passed += 1;
    } else {
        crate::teeprintln!(
            "  [FAIL] attempt[0] success={}, expected={:?}, actual={:?} (expected true/match)",
            success0,
            exp0,
            act0
        );
        stats.failed += 1;
    }

    // Second attempt: success=false, expected!=actual (test_attempt_failure).
    let success1 = attempts
        .get(1)
        .and_then(|a| a.get("success").and_then(|s| s.as_bool()))
        .unwrap_or(true);
    let act1 = attempts
        .get(1)
        .and_then(|a| a.get("actual_result").and_then(|a| a.as_str()))
        .unwrap_or("");
    if !success1 && act1 == "Still broken" {
        crate::teeprintln!(
            "  [OK] attempt[1] success=false (with_actual_result mismatch branch)"
        );
        stats.passed += 1;
    } else {
        crate::teeprintln!(
            "  [FAIL] attempt[1] success={}, actual={:?} (expected false/Still broken)",
            success1,
            act1
        );
        stats.failed += 1;
    }

    Ok(())
}
