// robot/src/experience/hypothesis/services/validator.rs

//! ============================================================================
//! HYPOTHESIS VALIDATOR
//! ============================================================================
//!
//! Validates hypotheses for consistency and possible conflicts.
//!
//! The validator does not modify hypotheses.
//! It reports issues that other systems can decide how to handle.
//!
//! Future versions may include:
//! - contradiction graphs
//! - logical reasoning
//! - LLM validation
//! - domain-specific rules

use serde::{Deserialize, Serialize};

use crate::experience::hypothesis::core::{Hypothesis, HypothesisStatus};

/// ============================================================================
/// VALIDATOR
/// ============================================================================

#[derive(Debug, Clone)]
pub struct HypothesisValidator {
    /// Minimum similarity before considering conflict.
    pub conflict_threshold: f32,
}

impl HypothesisValidator {
    pub fn new() -> Self {
        Self {
            conflict_threshold: 0.70,
        }
    }

    /// Validate a single hypothesis.
    pub fn validate(&self, hypothesis: &Hypothesis) -> ValidationReport {
        let mut issues = Vec::new();

        if hypothesis.title.trim().is_empty() {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::MissingTitle,

                description: "Hypothesis has no title.".to_string(),
            });
        }

        if hypothesis.description.trim().is_empty() {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::MissingDescription,

                description: "Hypothesis has no description.".to_string(),
            });
        }

        if !hypothesis.has_evidence() && hypothesis.status != HypothesisStatus::Draft {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::NoEvidence,

                description: "Hypothesis has no supporting evidence.".to_string(),
            });
        }

        ValidationReport {
            valid: issues.is_empty(),

            issues,
        }
    }

    /// Check two hypotheses for possible conflict.
    pub fn check_conflict(
        &self,
        first: &Hypothesis,
        second: &Hypothesis,
    ) -> Option<ConflictReport> {
        let score = self.similarity(first, second);

        if score >= self.conflict_threshold && first.id != second.id {
            return Some(ConflictReport {
                first_id: first.id.clone(),

                second_id: second.id.clone(),

                similarity: score,

                reason: "Potential conflicting hypotheses.".to_string(),
            });
        }

        None
    }

    fn similarity(&self, first: &Hypothesis, second: &Hypothesis) -> f32 {
        let first_text = format!("{} {}", first.title, first.description);

        let second_text = format!("{} {}", second.title, second.description);

        let first_words = Self::words(&first_text);

        let second_words = Self::words(&second_text);

        if first_words.is_empty() || second_words.is_empty() {
            return 0.0;
        }

        let shared = first_words.intersection(&second_words).count();

        let total = first_words.union(&second_words).count();

        shared as f32 / total as f32
    }

    fn words(text: &str) -> std::collections::HashSet<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|word| word.to_string())
            .collect()
    }
}

impl Default for HypothesisValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// ============================================================================
/// VALIDATION RESULTS
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub valid: bool,

    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub issue_type: ValidationIssueType,

    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationIssueType {
    MissingTitle,

    MissingDescription,

    NoEvidence,

    ConflictingEvidence,

    DuplicateHypothesis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictReport {
    pub first_id: crate::experience::hypothesis::core::HypothesisId,

    pub second_id: crate::experience::hypothesis::core::HypothesisId,

    pub similarity: f32,

    pub reason: String,
}
