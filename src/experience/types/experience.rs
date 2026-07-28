// src/experience/types/experience.rs
#![allow(dead_code)]

// Experience struct and related types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::context::ExperienceContext;
use super::maturity::KnowledgeMaturity;
use super::outcome::ExperienceOutcome;

/// Categories of experiences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperienceType {
    ToolExecution,
    MemoryLookup,
    MemoryStore,
    Workflow,
    Planning,
    Exploration,
    Hypothesis,
    Reflection,
    Learning,
    Conversation,
    UserFeedback,
    ModelInference,
    Error,
    System,
    Custom(String),
}

/// A single recorded experience within the system.
///
/// Per Architecture §07 Design Invariants:
/// - Every experience originates from one or more observations
/// - Experiences are immutable once committed
/// - Historical data is never destroyed, only archived
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    /// Unique identifier.
    pub id: Uuid,

    /// When the experience occurred.
    pub timestamp: DateTime<Utc>,

    /// Observation IDs that originated this experience (Architecture §07 invariant)
    pub observation_ids: Vec<Uuid>,

    /// Category of experience.
    pub experience_type: ExperienceType,

    /// Human-readable title.
    pub title: String,

    /// Detailed description.
    pub description: String,

    /// Context surrounding the experience.
    pub context: ExperienceContext,

    /// Outcome of the experience.
    pub outcome: ExperienceOutcome,

    /// Calculated later by scorer.rs
    pub score: Option<super::ExperienceScore>,

    /// Encounters contributing to this experience.
    pub encounter_ids: Vec<Uuid>,

    /// Current maturity level.
    pub maturity: KnowledgeMaturity,

    /// Overall confidence (updated through evidence, never manually)
    pub confidence: f32,

    /// Lessons learned.
    pub lessons: Vec<String>,

    /// Supporting evidence count.
    pub evidence_count: usize,

    /// Searchable tags.
    pub tags: Vec<String>,

    /// Whether this experience has been committed (immutable after this)
    pub committed: bool,

    /// Whether this experience has been archived (soft-delete, not destroyed)
    pub archived: bool,

    /// When archived (if applicable)
    pub archived_at: Option<DateTime<Utc>>,

    /// Arbitrary metadata.
    pub metadata: HashMap<String, String>,
}

impl Experience {
    /// Create a new uncommitted experience with observation origins
    pub fn new(
        title: String,
        description: String,
        experience_type: ExperienceType,
        observation_ids: Vec<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            observation_ids,
            experience_type,
            title,
            description,
            context: ExperienceContext::default(),
            outcome: ExperienceOutcome::success(),
            score: None,
            encounter_ids: Vec::new(),
            maturity: KnowledgeMaturity::Emerging,
            confidence: 0.5,
            lessons: Vec::new(),
            evidence_count: 0,
            tags: Vec::new(),
            committed: false,
            archived: false,
            archived_at: None,
            metadata: HashMap::new(),
        }
    }

    /// Commit this experience (makes it immutable)
    /// Returns error if already committed (Architecture §07 invariant)
    pub fn commit(&mut self) -> Result<(), &'static str> {
        if self.committed {
            return Err("Experience already committed (immutable)");
        }
        self.committed = true;
        Ok(())
    }

    /// Archive this experience (soft-delete, not destroy)
    /// Per Architecture §07: "Historical data is never destroyed, only archived"
    pub fn archive(&mut self) -> Result<(), &'static str> {
        if self.archived {
            return Err("Experience already archived");
        }
        self.archived = true;
        self.archived_at = Some(Utc::now());
        Ok(())
    }

    /// Add evidence to this experience
    /// Per Architecture §07: "Confidence is updated through evidence, never manually"
    pub fn add_evidence(&mut self, _evidence_id: Uuid) {
        self.evidence_count += 1;
    }
}
