// src/experience/types/encounter.rs
#![allow(dead_code)]
// Encounter types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::context::ExperienceContext;

/// Result of an encounter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncounterResult {
    Success,
    Failure,

    /// Partial completion with explanation.
    Partial(String),

    /// Error message.
    Error(String),

    Timeout,
}

/// A recorded encounter within an experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encounter {
    /// Unique encounter identifier.
    pub id: Uuid,

    /// When the encounter occurred.
    pub timestamp: DateTime<Utc>,

    /// Related experience.
    pub experience_id: Option<Uuid>,

    /// Context surrounding the encounter.
    pub context: ExperienceContext,

    /// Original input.
    pub input: String,

    /// Action performed.
    pub action: String,

    /// Result of the encounter.
    pub result: EncounterResult,

    /// Additional metadata.
    pub metadata: HashMap<String, String>,
}

/// Statistics for encounters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterStats {
    /// Experience being tracked.
    pub experience_id: Uuid,

    /// Total encounters.
    pub total_encounters: u64,

    /// Successful encounters.
    pub successes: u64,

    /// Failed encounters.
    pub failures: u64,

    /// First observed.
    pub first_seen: DateTime<Utc>,

    /// Most recent observation.
    pub last_seen: DateTime<Utc>,

    /// Average calculated score.
    pub average_score: f32,
}
