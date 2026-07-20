// /src/experience/reflection/services/validator.rs
// Validates reflections for quality and consistency

use super::super::{Reflection, ReflectionStatus};

/// Validation result with any issues found
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<ValidationIssue>,
    pub warnings: Vec<String>,
    pub score: f32,
}

/// A specific validation issue
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub code: String,
    pub message: String,
    pub field: Option<String>,
}

/// Severity level for validation issues
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

/// Validates reflections for quality and consistency
pub struct ReflectionValidator {
    min_confidence: f32,
    min_experiences: usize,
    require_description: bool,
    require_summary: bool,
}

impl ReflectionValidator {
    /// Create a validator with default settings
    pub fn new() -> Self {
        Self {
            min_confidence: 0.5,
            min_experiences: 1,
            require_description: true,
            require_summary: true,
        }
    }

    /// Create a validator with custom minimum confidence
    pub fn with_min_confidence(min: f32) -> Self {
        Self {
            min_confidence: min,
            min_experiences: 1,
            require_description: true,
            require_summary: true,
        }
    }

    /// Validate a reflection
    pub fn validate(&self, reflection: &Reflection) -> ValidationResult {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();
        let mut score: f32 = 1.0;

        // Check minimum confidence
        if reflection.confidence.score < self.min_confidence {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                code: "LOW_CONFIDENCE".to_string(),
                message: format!(
                    "Confidence score {} is below minimum {}",
                    reflection.confidence.score, self.min_confidence
                ),
                field: Some("confidence".to_string()),
            });
            score -= 0.3;
        }

        // Check minimum experiences
        if reflection.experience_ids.len() < self.min_experiences {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                code: "INSUFFICIENT_EXPERIENCES".to_string(),
                message: format!(
                    "Only {} experiences referenced, minimum is {}",
                    reflection.experience_ids.len(), self.min_experiences
                ),
                field: Some("experience_ids".to_string()),
            });
            score -= 0.2;
        }

        // Check description requirement
        if self.require_description && reflection.description.is_empty() {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                code: "MISSING_DESCRIPTION".to_string(),
                message: "Reflection should have a detailed description".to_string(),
                field: Some("description".to_string()),
            });
            score -= 0.1;
        }

        // Check summary requirement
        if self.require_summary && reflection.summary.is_empty() {
            warnings.push("Summary is empty - consider adding a brief summary".to_string());
            score -= 0.1;
        }

        // Check for validation status consistency
        if reflection.status == ReflectionStatus::Validated && reflection.confidence.score < 0.7 {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                code: "INVALID_VALIDATION".to_string(),
                message: "Cannot mark as validated with confidence below 0.7".to_string(),
                field: Some("status".to_string()),
            });
            score = 0.0;
        }

        // Check supporting evidence ratio
        let supporting = reflection.confidence.supporting_experiences;
        let contradicting = reflection.confidence.contradictory_experiences;
        if contradicting > supporting {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                code: "MORE_CONTRADICTIONS".to_string(),
                message: "More contradictions than supporting evidence".to_string(),
                field: Some("confidence".to_string()),
            });
            score -= 0.2;
        }

        // Check tags
        if reflection.tags.is_empty() {
            warnings.push("No tags specified - consider adding relevant tags".to_string());
        }

        ValidationResult {
            is_valid: !issues.iter().any(|i| i.severity == IssueSeverity::Error),
            issues,
            warnings,
            score: score.max(0.0),
        }
    }

    /// Quick validation check returning just validity
    pub fn is_valid(&self, reflection: &Reflection) -> bool {
        self.validate(reflection).is_valid
    }

    /// Get validation score (0.0 - 1.0)
    pub fn score(&self, reflection: &Reflection) -> f32 {
        self.validate(reflection).score
    }
}

impl Default for ReflectionValidator {
    fn default() -> Self {
        Self::new()
    }
}
