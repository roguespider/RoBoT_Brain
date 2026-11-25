//! Contradiction detection per Architecture Chapter 16.
//! Detects conflicting claims across retrieved sources.

/// A detected contradiction between two sources.
#[derive(Debug, Clone, PartialEq)]
pub struct Contradiction {
    /// The conflicting claim from source A.
    pub claim_a: String,
    /// The conflicting claim from source B.
    pub claim_b: String,
    /// URL or identifier for source A.
    pub source_a: String,
    /// URL or identifier for source B.
    pub source_b: String,
    /// Suggested resolution, if any.
    pub resolution: Option<String>,
}

impl Contradiction {
    /// Create a new contradiction.
    pub fn new(claim_a: String, claim_b: String, source_a: String, source_b: String) -> Self {
        Self {
            claim_a,
            claim_b,
            source_a,
            source_b,
            resolution: None,
        }
    }

    /// Set a resolution for this contradiction.
    pub fn with_resolution(mut self, resolution: String) -> Self {
        self.resolution = Some(resolution);
        self
    }
}

/// Detect contradictions between candidate claims.
pub fn detect_contradictions(
    candidates: &[crate::retrieval::RetrievalCandidate],
) -> Vec<Contradiction> {
    let mut contradictions = Vec::new();

    for (i, a) in candidates.iter().enumerate() {
        for b in candidates.iter().skip(i + 1) {
            // Simple contradiction detection: opposing polarity markers.
            let a_lower = a.content.to_lowercase();
            let b_lower = b.content.to_lowercase();

            let a_negated = a_lower.contains("not") || a_lower.contains("never");
            let b_negated = b_lower.contains("not") || b_lower.contains("never");

            if a_negated != b_negated && a.source != b.source {
                contradictions.push(Contradiction::new(
                    a.content.clone(),
                    b.content.clone(),
                    a.source.clone(),
                    b.source.clone(),
                ));
            }
        }
    }

    contradictions
}
