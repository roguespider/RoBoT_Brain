//! Confidence scoring per Architecture Chapter 16.
//! Computes confidence scores for retrieved candidates.

/// Compute a composite confidence score for a candidate.
pub fn compute_confidence(relevance: f32, source_reliability: f32, recency: f32) -> f32 {
    let score = 0.5 * relevance + 0.3 * source_reliability + 0.2 * recency;
    score.clamp(0.0, 1.0)
}

/// Compute confidence based on agreement across multiple sources.
pub fn compute_agreement_confidence(source_scores: &[f32]) -> f32 {
    if source_scores.is_empty() {
        return 0.0;
    }
    let avg = source_scores.iter().sum::<f32>() / source_scores.len() as f32;
    let variance: f32 =
        source_scores.iter().map(|s| (s - avg).powi(2)).sum::<f32>() / source_scores.len() as f32;
    // Lower variance means higher agreement, thus higher confidence.
    (1.0 - variance.min(1.0)).clamp(0.0, 1.0)
}
