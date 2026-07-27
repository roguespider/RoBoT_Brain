// src/experience/types/reputation.rs
// Reputation types

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Target of reputation tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReputationTarget {
    Tool(String),
    Workflow(String),
    Memory(String),
    Model(String),
    Hypothesis(Uuid),
    Exploration(Uuid),
    Experience(Uuid),
    Agent(String),
    Custom(String),
}

/// A record of reputation for an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationRecord {
    /// Entity whose reputation is being tracked.
    pub target: ReputationTarget,

    /// Overall reputation score (typically 0.0..1.0).
    pub score: f32,

    /// Successful outcomes.
    pub successes: u64,

    /// Failed outcomes.
    pub failures: u64,

    /// Number of observations.
    pub observations: u64,

    /// Average confidence across observations.
    pub confidence: f32,

    /// Last update time.
    pub last_updated: DateTime<Utc>,
}

impl ReputationRecord {
    /// Create a new reputation record.
    pub fn new(target: ReputationTarget) -> Self {
        Self {
            target,
            score: 0.0,
            successes: 0,
            failures: 0,
            observations: 0,
            confidence: 0.0,
            last_updated: Utc::now(),
        }
    }

    /// Record a successful observation.
    pub fn record_success(&mut self, confidence: f32) {
        self.successes += 1;
        self.observations += 1;
        self.confidence = confidence;
        self.last_updated = Utc::now();
    }

    /// Record a failed observation.
    pub fn record_failure(&mut self, confidence: f32) {
        self.failures += 1;
        self.observations += 1;
        self.confidence = confidence;
        self.last_updated = Utc::now();
    }
}
