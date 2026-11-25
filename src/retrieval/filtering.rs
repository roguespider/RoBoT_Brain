//! Context filtering per Architecture Chapter 16.
//! Filters candidates by relevance, confidence, and budget constraints.

use crate::retrieval::RetrievalCandidate;

/// Filter candidates by minimum confidence threshold.
pub fn filter_by_confidence(
    candidates: &[RetrievalCandidate],
    min_confidence: f32,
) -> Vec<RetrievalCandidate> {
    candidates
        .iter()
        .filter(|c| c.confidence >= min_confidence)
        .cloned()
        .collect()
}

/// Filter candidates by token budget, keeping the highest-scoring ones.
pub fn filter_by_budget(
    candidates: &[RetrievalCandidate],
    budget: usize,
) -> Vec<RetrievalCandidate> {
    // Simple budget filter: keep candidates whose content fits within budget.
    let mut remaining = budget;
    let mut filtered = Vec::new();

    for candidate in candidates {
        let content_len = candidate.content.len();
        if content_len <= remaining {
            remaining -= content_len;
            filtered.push(candidate.clone());
        }
    }

    filtered
}

/// Combine confidence and budget filtering.
pub fn filter_candidates(
    candidates: &[RetrievalCandidate],
    budget: usize,
    min_confidence: f32,
) -> Vec<RetrievalCandidate> {
    let by_confidence = filter_by_confidence(candidates, min_confidence);
    filter_by_budget(&by_confidence, budget)
}
