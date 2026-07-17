// robot/src/experience/hypothesis/core/evidence.rs

//! ============================================================================
//! EVIDENCE
//! ============================================================================
//!
//! Evidence represents observations that support or contradict a hypothesis.
//!
//! Evidence is intentionally independent from hypothesis evaluation. It simply
//! describes what was observed and how trustworthy that observation is.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ============================================================================
/// EVIDENCE
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Unique identifier.
    pub id: EvidenceId,

    /// Human-readable summary.
    pub title: String,

    /// Optional details.
    pub description: String,

    /// Where the evidence originated.
    pub source: EvidenceSource,

    /// Whether this supports or contradicts a hypothesis.
    pub relationship: EvidenceRelationship,

    /// Estimated strength.
    pub strength: EvidenceStrength,

    /// Confidence in this evidence.
    pub confidence: f32,

    /// Optional related experience.
    pub experience_id: Option<ExperienceId>,

    /// Optional related hypothesis.
    pub hypothesis_id: Option<HypothesisId>,

    /// Creation time.
    pub created_at: DateTime<Utc>,

    /// Additional metadata.
    pub metadata: EvidenceMetadata,
}

/// ============================================================================
/// IDENTIFIER
/// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EvidenceId(pub String);

impl EvidenceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Default for EvidenceId {
    fn default() -> Self {
        Self::new()
    }
}

/// ============================================================================
/// SOURCE
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceSource {
    Experience,
    UserFeedback,
    Exploration,
    Reputation,
    Memory,
    Observation,
    External,
    Simulation,
    Other(String),
}

/// ============================================================================
/// RELATIONSHIP
/// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EvidenceRelationship {
    Supports,
    Contradicts,
    Neutral,
}

/// ============================================================================
/// STRENGTH
/// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EvidenceStrength {
    VeryWeak,
    Weak,
    Moderate,
    Strong,
    VeryStrong,
}

impl EvidenceStrength {
    pub fn weight(&self) -> f32 {
        match self {
            Self::VeryWeak => 0.10,
            Self::Weak => 0.30,
            Self::Moderate => 0.50,
            Self::Strong => 0.75,
            Self::VeryStrong => 1.00,
        }
    }
}

impl Default for EvidenceStrength {
    fn default() -> Self {
        Self::Moderate
    }
}

/// ============================================================================
/// METADATA
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidenceMetadata {
    /// Optional notes.
    pub notes: Option<String>,

    /// Arbitrary tags.
    pub tags: Vec<String>,

    /// Optional source name.
    pub source_name: Option<String>,
}

/// ============================================================================
/// IMPLEMENTATION
/// ============================================================================

impl Evidence {
    pub fn new(title: impl Into<String>, relationship: EvidenceRelationship) -> Self {
        Self {
            id: EvidenceId::new(),

            title: title.into(),
            description: String::new(),

            source: EvidenceSource::Experience,
            relationship,

            strength: EvidenceStrength::default(),
            confidence: 0.50,

            experience_id: None,
            hypothesis_id: None,

            created_at: Utc::now(),

            metadata: EvidenceMetadata::default(),
        }
    }

    pub fn supports(&self) -> bool {
        matches!(self.relationship, EvidenceRelationship::Supports)
    }

    pub fn contradicts(&self) -> bool {
        matches!(self.relationship, EvidenceRelationship::Contradicts)
    }

    pub fn neutral(&self) -> bool {
        matches!(self.relationship, EvidenceRelationship::Neutral)
    }

    pub fn weight(&self) -> f32 {
        self.strength.weight() * self.confidence.clamp(0.0, 1.0)
    }

    pub fn set_confidence(&mut self, confidence: f32) {
        self.confidence = confidence.clamp(0.0, 1.0);
    }

    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.metadata.tags.push(tag.into());
    }
}
