//! ExplorationFinding - knowledge gained from exploration.
//!
//! Per Architecture §2.7, findings represent discoveries made during exploration.
//! Findings can be promoted to reusable knowledge.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Knowledge or discovery gained from exploration.
///
/// Findings represent what was learned during exploration activities.
/// They can be promoted to reusable knowledge in the knowledge system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationFinding {
    /// Unique finding identifier
    pub id: String,

    /// When the finding was made
    pub timestamp: DateTime<Utc>,

    /// Description of the discovery or observation
    pub description: String,

    /// Confidence level in this finding (0.0 - 1.0)
    pub confidence: f32,

    /// Whether this finding has been promoted to reusable knowledge
    pub promoted: bool,
}

impl ExplorationFinding {
    /// Create a new finding with the given description.
    pub fn new(id: String, description: String, confidence: f32) -> Self {
        Self {
            id,
            timestamp: Utc::now(),
            description,
            confidence: confidence.clamp(0.0, 1.0),
            promoted: false,
        }
    }

    /// Mark this finding as promoted to knowledge.
    pub fn promote(&mut self) {
        self.promoted = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finding_new_and_promote() {
        // Test new() and promote() - wires up the dead functions
        let mut finding = ExplorationFinding::new(
            "finding-1".to_string(),
            "Discovered a new pattern".to_string(),
            0.85,
        );
        assert!(!finding.promoted);
        
        finding.promote();
        assert!(finding.promoted);
    }
}
