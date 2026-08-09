// src/agent/safety_gate.rs
//! Safety gate for the autonomous agent loop (Architecture §16 — Safety).
//!
//! Per TASK-V2-07, the safety layer gates every action the autonomous loop
//! would take. This is a minimal, real implementation that enforces the
//! invariants required before an autonomous loop is safe to run:
//!
//!   * **Confidence threshold** — refuse to act when the evaluated action
//!     confidence is below the goal's threshold (Architecture §5.7).
//!   * **Uncertainty reporting** — when an action is blocked, the decision
//!     carries the reason so the loop can abstain and record a low-confidence
//!     experience instead of acting blindly.
//!   * **Action allow-list** — only known, low-risk read/learn actions are
//!     permitted until a fuller permission model (sandboxing, rollback) is
//!     added. This prevents the autonomous loop from invoking destructive
//!     tools while the safety layer matures.
//!
//! Fuller sandboxing, permission checks and rollback are future work tracked
//! under TASK-V2-07; this gate is the prerequisite that lets the loop run at
//! all safely.

use super::decision::ActionConfidence;

/// The safety gate's verdict on a proposed action.
#[derive(Debug, Clone)]
pub enum SafetyDecision {
    /// The action is cleared to execute.
    Allow,
    /// The action is blocked; `reason` explains why (recorded as uncertainty).
    Block { reason: String },
}

/// Action categories the gate recognizes. Read/learn actions are auto-allowed;
/// everything else is blocked pending a permission model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionRisk {
    /// Read-only retrieval or learning action (memory/knowledge/experience
    /// lookup, planning, reflection). Safe to run autonomously.
    Read,
    /// Mutating action (store_memory, add_knowledge, record_experience). These
    /// are permitted because they are idempotent/append-only and core to the
    /// learning loop, but they are logged.
    Mutate,
    /// Potentially destructive or external action. Blocked until a full
    /// permission + sandbox model exists.
    Destructive,
}

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
pub struct SafetyGate {
    /// Global minimum confidence, independent of per-goal thresholds.
    min_confidence: f32,
}

impl SafetyGate {
    pub fn new() -> Self {
        Self {
            min_confidence: 0.3,
        }
    }

    /// Evaluate whether a proposed action may execute.
    ///
    /// Returns `Block` (with a human-readable reason) when:
    ///   - the action risk is `Destructive` (no permission model yet), or
    ///   - confidence is below the global floor or the goal's threshold.
    pub fn evaluate(&self, action: &str, confidence: &ActionConfidence) -> SafetyDecision {
        let risk = action_risk(action);
        if risk == ActionRisk::Destructive {
            return SafetyDecision::Block {
                reason: format!(
                    "Action '{}' is not on the autonomous allow-list (destructive risk); \
                     a permission/sandbox model is required before autonomous execution.",
                    action
                ),
            };
        }

        if confidence.value < self.min_confidence {
            return SafetyDecision::Block {
                reason: format!(
                    "Confidence {:.2} below global safety floor {:.2}",
                    confidence.value, self.min_confidence
                ),
            };
        }

        if confidence.value < confidence.threshold {
            return SafetyDecision::Block {
                reason: format!(
                    "Confidence {:.2} below goal threshold {:.2}",
                    confidence.value, confidence.threshold
                ),
            };
        }

        SafetyDecision::Allow
    }
}

impl Default for SafetyGate {
    fn default() -> Self {
        Self::new()
    }
}
