
// src/learning/working_memory.rs

//! Learning system's Working Memory with state machine for active context
//!
//! This is a DIFFERENT concept from `src/memory/working.rs`:
//! - Memory Working Memory: Stores MemoryItem objects for retrieval (per §6.3)
//! - Learning Working Memory: Tracks active context with state machine transitions
//!
//! The Learning Working Memory is used for:
//! - Active context tracking during task execution
//! - State machine transitions (Active → Evaluated → Promoted → Archived)
//! - Promotion policies for knowledge extraction
//! - Lineage tracking for memory provenance

mod store;
pub mod memory_state;
pub mod promotion;

pub use memory_state::{MemoryState, StateTransition, StateTransitionRecord};
pub use promotion::PromotionPolicy;
pub use store::WorkingMemory;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Classification of working-memory items by purpose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Default)]
pub enum MemoryItemType {
    /// Background context for the current task.
    #[default]
    Context,
    /// A task or sub-goal to complete.
    Task,
    /// An outcome or produced result.
    Result,
    /// A learned fact or belief.
    Belief,
    /// An observation from the environment.
    Observation,
}


/// A single item held in the learning working memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryItem {
    pub id: String,
    pub key: String,
    pub value: String,
    pub item_type: MemoryItemType,
    pub importance: f32,
    pub confidence: f32,
    pub access_count: u32,
    pub confirmation_count: u32,
    pub contradiction_count: u32,
    pub state: MemoryState,
    pub created_at: DateTime<Utc>,
    pub accessed_at: DateTime<Utc>,
    pub ttl_seconds: Option<u64>,
    pub transition_history: Vec<StateTransitionRecord>,
}

impl WorkingMemoryItem {
    /// Create a new working-memory item.
    pub fn new(key: String, value: String, item_type: MemoryItemType, importance: f32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            key,
            value,
            item_type,
            importance: importance.clamp(0.0, 1.0),
            confidence: 0.0,
            access_count: 0,
            confirmation_count: 0,
            contradiction_count: 0,
            state: MemoryState::Active,
            created_at: now,
            accessed_at: now,
            ttl_seconds: None,
            transition_history: Vec::new(),
        }
    }

    /// Record an access of this item.
    pub fn record_access(&mut self) {
        self.access_count += 1;
        self.accessed_at = Utc::now();
        if self.state.can_transition(&StateTransition::Access) {
            self.apply_transition(StateTransition::Access, None);
        }
    }

    /// Record an external confirmation of this item.
    pub fn record_confirmation(&mut self) {
        self.confirmation_count += 1;
        self.confidence += 0.1;
        if self.state.can_transition(&StateTransition::Confirm) {
            self.apply_transition(StateTransition::Confirm, None);
        }
    }

    /// Record an external contradiction of this item.
    pub fn record_contradiction(&mut self) {
        self.contradiction_count += 1;
        self.confidence -= 0.1;
        if self.state.can_transition(&StateTransition::Contradict) {
            self.apply_transition(StateTransition::Contradict, None);
        }
    }

    /// Attempt a state transition. Returns false if not valid from the current state.
    pub fn transition(&mut self, transition: StateTransition, reason: Option<String>) -> bool {
        if self.state.can_transition(&transition) {
            self.apply_transition(transition, reason);
            true
        } else {
            false
        }
    }

    /// Whether a promotion policy says this item should be promoted.
    pub fn should_promote(&self, policy: &PromotionPolicy) -> bool {
        policy.evaluate(self).should_promote
    }

    /// Whether this item's TTL has elapsed (if one is set).
    pub fn is_expired(&self) -> bool {
        match self.ttl_seconds {
            Some(ttl) => {
                let age = Utc::now() - self.created_at;
                age.num_seconds() > ttl as i64
            }
            None => false,
        }
    }

    fn apply_transition(&mut self, transition: StateTransition, reason: Option<String>) {
        if let Some(to_state) = self.state.transition_to(&transition) {
            let record = StateTransitionRecord::new(self.state, to_state, transition, reason);
            self.state = to_state;
            self.transition_history.push(record);
        }
    }
}


