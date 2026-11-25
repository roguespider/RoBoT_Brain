// src/experience/types/outcome.rs
// Outcome types for experiences

use serde::{Deserialize, Serialize};

/// Overall outcome kind.
///
/// The wire format is the PascalCase enum variant (`Success`, `Failure`, ...)
/// documented for the `record_experience` tool. The lowercase spellings are
/// accepted as aliases so clients that send `success` keep working.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
pub enum OutcomeKind {
    #[serde(alias = "success")]
    Success,
    #[serde(alias = "failure")]
    Failure,
    #[serde(alias = "partial")]
    Partial,
    #[serde(alias = "timeout")]
    Timeout,
    #[serde(alias = "interrupted")]
    Interrupted,
    #[serde(alias = "unknown")]
    Unknown,
}

/// Outcome of an experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceOutcome {
    /// Overall outcome.
    pub kind: OutcomeKind,

    /// Optional informational message.
    pub message: Option<String>,

    /// Optional error message.
    pub error: Option<String>,

    /// Execution duration in milliseconds.
    pub duration_ms: Option<u64>,
}

impl ExperienceOutcome {
    /// Successful execution.
    pub fn success() -> Self {
        Self {
            kind: OutcomeKind::Success,
            message: None,
            error: None,
            duration_ms: None,
        }
    }

    /// Partially successful execution.
    pub fn partial(message: impl Into<String>) -> Self {
        Self {
            kind: OutcomeKind::Partial,
            message: Some(message.into()),
            error: None,
            duration_ms: None,
        }
    }

    /// Failed execution.
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            kind: OutcomeKind::Failure,
            message: None,
            error: Some(error.into()),
            duration_ms: None,
        }
    }

    /// Timed out.
    pub fn timeout() -> Self {
        Self {
            kind: OutcomeKind::Timeout,
            message: None,
            error: None,
            duration_ms: None,
        }
    }

    /// Interrupted before completion.
    pub fn interrupted() -> Self {
        Self {
            kind: OutcomeKind::Interrupted,
            message: None,
            error: None,
            duration_ms: None,
        }
    }
}
