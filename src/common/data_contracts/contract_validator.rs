//! Contract Validator — Runtime schema enforcement for Data Contracts (Chapter 5).
//!
//! Per Architecture Chapter 5.2 (Design Principles):
//! - Immutable by Default: contracts are validated, not modified
//! - Versioned: every contract carries CONTRACT_VERSION
//! - Self-Describing: metadata includes origin, timestamp, correlation
//! - Explainable: validation errors include reason and suggested fix
//! - Serializable: contracts must serialize cleanly
//!
//! Wiring: data_contracts/ -> database/ (DB init validates contracts) -> bridge/app/initialization/db.rs

use crate::common::data_contracts::version::CONTRACT_VERSION;

/// Validation result for a single contract.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    /// Contract passes all checks.
    Valid,
    /// Contract fails with a descriptive message.
    Invalid { reason: String, field: String },
}

/// A contract validator that enforces schema rules at runtime.
#[derive(Debug, Clone, Default)]
pub struct ContractValidator;

impl ContractValidator {
    /// Create a new validator.
    pub fn new() -> Self {
        Self
    }

    /// Validate that a version string matches the current contract version.
    pub fn validate_version(version: &str) -> ValidationResult {
        if version == CONTRACT_VERSION {
            ValidationResult::Valid
        } else {
            ValidationResult::Invalid {
                reason: format!(
                    "version mismatch: expected {}, got {}",
                    CONTRACT_VERSION, version
                ),
                field: "version".to_string(),
            }
        }
    }

    /// Validate that a correlation ID is non-empty.
    pub fn validate_correlation_id(id: &str) -> ValidationResult {
        if id.is_empty() {
            ValidationResult::Invalid {
                reason: "correlation_id must not be empty".to_string(),
                field: "correlation_id".to_string(),
            }
        } else {
            ValidationResult::Valid
        }
    }

    /// Validate that a source identifier is non-empty.
    pub fn validate_source(source: &str) -> ValidationResult {
        if source.is_empty() {
            ValidationResult::Invalid {
                reason: "source must not be empty".to_string(),
                field: "source".to_string(),
            }
        } else {
            ValidationResult::Valid
        }
    }

    /// Validate that a confidence value is within [0.0, 1.0].
    pub fn validate_confidence(confidence: f32) -> ValidationResult {
        if (0.0..=1.0).contains(&confidence) {
            ValidationResult::Valid
        } else {
            ValidationResult::Invalid {
                reason: format!("confidence must be in [0.0, 1.0], got {}", confidence),
                field: "confidence".to_string(),
            }
        }
    }

    /// Run all validation checks on a contract descriptor.
    pub fn validate_contract(
        version: &str,
        correlation_id: &str,
        source: &str,
        confidence: f32,
    ) -> Vec<ValidationResult> {
        vec![
            Self::validate_version(version),
            Self::validate_correlation_id(correlation_id),
            Self::validate_source(source),
            Self::validate_confidence(confidence),
        ]
    }

    /// Check whether all results are valid.
    pub fn all_valid(results: &[ValidationResult]) -> bool {
        results.iter().all(|r| matches!(r, ValidationResult::Valid))
    }
}
