//! Shared types for the safety gate (Architecture §16).

/// The safety gate's verdict on a proposed action.
#[derive(Debug, Clone)]
pub enum SafetyDecision {
    /// The action is cleared to execute.
    Allow,
    /// The action is blocked; `reason` explains why (recorded as uncertainty),
    /// and `report` carries the structured uncertainty breakdown.
    Block {
        reason: String,
        report: UncertaintyReport,
    },
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

/// Structured uncertainty report produced when an action is evaluated.
///
/// Per Architecture §16 "uncertainty reporting" — when the gate blocks or
/// conditionally allows an action, it emits a structured report so the loop
/// can record *why* in the experience, not just *that* it was blocked.
#[derive(Debug, Clone, Default)]
pub struct UncertaintyReport {
    /// Human-readable summary of all concerns.
    pub summary: String,
    /// Whether a hallucination risk was detected.
    pub hallucination_risk: bool,
    /// Whether confidence was below threshold.
    pub low_confidence: bool,
    /// Whether the action was blocked by the sandbox allow-list.
    pub sandbox_blocked: bool,
    /// Number of supporting evidence items across all channels.
    pub evidence_count: usize,
}

impl UncertaintyReport {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a concern to the summary and return self for chaining.
    pub fn concern(mut self, msg: impl Into<String>) -> Self {
        if self.summary.is_empty() {
            self.summary = msg.into();
        } else {
            self.summary.push_str("; ");
            self.summary.push_str(&msg.into());
        }
        self
    }

    pub fn hallucination_risk(mut self, v: bool) -> Self {
        self.hallucination_risk = v;
        self
    }

    pub fn low_confidence(mut self, v: bool) -> Self {
        self.low_confidence = v;
        self
    }

    pub fn sandbox_blocked(mut self, v: bool) -> Self {
        self.sandbox_blocked = v;
        self
    }

    pub fn evidence_count(mut self, n: usize) -> Self {
        self.evidence_count = n;
        self
    }
}

/// An entry in the rollback journal recording a mutation for potential reversal.
#[derive(Debug, Clone)]
pub struct RollbackEntry {
    /// Unique ID for this journal entry.
    pub id: String,
    /// The action/tool that was executed.
    pub action: String,
    /// When the action was taken.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The key identifying what was mutated (e.g. memory_id, knowledge_id).
    pub target_id: String,
    /// Whether this entry has been rolled back.
    pub rolled_back: bool,
}

impl RollbackEntry {
    pub fn new(action: &str, target_id: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            action: action.to_string(),
            timestamp: chrono::Utc::now(),
            target_id,
            rolled_back: false,
        }
    }
}
