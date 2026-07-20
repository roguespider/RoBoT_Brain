use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Knowledge gained from exploration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationFinding {
    pub id: String,

    pub timestamp: DateTime<Utc>,

    /// Observation or discovery
    pub description: String,

    /// Confidence in this finding
    pub confidence: f32,

    /// Whether this should become reusable knowledge
    pub promoted: bool,
}
