//! Learning types - Per Architecture §10.5

/// Errors that can occur during learning operations.
#[derive(Debug, Clone, PartialEq)]
pub enum LearningError {
    NotFound,
    UpdateFailed(String),
}

impl std::fmt::Display for LearningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LearningError::NotFound => write!(f, "Item not found"),
            LearningError::UpdateFailed(msg) => write!(f, "Update failed: {}", msg),
        }
    }
}

impl std::error::Error for LearningError {}
