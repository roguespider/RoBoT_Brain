// /src/experience/evolution/evidence.rs
// Evidence for behavior evaluation

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Evidence supporting or contradicting a behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionEvidence {
    /// Unique identifier
    pub id: String,

    /// ID of the behavior this evidence supports
    pub behavior_id: String,

    /// Type of evidence
    pub evidence_type: EvidenceType,

    /// Description of what happened
    pub description: String,

    /// Whether this evidence supports or contradicts the behavior
    pub verdict: EvidenceVerdict,

    /// Confidence in this evidence (0.0 - 1.0)
    pub confidence: f32,

    /// When the evidence was collected
    pub collected_at: DateTime<Utc>,
}

/// Type of evidence collected
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvidenceType {
    /// Direct application result
    ApplicationResult,
    
    /// Comparison with alternative approaches
    Comparison,
    
    /// Expert feedback
    ExpertFeedback,
    
    /// Automated test result
    TestResult,
    
    /// User satisfaction rating
    UserRating,
    
    /// Observational data
    Observation,
    
    /// Historical analysis
    Historical,
}

/// Verdict of the evidence
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvidenceVerdict {
    /// Supports the behavior
    Supports,
    
    /// Contradicts the behavior
    Contradicts,
    
    /// Neutral/inconclusive
    Neutral,
}

impl EvolutionEvidence {
    /// Create new evidence
    pub fn new(
        id: impl Into<String>,
        behavior_id: impl Into<String>,
        evidence_type: EvidenceType,
        description: impl Into<String>,
        verdict: EvidenceVerdict,
    ) -> Self {
        Self {
            id: id.into(),
            behavior_id: behavior_id.into(),
            evidence_type,
            description: description.into(),
            verdict,
            confidence: 0.5,
            collected_at: Utc::now(),
        }
    }

    /// Create supporting evidence
    pub fn supporting(
        id: impl Into<String>,
        behavior_id: impl Into<String>,
        evidence_type: EvidenceType,
        description: impl Into<String>,
    ) -> Self {
        Self::new(id, behavior_id, evidence_type, description, EvidenceVerdict::Supports)
    }

    /// Create contradicting evidence
    pub fn contradicting(
        id: impl Into<String>,
        behavior_id: impl Into<String>,
        evidence_type: EvidenceType,
        description: impl Into<String>,
    ) -> Self {
        Self::new(id, behavior_id, evidence_type, description, EvidenceVerdict::Contradicts)
    }

    /// Create neutral evidence
    pub fn neutral(
        id: impl Into<String>,
        behavior_id: impl Into<String>,
        evidence_type: EvidenceType,
        description: impl Into<String>,
    ) -> Self {
        Self::new(id, behavior_id, evidence_type, description, EvidenceVerdict::Neutral)
    }

    /// Set confidence level
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }
}
