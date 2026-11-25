//! Security Trust — Per Architecture Chapter 25.3 (Trust Model).
//!
//! Trust in RoBoT is a collection of independent measurements: knowledge
//! confidence, skill confidence, relationship confidence, and action confidence.

/// A trust score for a specific domain.
#[derive(Debug, Clone)]
pub struct TrustScore {
    /// Domain being scored (e.g., "knowledge", "skill", "relationship", "action").
    pub domain: String,
    /// Target identifier within the domain.
    pub target_id: String,
    /// Trust score (0.0-1.0).
    pub score: f32,
    /// Evidence supporting the score.
    pub evidence: Vec<String>,
}

/// Trust system managing scores across domains.
#[derive(Debug, Clone, Default)]
pub struct TrustSystem {
    /// All trust scores.
    scores: Vec<TrustScore>,
}

impl TrustSystem {
    /// Create a new empty trust system.
    pub fn new() -> Self {
        Self { scores: Vec::new() }
    }

    /// Set a trust score for a target.
    pub fn set_score(&mut self, domain: String, target_id: String, score: f32) {
        let score = score.clamp(0.0, 1.0);
        if let Some(existing) = self
            .scores
            .iter_mut()
            .find(|s| s.domain == domain && s.target_id == target_id)
        {
            existing.score = score;
        } else {
            self.scores.push(TrustScore {
                domain,
                target_id,
                score,
                evidence: Vec::new(),
            });
        }
    }

    /// Get a trust score.
    pub fn get(&self, domain: &str, target_id: &str) -> Option<f32> {
        self.scores
            .iter()
            .find(|s| s.domain == domain && s.target_id == target_id)
            .map(|s| s.score)
    }

    /// Count total trust scores.
    pub fn count(&self) -> usize {
        self.scores.len()
    }
}
