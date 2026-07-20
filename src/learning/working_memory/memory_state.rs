// src/learning/working_memory/memory_state.rs
//! Memory state machine types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// States a memory item can be in
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryState {
    /// Currently active and being used
    Active,
    
    /// Has been accessed multiple times
    Repeated,
    
    /// Confirmed by external sources
    Confirmed,
    
    /// Temporarily dormant, may be revived
    Dormant,
    
    /// Discarded or expired
    Discarded,
}

impl Default for MemoryState {
    fn default() -> Self {
        Self::Active
    }
}

/// Valid state transitions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StateTransition {
    /// Access the memory
    Access,
    
    /// Observe the memory
    Observe,
    
    /// Confirm the memory
    Confirm,
    
    /// Contradict the memory
    Contradict,
    
    /// Reject the memory
    Reject,
    
    /// Promote to long-term
    Promote,
    
    /// Demote to dormant
    Demote,
    
    /// Discard the memory
    Discard,
    
    /// Timeout expired
    Timeout,
}

impl StateTransition {
    /// Check if a transition is valid from a given state
    pub fn is_valid_from(&self, from: MemoryState) -> bool {
        match (self, from) {
            (StateTransition::Access, MemoryState::Active) => true,
            (StateTransition::Access, MemoryState::Dormant) => true,
            (StateTransition::Observe, MemoryState::Active) => true,
            (StateTransition::Observe, MemoryState::Repeated) => true,
            (StateTransition::Confirm, MemoryState::Repeated) => true,
            (StateTransition::Confirm, MemoryState::Confirmed) => true,
            (StateTransition::Contradict, _) => true,
            (StateTransition::Reject, _) => true,
            (StateTransition::Promote, MemoryState::Confirmed) => true,
            (StateTransition::Demote, MemoryState::Active) => true,
            (StateTransition::Demote, MemoryState::Repeated) => true,
            (StateTransition::Discard, _) => true,
            (StateTransition::Timeout, _) => true,
            _ => false,
        }
    }
    
    /// Get the resulting state after this transition
    pub fn target_state(&self) -> MemoryState {
        match self {
            Self::Access => MemoryState::Active,
            Self::Observe => MemoryState::Repeated,
            Self::Confirm => MemoryState::Confirmed,
            Self::Contradict => MemoryState::Dormant,
            Self::Reject => MemoryState::Discarded,
            Self::Promote => MemoryState::Discarded,
            Self::Demote => MemoryState::Dormant,
            Self::Discard => MemoryState::Discarded,
            Self::Timeout => MemoryState::Dormant,
        }
    }
}

impl MemoryState {
    /// Check if a transition is valid from this state
    pub fn can_transition(&self, transition: &StateTransition) -> bool {
        transition.is_valid_from(*self)
    }
    
    /// Get the target state for a transition
    pub fn transition_to(&self, transition: &StateTransition) -> Option<MemoryState> {
        if self.can_transition(transition) {
            Some(transition.target_state())
        } else {
            None
        }
    }
}

/// Record of a state transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransitionRecord {
    pub timestamp: DateTime<Utc>,
    pub from_state: MemoryState,
    pub to_state: MemoryState,
    pub transition: StateTransition,
    pub reason: Option<String>,
}

impl StateTransitionRecord {
    /// Create a new transition record
    pub fn new(
        from_state: MemoryState,
        to_state: MemoryState,
        transition: StateTransition,
        reason: Option<String>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            from_state,
            to_state,
            transition,
            reason,
        }
    }
}
