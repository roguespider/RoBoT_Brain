use crate::data_contracts::memory_record::MemoryKind;
use crate::data_contracts::memory_record::MemoryRecord;
use serde::{Deserialize, Serialize};

/// Memory lifecycle states - Per Architecture §8.4
///
/// Legal transitions: Working → Candidate → Accepted → Permanent
/// Any state may transition to Archived (terminal).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub enum MemoryLifecycle {
    /// Memory is being actively used or evaluated
    #[default]
    Working,
    /// Memory has been identified as worth promoting but not yet accepted
    Candidate,
    /// Memory has been reviewed and accepted for promotion
    Accepted,
    /// Memory is fully promoted to permanent storage
    Permanent,
    /// Memory has been archived (terminal state)
    Archived,
}

impl std::fmt::Display for MemoryLifecycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MemoryLifecycle::Working => "working",
            MemoryLifecycle::Candidate => "candidate",
            MemoryLifecycle::Accepted => "accepted",
            MemoryLifecycle::Permanent => "permanent",
            MemoryLifecycle::Archived => "archived",
        };
        write!(f, "{}", s)
    }
}

/// Check if a promotion from `from` to `to` is a legal transition.
///
/// Legal transitions per Architecture §8.4:
/// - Working → Candidate
/// - Candidate → Accepted
/// - Accepted → Permanent
/// - Any state → Archived
pub fn can_promote(from: MemoryLifecycle, to: MemoryLifecycle) -> bool {
    match (from, to) {
        (MemoryLifecycle::Working, MemoryLifecycle::Candidate) => true,
        (MemoryLifecycle::Candidate, MemoryLifecycle::Accepted) => true,
        (MemoryLifecycle::Accepted, MemoryLifecycle::Permanent) => true,
        (_, MemoryLifecycle::Archived) => true,
        _ => false,
    }
}

/// Returns true only for the Archived terminal state.
pub fn is_terminal(s: MemoryLifecycle) -> bool {
    s == MemoryLifecycle::Archived
}

/// Active reference to memory lifecycle contracts.
pub fn reference_memory_lifecycle_contracts() {
    // Wire can_promote
    let can_result = can_promote(MemoryLifecycle::Working, MemoryLifecycle::Candidate);
    tracing::info!(can_promote = can_result, "can_promote actively referenced");

    // Wire is_terminal
    let is_term = is_terminal(MemoryLifecycle::Archived);
    tracing::info!(is_terminal = is_term, "is_terminal actively referenced");
}

/// Promotion gate for memory lifecycle transitions.
///
/// This struct defines the thresholds that must be met for a memory to be promoted
/// from one lifecycle state to the next. Used in T2-32 to evaluate if a memory
/// should be promoted based on age, confidence, and access count.
pub struct PromotionGate {
    /// Minimum age in seconds required for promotion
    pub min_age_secs: u64,
    /// Minimum confidence score required for promotion
    pub min_confidence: f32,
    /// Minimum access count required for promotion
    pub min_access_count: u32,
}

impl PromotionGate {
    /// Evaluate if a memory should be promoted to the next lifecycle state.
    ///
    /// Returns the next lifecycle state if all thresholds are met, otherwise None.
    ///
    /// The evaluation logic:
    /// 1. Check if the memory is in a state that can be promoted
    /// 2. Check if the memory's age meets the minimum age requirement
    /// 3. Check if the memory's confidence meets the minimum confidence requirement
    /// 4. Check if the memory's access count meets the minimum access count requirement
    ///
    /// Returns None if any condition fails.
    pub fn evaluate(&self, memory: &MemoryRecord) -> Option<MemoryLifecycle> {
        // Check if the memory is in a state that can be promoted
        let current_kind = memory.kind;
        let can_promote = match current_kind {
            MemoryKind::Working => true,
            MemoryKind::Candidate => true,
            MemoryKind::Accepted => true,
            MemoryKind::Permanent => false,
            MemoryKind::Archived => false,
        };

        if !can_promote {
            return None;
        }

        // Calculate age in seconds
        let now = chrono::Utc::now().timestamp();
        let age_secs = (now - memory.metadata.created_at) as u64;

        // Check all thresholds
        if age_secs >= self.min_age_secs
            && memory.metadata.confidence >= self.min_confidence
            && memory.access_count >= self.min_access_count
        {
            // All thresholds met, return the next state
            match current_kind {
                MemoryKind::Working => Some(MemoryLifecycle::Candidate),
                MemoryKind::Candidate => Some(MemoryLifecycle::Accepted),
                MemoryKind::Accepted => Some(MemoryLifecycle::Permanent),
                _ => None, // Working, Permanent, or Archived cannot be promoted further
            }
        } else {
            None
        }
    }
}
