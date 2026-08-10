//! Observation handlers: record attempt, get exploration status.

use uuid::Uuid;

use crate::bridge::tools::ToolOutput;
use crate::experience::exploration::ExplorationAttempt;

use super::store::{ensure_exploration, with_store};
use crate::bridge::tools::exploration::{GetExplorationStatusInput, RecordAttemptInput};

pub fn execute_record_attempt(input: RecordAttemptInput) -> ToolOutput {
    with_store(|store| {
        ensure_exploration(store, &input.exploration_id);
        match store.get_mut(&input.exploration_id) {
            Some(exp) => {
                let mut attempt =
                    ExplorationAttempt::new(Uuid::new_v4().to_string(), input.action);

                if let Some(expected) = input.expected_result {
                    attempt = attempt.with_expected_result(expected);
                }
                if let Some(actual) = input.actual_result {
                    attempt = attempt.with_actual_result(actual);
                }

                exp.add_attempt(attempt);

                ToolOutput::success(serde_json::json!({
                    "exploration_id": exp.id,
                    "attempt_count": exp.attempts.len(),
                    "message": "Attempt recorded."
                }))
            }
            None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
        }
    })
}

pub fn execute_get_exploration_status(input: GetExplorationStatusInput) -> ToolOutput {
    with_store(|store| {
        ensure_exploration(store, &input.exploration_id);
        match store.get(&input.exploration_id) {
            Some(exp) => {
                let hypotheses_json: Vec<_> = exp.hypotheses.iter().map(|h| serde_json::json!({
                    "id": h.id,
                    "statement": h.statement,
                    "confidence": h.confidence,
                    "result": h.result.as_ref().map(|r| format!("{:?}", r))
                })).collect();

                let attempts_json: Vec<_> = exp.attempts.iter().map(|a| serde_json::json!({
                    "id": a.id,
                    "action": a.action,
                    "expected_result": a.expected_result,
                    "actual_result": a.actual_result,
                    "success": a.success
                })).collect();

                let findings_json: Vec<_> = exp.findings.iter().map(|f| serde_json::json!({
                    "id": f.id,
                    "description": f.description,
                    "confidence": f.confidence,
                    "promoted": f.promoted
                })).collect();

                ToolOutput::success(serde_json::json!({
                    "exploration_id": exp.id,
                    "title": exp.title,
                    "purpose": exp.purpose,
                    "status": format!("{:?}", exp.status),
                    "started_at": exp.started_at.to_rfc3339(),
                    "completed_at": exp.completed_at.map(|t| t.to_rfc3339()),
                    "is_active": exp.is_active(),
                    "is_complete": exp.is_complete(),
                    "hypotheses": hypotheses_json,
                    "attempts": attempts_json,
                    "findings": findings_json
                }))
            }
            None => ToolOutput::error(format!("Exploration not found: {}", input.exploration_id)),
        }
    })
}
