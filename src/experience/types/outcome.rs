// src/experience/types/outcome.rs
// Outcome types for experiences

use serde::{Deserialize, Serialize};

/// Overall outcome kind.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
pub enum OutcomeKind {
    Success,
    Failure,
    Partial,
    Timeout,
    Interrupted,
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
