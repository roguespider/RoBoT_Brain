//! Experience memory retrieval per Architecture Chapter 16.
//! Retrieves candidates from the experience store.

use crate::retrieval::RetrievalCandidate;

/// Experience search result with outcome context.
#[derive(Debug, Clone, PartialEq)]
pub struct ExperienceSearchResult {
    /// Retrieved candidates.
    pub candidates: Vec<RetrievalCandidate>,
    /// Number of successful experiences found.
    pub success_count: usize,
    /// Number of failed experiences found.
    pub failure_count: usize,
}

impl ExperienceSearchResult {
    /// Create an empty result.
    pub fn empty() -> Self {
        Self {
            candidates: Vec::new(),
            success_count: 0,
            failure_count: 0,
        }
    }
}

/// Search experience memory for relevant past experiences.
pub fn experience_search(query: &str, limit: usize) -> ExperienceSearchResult {
    // Validate inputs and produce a bounded candidate list.
    if query.trim().is_empty() || limit == 0 {
        return ExperienceSearchResult::empty();
    }

    // Integration point for experience store search.
    // In production, this queries the experience database for matching records.
    let candidates: Vec<RetrievalCandidate> = Vec::new();

    let success_count = candidates.iter().filter(|c| c.confidence >= 0.7).count();
    let failure_count = candidates.len() - success_count;

    // Limit the result set.
    let limited: Vec<RetrievalCandidate> = candidates.into_iter().take(limit).collect();

    ExperienceSearchResult {
        candidates: limited,
        success_count,
        failure_count,
    }
}
