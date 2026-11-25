//! Improvement Candidate — Per Architecture Chapter 26.6 (Improvement Candidates).
//!
//! Improvement candidates await validation before becoming active
//! improvements. Each candidate carries an ID, description, evidence,
//! confidence, and status.

/// An improvement candidate awaiting validation.
#[derive(Debug, Clone)]
pub struct ImprovementCandidate {
    /// Unique candidate identifier.
    pub id: String,
    /// Description of the improvement.
    pub description: String,
    /// Evidence supporting the candidate.
    pub evidence: Vec<String>,
    /// Confidence in the candidate (0.0-1.0).
    pub confidence: f32,
    /// Current status of the candidate.
    pub status: CandidateStatus,
}

/// Status of an improvement candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CandidateStatus {
    /// Candidate has been proposed but not yet tested.
    Proposed,
    /// Candidate is undergoing experimentation.
    Testing,
    /// Candidate has been validated and accepted.
    Accepted,
    /// Candidate has been rejected.
    Rejected,
}

impl ImprovementCandidate {
    /// Create a new improvement candidate.
    pub fn new(id: String, description: String) -> Self {
        Self {
            id,
            description,
            evidence: Vec::new(),
            confidence: 0.5,
            status: CandidateStatus::Proposed,
        }
    }

    /// Add evidence to the candidate.
    pub fn add_evidence(&mut self, evidence: String) {
        self.evidence.push(evidence);
    }

    /// Update confidence.
    pub fn set_confidence(&mut self, confidence: f32) {
        self.confidence = confidence.clamp(0.0, 1.0);
    }

    /// Update status.
    pub fn set_status(&mut self, status: CandidateStatus) {
        self.status = status;
    }
}
