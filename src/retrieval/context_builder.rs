//! Context assembly per Architecture Chapter 16.
//! Builds final context packages from ranked and filtered candidates.

/// A constructed context package for consumption by downstream systems.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextPackage {
    /// The assembled context text.
    pub text: String,
    /// Source attributions for the context.
    pub sources: Vec<String>,
    /// Total token estimate for the context.
    pub token_estimate: usize,
    /// Number of candidates included.
    pub candidate_count: usize,
}

impl ContextPackage {
    /// Create a new empty context package.
    pub fn empty() -> Self {
        Self {
            text: String::new(),
            sources: Vec::new(),
            token_estimate: 0,
            candidate_count: 0,
        }
    }

    /// Check if the package has content.
    pub fn has_content(&self) -> bool {
        !self.text.is_empty()
    }
}

/// Build a context package from ranked candidates.
pub fn build_context(
    candidates: &[crate::retrieval::RetrievalCandidate],
) -> ContextPackage {
    let mut text_parts = Vec::new();
    let mut sources = Vec::new();
    let mut token_estimate = 0usize;

    for candidate in candidates {
        let entry = format!("[{}] {}", candidate.source, candidate.content);
        let entry_len = entry.len();
        text_parts.push(entry);
        sources.push(candidate.source.clone());
        token_estimate += entry_len / 4; // Rough token estimate: 4 chars per token
    }

    ContextPackage {
        text: text_parts.join("\n\n"),
        sources,
        token_estimate,
        candidate_count: candidates.len(),
    }
}
