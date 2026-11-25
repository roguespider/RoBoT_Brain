//! Confidence types — domains, scores, thresholds, history, evidence, summary.

use chrono::Utc;

/// Confidence domain categories.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConfidenceDomain {
    /// Fact confidence.
    Fact,
    /// Relationship confidence.
    Relationship,
    /// Experience confidence.
    Experience,
    /// Skill confidence.
    Skill,
    /// Workflow confidence.
    Workflow,
    /// Tool confidence.
    Tool,
    /// Strategy confidence.
    Strategy,
    /// Prompt quality confidence.
    Prompt,
}

impl ConfidenceDomain {
    /// Return domain label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Fact => "Fact",
            Self::Relationship => "Relationship",
            Self::Experience => "Experience",
            Self::Skill => "Skill",
            Self::Workflow => "Workflow",
            Self::Tool => "Tool",
            Self::Strategy => "Strategy",
            Self::Prompt => "Prompt",
        }
    }
}

/// A confidence score with provenance.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfidenceScore {
    /// The domain this score applies to.
    pub domain: ConfidenceDomain,
    /// The score value (0.0 to 1.0).
    pub value: f32,
    /// Source of the confidence assessment.
    pub source: String,
    /// Timestamp of assessment.
    pub timestamp: i64,
    /// Evidence supporting this score.
    pub evidence: Vec<String>,
}

impl ConfidenceScore {
    /// Create a new confidence score.
    pub fn new(domain: ConfidenceDomain, value: f32, source: &str) -> Self {
        Self {
            domain,
            value: value.clamp(0.0, 1.0),
            source: source.to_string(),
            timestamp: Utc::now().timestamp(),
            evidence: Vec::new(),
        }
    }

    /// Add evidence.
    pub fn with_evidence(mut self, evidence: &str) -> Self {
        self.evidence.push(evidence.to_string());
        self
    }

    /// Check if score exceeds a threshold.
    pub fn exceeds(&self, threshold: f32) -> bool {
        self.value >= threshold
    }

    /// Check if score exceeds the high threshold.
    pub fn exceeds_high(&self, thresholds: &ConfidenceThresholds) -> bool {
        self.exceeds(thresholds.high)
    }
}

/// Confidence thresholds for different decision contexts.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfidenceThresholds {
    /// High confidence threshold (e.g., 0.85).
    pub high: f32,
    /// Medium confidence threshold (e.g., 0.60).
    pub medium: f32,
    /// Low confidence threshold (e.g., 0.30).
    pub low: f32,
}

impl Default for ConfidenceThresholds {
    fn default() -> Self {
        Self {
            high: 0.85,
            medium: 0.60,
            low: 0.30,
        }
    }
}

/// A recorded confidence change for audit/history purposes.
#[derive(Debug, Clone)]
pub struct ConfidenceHistoryEntry {
    /// The identifier of the scored item.
    pub id: String,
    /// Previous confidence value.
    pub old_value: f32,
    /// New confidence value.
    pub new_value: f32,
    /// Reason for the change.
    pub reason: String,
    /// Timestamp of the change.
    pub timestamp: i64,
}

/// Evidence-based confidence update direction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceDirection {
    /// Supporting evidence (increases confidence).
    Support,
    /// Contradicting evidence (decreases confidence).
    Contradict,
}

impl EvidenceDirection {
    /// Return the label for this direction.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Support => "support",
            Self::Contradict => "contradict",
        }
    }
}

/// Summary statistics of all tracked confidence scores.
#[derive(Debug, Clone)]
pub struct ConfidenceSummary {
    /// Total number of tracked scores.
    pub total_scores: usize,
    /// Average confidence across all scores.
    pub avg_confidence: f32,
    /// Minimum confidence.
    pub min_confidence: f32,
    /// Maximum confidence.
    pub max_confidence: f32,
    /// Count of high-confidence scores.
    pub high_confidence_count: usize,
    /// Count of medium-confidence scores.
    pub medium_confidence_count: usize,
    /// Count of low-confidence scores.
    pub low_confidence_count: usize,
}
