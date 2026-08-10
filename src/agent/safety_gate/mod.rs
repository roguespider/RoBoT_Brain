//! Safety gate for the autonomous agent loop (Architecture §16 — Safety).
//!
//! Per TASK-V2-07, the safety layer gates every action the autonomous loop
//! would take. This module implements the full §16 feature set:
//!
//!   * **Sandboxing** — `Sandbox` enforces resource boundaries and a mutation
//!     budget so the loop cannot access disallowed subsystems or amplify
//!     writes unboundedly.
//!   * **Permission checks** — `action_risk()` classifies every action as
//!     Read / Mutate / Destructive. Destructive actions are blocked.
//!   * **Confidence thresholds** — the gate refuses to act when the evaluated
//!     action confidence is below the goal's threshold (Architecture §5.7).
//!   * **Rollback** — `RollbackJournal` records mutations so a failed action
//!     can be reversed by marking entries as rolled-back.
//!   * **Hallucination handling** — `HallucinationCheck` detects when the
//!     agent commits to an action with insufficient evidence diversity and
//!     penalizes confidence to make the threshold check more likely to block.
//!   * **Uncertainty reporting** — `UncertaintyReport` provides a structured
//!     reason for every block, recorded in the abstention experience.

pub mod hallucination;
pub mod rollback;
pub mod sandbox;
pub mod types;

pub use hallucination::{apply_hallucination_penalty, HallucinationCheck};
pub use rollback::RollbackJournal;
pub use types::{ActionRisk, RollbackEntry, SafetyDecision, UncertaintyReport};

use crate::agent::decision::{ActionConfidence, SelectedAction};
use sandbox::Sandbox;
use std::sync::Mutex;

/// Maps an action name to a risk category.
pub fn action_risk(action: &str) -> ActionRisk {
    match action {
        // Read-only retrieval / learning actions.
        "search_memory" | "get_memory" | "list_memories" | "query_knowledge"
        | "get_knowledge" | "global_search" | "list_experiences" | "get_insights"
        | "get_plan" | "list_plans" | "list_workflows" | "get_system_status"
        | "discover_skill" | "list_tools" => ActionRisk::Read,
        // Append-only / idempotent mutations central to the learning loop.
        "store_memory" | "add_knowledge" | "record_experience" | "create_plan"
        | "create_workflow" | "start_workflow" | "create_hypothesis"
        | "add_evidence" | "register_agent" | "register_skill" => ActionRisk::Mutate,
        // Everything else is treated as destructive until explicitly allow-listed.
        _ => ActionRisk::Destructive,
    }
}

/// Safety gate evaluated before every autonomous action.
///
/// Composes four safety checks: sandbox boundary, hallucination detection,
/// confidence threshold, and action risk classification. Each check
/// contributes to a structured `UncertaintyReport` when the action is blocked.
pub struct SafetyGate {
    /// Global minimum confidence, independent of per-goal thresholds.
    min_confidence: f32,
    /// Sandbox enforcing resource boundaries.
    sandbox: Mutex<Sandbox>,
    /// Rollback journal tracking mutations for potential reversal.
    journal: Mutex<RollbackJournal>,
}

impl SafetyGate {
    pub fn new() -> Self {
        Self {
            min_confidence: 0.3,
            sandbox: Mutex::new(Sandbox::default()),
            journal: Mutex::new(RollbackJournal::new()),
        }
    }

