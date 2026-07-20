// robot/src/experience/hypothesis/core/hypothesis.rs

//! ============================================================================
//! HYPOTHESIS
//! ============================================================================
//!
//! Core data structures representing hypotheses.
//!
//! A hypothesis is an evolving belief formed from accumulated experiences.
//! It is neither permanently true nor false. Instead, it gains or loses
//! confidence as additional evidence is observed.
//!
//! This module intentionally contains no persistence or evaluation logic.
//! Those responsibilities belong to the repository and evaluator services.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ============================================================================
/// HYPOTHESIS
/// ============================================================================

/// A belief formed from one or more experiences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    /// Unique identifier.
    pub id: HypothesisId,

    /// Human-readable title.
    pub title: String,

    /// Detailed description.
    pub description: String,

    /// High-level classification.
    pub category: HypothesisCategory,

    /// Relative importance.
    pub priority: HypothesisPriority,

    /// Current lifecycle state.
    pub status: HypothesisStatus,

    /// Current confidence score.
    pub confidence: HypothesisConfidence,

    /// Tags for searching/filtering.
    pub tags: Vec<String>,

    /// Supporting evidence IDs.
    pub supporting_evidence: Vec<String>,

    /// Contradicting evidence IDs.
    pub contradicting_evidence: Vec<String>,

    /// Number of evaluations performed.
    pub evaluations: u32,

    /// Number of confirmations.
    pub confirmations: u32,

    /// Number of contradictions.
    pub contradictions: u32,

    /// Creation timestamp.
    pub created_at: DateTime<Utc>,

    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,

    /// Optional free-form metadata.
    pub metadata: HypothesisMetadata,
}

/// ============================================================================
/// IDENTIFIER
/// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HypothesisId(pub String);

impl HypothesisId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Default for HypothesisId {
    fn default() -> Self {
        Self::new()
    }
}

/// ============================================================================
/// STATUS
/// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HypothesisStatus {
    Draft,
    Active,
    Supported,
    Rejected,
    Archived,
}

impl Default for HypothesisStatus {
    fn default() -> Self {
        Self::Draft
    }
}

/// ============================================================================
/// CATEGORY
/// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HypothesisCategory {
    Behavioral,
    Preference,
    Performance,
    Environmental,
    Social,
    Workflow,
    Knowledge,
    Technical,
    Prediction,
    Other,
}

impl Default for HypothesisCategory {
    fn default() -> Self {
        Self::Other
    }
}

/// ============================================================================
/// PRIORITY
/// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HypothesisPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for HypothesisPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// ============================================================================
/// CONFIDENCE
/// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HypothesisConfidence {
    /// Confidence between 0.0 and 1.0.
    pub value: f32,
}

impl HypothesisConfidence {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
        }
    }

    pub fn increase(&mut self, amount: f32) {
        self.value = (self.value + amount).clamp(0.0, 1.0);
    }

    pub fn decrease(&mut self, amount: f32) {
        self.value = (self.value - amount).clamp(0.0, 1.0);
    }

    pub fn is_confident(&self) -> bool {
        self.value >= 0.80
    }

    pub fn is_uncertain(&self) -> bool {
        self.value <= 0.30
    }
}

impl Default for HypothesisConfidence {
    fn default() -> Self {
        Self::new(0.5)
    }
}

/// ============================================================================
/// METADATA
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HypothesisMetadata {
    /// Who or what created the hypothesis.
    pub source: String,

    /// Optional notes.
    pub notes: Option<String>,

    /// Arbitrary labels.
    pub labels: Vec<String>,
}

/// ============================================================================
/// IMPLEMENTATION
/// ============================================================================

impl Hypothesis {
    /// Create a new hypothesis.
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();

        Self {
            id: HypothesisId::new(),

            title: title.into(),
            description: description.into(),

            category: HypothesisCategory::default(),
            priority: HypothesisPriority::default(),
            status: HypothesisStatus::Draft,

            confidence: HypothesisConfidence::default(),

            tags: Vec::new(),

            supporting_evidence: Vec::new(),
            contradicting_evidence: Vec::new(),

            evaluations: 0,
            confirmations: 0,
            contradictions: 0,

            created_at: now,
            updated_at: now,

            metadata: HypothesisMetadata::default(),
        }
    }

    /// Add a tag.
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.push(tag.into());
    }

    /// Record supporting evidence.
    pub fn add_supporting_evidence(&mut self, evidence_id: impl Into<String>) {
        self.supporting_evidence.push(evidence_id.into());
        self.updated_at = Utc::now();
    }

    /// Record contradicting evidence.
    pub fn add_contradicting_evidence(&mut self, evidence_id: impl Into<String>) {
        self.contradicting_evidence.push(evidence_id.into());
        self.updated_at = Utc::now();
    }

    /// Total evidence count.
    pub fn evidence_count(&self) -> usize {
        self.supporting_evidence.len() + self.contradicting_evidence.len()
    }

    /// Whether this hypothesis has any evidence.
    pub fn has_evidence(&self) -> bool {
        self.evidence_count() > 0
    }

    /// Mark the hypothesis as updated.
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}
