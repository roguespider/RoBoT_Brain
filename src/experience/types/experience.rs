// src/experience/types/experience.rs

// Experience struct and related types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::OutcomeKind;
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

    /// Lessons learned (§15 / T10.15).
    #[serde(alias = "lessons")]
    pub lessons_learned: Vec<String>,

    /// Objective pursued in this experience (§15 / T10.1).
    pub objective: String,

    /// Initial assumptions before action (§15 / T10.2).
    pub initial_assumptions: Vec<String>,

    /// Plan that was executed (§15 / T10.3).
    pub plan: String,

    /// Actions taken (§15 / T10.4).
    pub actions: Vec<String>,

    /// Tools used (§15 / T10.5).
    pub tools_used: Vec<String>,

    /// Results obtained (§15 / T10.6).
    pub results: Vec<String>,

    /// Failures encountered (§15 / T10.7).
    pub failures: Vec<String>,

    /// Corrections applied (§15 / T10.8).
    pub corrections: Vec<String>,

    /// Strategies that worked (§15 / T10.9).
    pub successful_strategies: Vec<String>,

    /// Strategies that failed (§15 / T10.10).
    pub unsuccessful_strategies: Vec<String>,

    /// Constraints discovered (§15 / T10.11).
    pub discovered_constraints: Vec<String>,

    /// Capabilities discovered (§15 / T10.12).
    pub discovered_capabilities: Vec<String>,

    /// Final outcome summary (§15 / T10.13).
    pub final_outcome: String,

    /// Supporting evidence count.
    pub evidence_count: usize,

    /// Evidence IDs that contributed to this experience's confidence.
    pub evidence_ids: Vec<Uuid>,

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
            lessons_learned: Vec::new(),
            objective: String::new(),
            initial_assumptions: Vec::new(),
            plan: String::new(),
            actions: Vec::new(),
            tools_used: Vec::new(),
            results: Vec::new(),
            failures: Vec::new(),
            corrections: Vec::new(),
            successful_strategies: Vec::new(),
            unsuccessful_strategies: Vec::new(),
            discovered_constraints: Vec::new(),
            discovered_capabilities: Vec::new(),
            final_outcome: String::new(),
            evidence_count: 0,
            evidence_ids: Vec::new(),
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
    pub fn add_evidence(&mut self, evidence_id: Uuid) {
        // Track which evidence IDs contributed to this experience's confidence
        self.evidence_ids.push(evidence_id);
        self.evidence_count += 1;
    }

    /// Returns a PostTaskEvaluation by analyzing this experience's outcome and metadata.
    pub fn evaluate_post_task(&self) -> crate::cooboploop::post_task::PostTaskEvaluation {
        use crate::cooboploop::post_task::PostTaskEvaluation;
        let mut eval = PostTaskEvaluation::new();
        eval.set_did_succeed(self.outcome.kind == OutcomeKind::Success);
        eval.set_verification_confirmed(self.outcome.kind == OutcomeKind::Success);
        if let Some(ref msg) = self.outcome.message {
            eval.add_unexpected_problem(msg.clone());
        }
        for lesson in &self.lessons_learned {
            eval.add_knowledge_gap(lesson.clone());
        }
        eval.set_efficiency_score(self.score.as_ref().map(|s| s.importance).unwrap_or(0.5));
        eval
    }

    /// Wire evaluate_post_task by calling it in a summary method.
    pub fn post_task_summary(&self) -> String {
        let eval = self.evaluate_post_task();
        eval.summarize()
    }
}