    /// Evaluate whether a proposed action may execute.
    ///
    /// Returns `Block` (with a human-readable reason) when:
    ///   - the action risk is `Destructive` (no permission model yet), or
    ///   - the sandbox blocks it (resource boundary or mutation budget), or
    ///   - a hallucination risk is detected and penalized confidence falls
    ///     below threshold, or
    ///   - confidence is below the global floor or the goal's threshold.
    pub fn evaluate(&self, action: &str, confidence: &ActionConfidence) -> SafetyDecision {
        let risk = action_risk(action);

        // 1. Destructive actions are always blocked.
        if risk == ActionRisk::Destructive {
            let report = UncertaintyReport::new()
                .sandbox_blocked(true)
                .concern(format!(
                    "Action '{}' is not on the autonomous allow-list (destructive risk)",
                    action
                ));
            return SafetyDecision::Block {
                reason: report.summary.clone(),
                report,
            };
        }

        // 2. Sandbox check (resource boundary + mutation budget).
        if let Ok(mut sandbox) = self.sandbox.lock()
            && let Err(reason) = sandbox.check(action)
        {
            let report = UncertaintyReport::new()
                .sandbox_blocked(true)
                .concern(reason);
            return SafetyDecision::Block {
                reason: report.summary.clone(),
                report,
            };
        }

        // 3. Confidence threshold checks.
        if confidence.value < self.min_confidence {
            let report = UncertaintyReport::new()
                .low_confidence(true)
                .concern(format!(
                    "Confidence {:.2} below global safety floor {:.2}",
                    confidence.value, self.min_confidence
                ));
            return SafetyDecision::Block {
                reason: report.summary.clone(),
                report,
            };
        }

        if confidence.value < confidence.threshold {
            let report = UncertaintyReport::new()
                .low_confidence(true)
                .concern(format!(
                    "Confidence {:.2} below goal threshold {:.2}",
                    confidence.value, confidence.threshold
                ));
            return SafetyDecision::Block {
                reason: report.summary.clone(),
                report,
            };
        }

        SafetyDecision::Allow
    }

    /// Full evaluation with hallucination check. Called by the agent loop
    /// when it has a `SelectedAction` with supporting evidence.
    ///
    /// This is the preferred entry point: it runs the hallucination check
    /// (which requires evidence counts only available from `SelectedAction`),
    /// applies a confidence penalty if risk is detected, then delegates to
    /// `evaluate()`.
    pub fn evaluate_full(&self, selected: &mut SelectedAction) -> SafetyDecision {
        let check = HallucinationCheck::evaluate(selected);
        if check.risk_detected {
            apply_hallucination_penalty(&mut selected.confidence, &check);
        }

        let decision = self.evaluate(&selected.step.action, &selected.confidence);

        if let SafetyDecision::Block { reason, mut report } = decision {
            if check.risk_detected {
                report = report
                    .hallucination_risk(true)
                    .evidence_count(check.evidence_count)
                    .concern(format!(
                        "hallucination: {}, evidence channels: {}",
                        check.reason, check.evidence_channels
                    ));
                return SafetyDecision::Block {
                    reason: report.summary.clone(),
                    report,
                };
            }
            return SafetyDecision::Block { reason, report };
        }

        // Log evidence diversity for monitoring even when allowed.
        if !check.risk_detected && check.evidence_channels > 0 {
            tracing::debug!(
                "Safety gate: action '{}' cleared with {} evidence channel(s), {} item(s)",
                selected.step.action,
                check.evidence_channels,
                check.evidence_count
            );
        }

        SafetyDecision::Allow
    }

    /// Record a mutation in the rollback journal for potential reversal.
    pub fn record_mutation(&self, action: &str, target_id: String) {
        if let Ok(mut journal) = self.journal.lock() {
            journal.record(action, target_id);
        }
    }

    /// Roll back all mutations in the current journal. Returns the entries
    /// that were reversed so the caller can record a rollback experience.
    pub fn rollback_all(&self) -> Vec<RollbackEntry> {
        if let Ok(mut journal) = self.journal.lock() {
            return journal.rollback_all();
        }
        Vec::new()
    }

    /// Reset the per-iteration counters (sandbox mutation budget + journal).
    /// Called at the start of each new agent loop iteration.
    pub fn reset_iteration(&self) {
        if let Ok(mut sandbox) = self.sandbox.lock() {
            sandbox.reset_iteration();
        }
        if let Ok(mut journal) = self.journal.lock() {
            journal.clear();
        }
    }

    /// Number of active (non-rolled-back) mutations in the journal.
    pub fn active_mutations(&self) -> usize {
        if let Ok(journal) = self.journal.lock() {
            journal.active_count()
        } else {
            0
        }
    }

    /// Roll back a specific mutation by its target ID (partial rollback).
    /// Returns true if a matching active entry was found and reversed.
    pub fn rollback_target(&self, target_id: &str) -> bool {
        if let Ok(mut journal) = self.journal.lock() {
            return journal.rollback_target(target_id).is_some();
        }
        false
    }

    /// Return a snapshot of all journal entries for audit purposes.
    pub fn journal_entries(&self) -> Vec<RollbackEntry> {
        if let Ok(journal) = self.journal.lock() {
            return journal.entries().to_vec();
        }
        Vec::new()
    }
}

impl Default for SafetyGate {
    fn default() -> Self {
        Self::new()
    }
}
