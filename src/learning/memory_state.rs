// src/learning/memory_state.rs
//! State machine for working memory items

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fmt;

/// Working memory states - items progress through these as they're evaluated
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryState {
    /// Newly added, actively in use
    Active,
    
    /// No longer accessed, pending evaluation
    Dormant,
    
    /// TTL expired, awaiting promotion decision
    Expired,
    
    /// Same information seen multiple times
    Repeated,
    
    /// Information confirmed by multiple sources
    Confirmed,
    
    /// Contradicted by conflicting information
    Contradicted,
    
    /// Promoted to long-term memory
    Promoted,
    
    /// Rejected and marked for deletion
    Rejected,
}

impl Default for MemoryState {
    fn default() -> Self {
        MemoryState::Active
    }
}

impl fmt::Display for MemoryState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryState::Active => write!(f, "Active"),
            MemoryState::Dormant => write!(f, "Dormant"),
            MemoryState::Expired => write!(f, "Expired"),
            MemoryState::Repeated => write!(f, "Repeated"),
            MemoryState::Confirmed => write!(f, "Confirmed"),
            MemoryState::Contradicted => write!(f, "Contradicted"),
            MemoryState::Promoted => write!(f, "Promoted"),
            MemoryState::Rejected => write!(f, "Rejected"),
        }
    }
}

/// Valid state transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateTransition {
    Access,
    Timeout,
    Observe,
    Confirm,
    Contradict,
    Promote,
    Reject,
    Revive,
}

impl MemoryState {
    /// Check if transition is valid from current state
    pub fn can_transition(&self, transition: StateTransition) -> bool {
        match (self, transition) {
            // Active can go to many states
            (MemoryState::Active, StateTransition::Access) => true,
            (MemoryState::Active, StateTransition::Timeout) => true,
            (MemoryState::Active, StateTransition::Observe) => true,
            (MemoryState::Active, StateTransition::Promote) => true,
            (MemoryState::Active, StateTransition::Reject) => true,
            
            // Dormant can be revived or timeout
            (MemoryState::Dormant, StateTransition::Access) => true,
            (MemoryState::Dormant, StateTransition::Timeout) => true,
            
            // Expired can be promoted, rejected, or revived
            (MemoryState::Expired, StateTransition::Promote) => true,
            (MemoryState::Expired, StateTransition::Reject) => true,
            (MemoryState::Expired, StateTransition::Revive) => true,
            
            // Repeated needs confirmation or contradiction
            (MemoryState::Repeated, StateTransition::Confirm) => true,
            (MemoryState::Repeated, StateTransition::Contradict) => true,
            (MemoryState::Repeated, StateTransition::Promote) => true,
            (MemoryState::Repeated, StateTransition::Reject) => true,
            
            // Confirmed can be promoted or contradicted
            (MemoryState::Confirmed, StateTransition::Promote) => true,
            (MemoryState::Confirmed, StateTransition::Contradict) => true,
            
            // Contradicted can be rejected or revived for re-evaluation
            (MemoryState::Contradicted, StateTransition::Reject) => true,
            (MemoryState::Contradicted, StateTransition::Revive) => true,
            
            // Promoted and Rejected are terminal states
            (MemoryState::Promoted, _) => false,
            (MemoryState::Rejected, _) => false,
        }
    }
    
    /// Get the target state for a transition
    pub fn transition_to(&self, transition: StateTransition) -> Option<MemoryState> {
        match (self, transition) {
            (MemoryState::Active, StateTransition::Access) => Some(MemoryState::Active),
            (MemoryState::Active, StateTransition::Timeout) => Some(MemoryState::Dormant),
            (MemoryState::Active, StateTransition::Observe) => Some(MemoryState::Repeated),
            (MemoryState::Active, StateTransition::Promote) => Some(MemoryState::Promoted),
            (MemoryState::Active, StateTransition::Reject) => Some(MemoryState::Rejected),
            
            (MemoryState::Dormant, StateTransition::Access) => Some(MemoryState::Active),
            (MemoryState::Dormant, StateTransition::Timeout) => Some(MemoryState::Expired),
            
            (MemoryState::Expired, StateTransition::Promote) => Some(MemoryState::Promoted),
            (MemoryState::Expired, StateTransition::Reject) => Some(MemoryState::Rejected),
            (MemoryState::Expired, StateTransition::Revive) => Some(MemoryState::Dormant),
            
            (MemoryState::Repeated, StateTransition::Confirm) => Some(MemoryState::Confirmed),
            (MemoryState::Repeated, StateTransition::Contradict) => Some(MemoryState::Contradicted),
            (MemoryState::Repeated, StateTransition::Promote) => Some(MemoryState::Promoted),
            (MemoryState::Repeated, StateTransition::Reject) => Some(MemoryState::Rejected),
            
            (MemoryState::Confirmed, StateTransition::Promote) => Some(MemoryState::Promoted),
            (MemoryState::Confirmed, StateTransition::Contradict) => Some(MemoryState::Contradicted),
            
            (MemoryState::Contradicted, StateTransition::Reject) => Some(MemoryState::Rejected),
            (MemoryState::Contradicted, StateTransition::Revive) => Some(MemoryState::Dormant),
            
            // Terminal states
            (MemoryState::Promoted, _) => None,
            (MemoryState::Rejected, _) => None,
        }
    }
}

/// State transition record for auditing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransitionRecord {
    pub from_state: MemoryState,
    pub to_state: MemoryState,
    pub transition: StateTransition,
    pub timestamp: DateTime<Utc>,
    pub reason: Option<String>,
}

impl StateTransitionRecord {
    pub fn new(from: MemoryState, to: MemoryState, transition: StateTransition, reason: Option<String>) -> Self {
        Self {
            from_state: from,
            to_state: to,
            transition,
            timestamp: Utc::now(),
            reason,
        }
    }
}
