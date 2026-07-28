// src/experience/types/evidence.rs
#![allow(dead_code)]
// Evidence types

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Source of an experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperienceSource {
    User,
    Tool,
    Planner,
    Memory,
    Reflection,
    Exploration,
    Hypothesis,
    Evolution,
    System,
    Model,
}

/// Evidence supporting an experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Evidence identifier.
    pub id: Uuid,

    /// Experiences supported by this evidence.
    pub experience_ids: Vec<Uuid>,

    /// Confidence in this evidence.
    pub confidence: f32,
}
