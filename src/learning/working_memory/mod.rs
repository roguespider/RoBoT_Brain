
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
#[cfg(test)]
mod tests;

pub mod memory_state;
pub mod promotion;

pub use memory_state::{MemoryState, StateTransition, StateTransitionRecord};
pub use promotion::PromotionPolicy;
#[cfg(test)]
pub use store::WorkingMemory;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A piece of information in working memory with state machine support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryItem {
    pub id: String,
    pub key: String,
    pub value: String,
    pub item_type: MemoryItemType,
    pub importance: f32,
    pub confidence: f32,
    pub state: MemoryState,
    pub ttl_seconds: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub accessed_at: DateTime<Utc>,
    pub access_count: u32,
    pub repeated_count: u32,
    pub confirmation_count: u32,
    pub contradicted: bool,
    pub transition_history: Vec<StateTransitionRecord>,
}

impl WorkingMemoryItem {
    pub fn new(
        key: String,
        value: String,
        item_type: MemoryItemType,
        importance: f32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            key,
            value,
            item_type,
            importance,
            confidence: 0.5,
            state: MemoryState::Active,
            ttl_seconds: None,
            created_at: now,
            accessed_at: now,
            access_count: 1,
            repeated_count: 0,
            confirmation_count: 0,
            contradicted: false,
            transition_history: Vec::new(),
        }
    }
    
    pub fn transition(&mut self, transition: StateTransition, reason: Option<String>) -> bool {
        if !self.state.can_transition(&transition) {
            return false;
        }
        
        if let Some(new_state) = self.state.transition_to(&transition) {
            let record = StateTransitionRecord::new(self.state, new_state, transition, reason);
            self.transition_history.push(record);
            self.state = new_state;
            return true;
        }
        
        false
    }
    
    pub fn record_access(&mut self) {
        self.accessed_at = Utc::now();
        self.access_count += 1;
        
        if self.state == MemoryState::Active {
            self.repeated_count += 1;
            if self.repeated_count > 1 {
                let _ = self.transition(StateTransition::Observe, Some("Repeated access".to_string()));
            }
        } else if self.state == MemoryState::Dormant {
            let _ = self.transition(StateTransition::Access, Some("Revived by access".to_string()));
        }
    }
    
    pub fn record_confirmation(&mut self) {
        self.confirmation_count += 1;
        if self.state == MemoryState::Repeated {
            let _ = self.transition(StateTransition::Confirm, Some("Confirmed".to_string()));
        }
    }
    
    pub fn record_contradiction(&mut self) {
        self.contradicted = true;
        if matches!(self.state, MemoryState::Active | MemoryState::Repeated | MemoryState::Confirmed) {
            let _ = self.transition(StateTransition::Contradict, Some("Contradicted".to_string()));
        }
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl_seconds {
            let age = Utc::now() - self.created_at;
            chrono::Duration::seconds(ttl as i64) < age
        } else {
            false
        }
    }
    
    pub fn should_promote(&self, policy: &PromotionPolicy) -> bool {
        policy.evaluate(self).should_promote
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryItemType {
    Context,
    Task,
    Result,
    Error,
    Metadata,
    State,
}
