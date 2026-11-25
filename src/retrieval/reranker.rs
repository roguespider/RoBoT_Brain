//! Re-ranking logic per Architecture Chapter 16.
//! Applies secondary ranking passes to refine candidate order.

use crate::retrieval::RetrievalCandidate;
use std::collections::HashSet;

/// Re-ranks candidates using a secondary scoring function.
///
/// The re-ranker takes already-ranked candidates and applies additional
/// signals to adjust their order.
pub fn rerank_candidates(
    candidates: &[(RetrievalCandidate, f32)],
    limit: usize,
) -> Vec<(RetrievalCandidate, f32)> {
    // Apply a diversity boost to prevent dominance by a single source.
    let mut seen_sources: HashSet<String> = HashSet::new();
    let mut adjusted: Vec<(RetrievalCandidate, f32)> = Vec::new();

    for (candidate, score) in candidates {
        let diversity_penalty = if seen_sources.contains(&candidate.source) {
            0.1
        } else {
            0.0
        };
        seen_sources.insert(candidate.source.clone());

        let adjusted_score = score - diversity_penalty;
        adjusted.push((candidate.clone(), adjusted_score));
    }

    adjusted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    adjusted.truncate(limit);
    adjusted
}
