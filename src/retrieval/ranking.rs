//! Candidate ranking engine per Architecture Chapter 16.
//! Ranks retrieved candidates by relevance, confidence, and importance.

use crate::retrieval::RetrievalCandidate;

/// Ranking configuration and weights.
#[derive(Debug, Clone, PartialEq)]
pub struct RankingConfig {
    /// Weight for relevance score (0.0 - 1.0).
    pub relevance_weight: f32,
    /// Weight for confidence score (0.0 - 1.0).
    pub confidence_weight: f32,
    /// Weight for recency score (0.0 - 1.0).
    pub recency_weight: f32,
    /// Weight for importance score (0.0 - 1.0).
    pub importance_weight: f32,
}

impl Default for RankingConfig {
    fn default() -> Self {
        Self {
            relevance_weight: 0.4,
            confidence_weight: 0.3,
            recency_weight: 0.2,
            importance_weight: 0.1,
        }
    }
}

/// Rank candidates using the configured weights.
pub fn rank_candidates(
    candidates: &[RetrievalCandidate],
    config: &RankingConfig,
) -> Vec<(RetrievalCandidate, f32)> {
    let mut scored: Vec<(RetrievalCandidate, f32)> = candidates
        .iter()
        .map(|c| {
            let score = config.relevance_weight * c.relevance
                + config.confidence_weight * c.confidence
                + config.recency_weight * c.recency
                + config.importance_weight * c.importance;
            (c.clone(), score)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored
}
