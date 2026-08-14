// src/experience/types/evidence.rs
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

impl Evidence {
    /// Create new evidence linking the given experiences at a confidence.
    pub fn new(experience_ids: Vec<Uuid>, confidence: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_ids,
            confidence,
        }
    }
}
